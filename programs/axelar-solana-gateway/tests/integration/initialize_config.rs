use axelar_message_primitives::U256;
use axelar_solana_gateway::get_gateway_root_config_pda;
use axelar_solana_gateway::state::GatewayConfig;
use axelar_solana_gateway_test_fixtures::{
    SolanaAxelarIntegration, SolanaAxelarIntegrationMetadata,
};
use solana_program_test::tokio;
use solana_sdk::clock::Clock;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

fn cmp_config(init: &SolanaAxelarIntegrationMetadata, created: &GatewayConfig) -> bool {
    let current_epoch: U256 = U256::ONE;
    let previous_verifier_retention: U256 = init.previous_signers_retention.into();
    created.operator == init.operator.pubkey()
        && created.domain_separator == init.domain_separator
        && created.current_epoch == current_epoch
        && created.previous_verifier_set_retention == previous_verifier_retention
        && created.minimum_rotation_delay == init.minimum_rotate_signers_delay_seconds
        // this just checks that the last rotation ts has been set to a non-zero value
        && created.last_rotation_timestamp > 0
}

#[allow(clippy::arithmetic_side_effects)]
#[allow(clippy::as_conversions)]
async fn assert_verifier_sets(metadata: &mut SolanaAxelarIntegrationMetadata) {
    let vs_data = metadata.init_gateway_config_verifier_set_data();
    for (idx, (verifier_set_hash, pda)) in vs_data.into_iter().enumerate() {
        let vs_data = metadata.verifier_set_tracker(pda).await;
        let epoch = U256::from_u64(idx as u64 + 1);

        assert_eq!(
            vs_data.epoch, epoch,
            "verifier set tracker not properly initialized"
        );
        assert_eq!(
            vs_data.verifier_set_hash, verifier_set_hash,
            "verifier set tracker not properly initialized"
        );
    }
}

#[tokio::test]
async fn test_successfylly_initialize_config_with_single_initial_signer() {
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42])
        .build()
        .setup_without_init_config()
        .await;
    let (gateway_config_pda, _bump) = get_gateway_root_config_pda();
    let initial_sets = metadata.init_gateway_config_verifier_set_data();
    let ix = axelar_solana_gateway::instructions::initialize_config(
        metadata.fixture.payer.pubkey(),
        metadata.upgrade_authority.pubkey(),
        metadata.domain_separator,
        initial_sets,
        metadata.minimum_rotate_signers_delay_seconds,
        metadata.operator.pubkey(),
        metadata.previous_signers_retention.into(),
        gateway_config_pda,
    )
    .unwrap();

    let signers = &[
        metadata.fixture.payer.insecure_clone(),
        metadata.upgrade_authority.insecure_clone(),
    ];
    metadata
        .send_tx_with_custom_signers(&[ix], signers)
        .await
        .unwrap();

    // Assert -- config derived correctly
    let root_pda_data = metadata.gateway_confg(gateway_config_pda).await;
    assert!(cmp_config(&metadata, &root_pda_data));

    // Assert -- block timestamp updated
    let clock = metadata.get_sysvar::<Clock>().await;
    let block_timestamp: u64 = clock
        .unix_timestamp
        .try_into()
        .expect("got a negative timestamp");
    assert_eq!(
        root_pda_data.last_rotation_timestamp, block_timestamp,
        "timestamp invalid"
    );

    // Assert -- epoch set to the one we expect
    let current_epoch = U256::from(1_u8);
    assert_eq!(root_pda_data.current_epoch, current_epoch);

    // Assert -- verifier set PDAs are initialized
    assert_verifier_sets(&mut metadata).await;
}

#[tokio::test]
async fn test_reverts_on_invalid_gateway_pda_pubkey() {
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42])
        .build()
        .setup_without_init_config()
        .await;
    let initial_sets = metadata.init_gateway_config_verifier_set_data();
    let ix = axelar_solana_gateway::instructions::initialize_config(
        metadata.fixture.payer.pubkey(),
        metadata.upgrade_authority.pubkey(),
        metadata.domain_separator,
        initial_sets,
        metadata.minimum_rotate_signers_delay_seconds,
        metadata.operator.pubkey(),
        metadata.previous_signers_retention.into(),
        Pubkey::new_unique(), // source of failure
    )
    .unwrap();

    let signers = &[
        metadata.fixture.payer.insecure_clone(),
        metadata.upgrade_authority.insecure_clone(),
    ];

    let res = metadata
        .send_tx_with_custom_signers(&[ix], signers)
        .await
        .expect_err("tx should fail");

    // Assert
    assert!(
        res.metadata
            .unwrap()
            .log_messages
            .into_iter()
            .any(|x| x.to_lowercase().contains("invalid gateway root pda")),
        "Expected error message not found!"
    );
}

#[tokio::test]
async fn test_reverts_on_already_initialized_gateway_pda() {
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42])
        .build()
        .setup()
        .await;
    let (gateway_config_pda, _bump) = get_gateway_root_config_pda();
    let initial_sets = metadata.init_gateway_config_verifier_set_data();
    let ix = axelar_solana_gateway::instructions::initialize_config(
        metadata.fixture.payer.pubkey(),
        metadata.upgrade_authority.pubkey(),
        metadata.domain_separator,
        initial_sets,
        metadata.minimum_rotate_signers_delay_seconds,
        metadata.operator.pubkey(),
        metadata.previous_signers_retention.into(),
        gateway_config_pda,
    )
    .unwrap();

    let signers = &[
        metadata.fixture.payer.insecure_clone(),
        metadata.upgrade_authority.insecure_clone(),
    ];

    let res = metadata
        .send_tx_with_custom_signers(&[ix], signers)
        .await
        .expect_err("tx should fail");

    // Assert
    assert!(
        res.metadata
            .unwrap()
            .log_messages
            .into_iter()
            .any(|x| x.contains("invalid account data for instruction")),
        "Expected error message not found!"
    );
}

#[tokio::test]
async fn test_reverts_without_proper_upgrade_authority_signature() {
    // Setup with a different payer account
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42])
        .build()
        .setup_without_init_config()
        .await;

    let (gateway_config_pda, _bump) = get_gateway_root_config_pda();
    let initial_sets = metadata.init_gateway_config_verifier_set_data();

    // Create a different account to be passed in place of the update authority account
    let not_upgrade_authority = Keypair::new();

    // Create instruction with different payer
    let ix = axelar_solana_gateway::instructions::initialize_config(
        metadata.fixture.payer.pubkey(),
        not_upgrade_authority.pubkey(), // Using different account instead of upgrade authority.
        metadata.domain_separator,
        initial_sets,
        metadata.minimum_rotate_signers_delay_seconds,
        metadata.operator.pubkey(),
        metadata.previous_signers_retention.into(),
        gateway_config_pda,
    )
    .unwrap();

    let signers = &[
        not_upgrade_authority,
        metadata.fixture.payer.insecure_clone(),
    ];

    // Execute transaction and expect failure
    let res = metadata
        .send_tx_with_custom_signers(&[ix], signers)
        .await
        .expect_err("tx should fail without proper upgrade authority signature");

    // Assert that the error message indicates the correct failure reason
    assert!(
        res.metadata
            .unwrap()
            .log_messages
            .into_iter()
            .any(|x| x.contains("Given authority is not the program upgrade authority")),
        "Expected error message about invalid upgrade authority was not found!"
    );
}
