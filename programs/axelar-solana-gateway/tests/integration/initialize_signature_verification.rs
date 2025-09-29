use axelar_solana_gateway::get_gateway_root_config_pda;
use axelar_solana_gateway::state::signature_verification::SignatureVerification;
use axelar_solana_gateway_test_fixtures::gateway::random_bytes;
use axelar_solana_gateway_test_fixtures::SolanaAxelarIntegration;
use bytemuck::Zeroable;
use solana_program_test::tokio;
use solana_sdk::signer::Signer;

#[tokio::test]
async fn test_initialize_payload_verification_session() {
    // Setup
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42])
        .build()
        .setup()
        .await;

    // Action
    let payload_merkle_root = random_bytes();
    let gateway_config_pda = get_gateway_root_config_pda().0;
    let signing_verifier_set_hash = metadata.init_gateway_config_verifier_set_data().hash;

    let ix = axelar_solana_gateway::instructions::initialize_payload_verification_session(
        metadata.payer.pubkey(),
        gateway_config_pda,
        payload_merkle_root,
        signing_verifier_set_hash,
    )
    .unwrap();
    let _tx_result = metadata.send_tx(&[ix]).await.unwrap();

    // Check PDA contains the expected data
    let (verification_pda, bump) =
        axelar_solana_gateway::get_signature_verification_pda(&payload_merkle_root);

    let verification_session_account = metadata
        .try_get_account_no_checks(&verification_pda)
        .await
        .ok()
        .flatten()
        .expect("verification session PDA account should exist");

    assert_eq!(
        verification_session_account.owner,
        axelar_solana_gateway::ID
    );

    let session = metadata
        .signature_verification_session(verification_pda)
        .await;

    assert_eq!(session.bump, bump);
    let mut expected_verification = SignatureVerification::zeroed();
    expected_verification.signing_verifier_set_hash = signing_verifier_set_hash;
    assert_eq!(session.signature_verification, expected_verification);
}

#[tokio::test]
async fn test_cannot_initialize_pda_twice() {
    // Setup
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42])
        .build()
        .setup()
        .await;

    // Action: First initialization
    let payload_merkle_root = random_bytes();
    let gateway_config_pda = get_gateway_root_config_pda().0;
    let signing_verifier_set_hash = metadata.init_gateway_config_verifier_set_data().hash;

    let ix = axelar_solana_gateway::instructions::initialize_payload_verification_session(
        metadata.payer.pubkey(),
        gateway_config_pda,
        payload_merkle_root,
        signing_verifier_set_hash,
    )
    .unwrap();
    let _tx_result = metadata.send_tx(&[ix]).await.unwrap();

    // Attempt to initialize the PDA a second time
    let ix_second = axelar_solana_gateway::instructions::initialize_payload_verification_session(
        metadata.payer.pubkey(),
        gateway_config_pda,
        payload_merkle_root,
        signing_verifier_set_hash,
    )
    .unwrap();
    let tx_result_second = metadata.send_tx(&[ix_second]).await.unwrap_err();

    // Assert that the second initialization fails
    assert!(
        tx_result_second.result.is_err(),
        "Second initialization should fail"
    );
}

/// This test verifies that initialize_payload_verification_session properly validates
/// the signing_verifier_set_hash parameter.
#[tokio::test]
async fn test_rejects_invalid_verifier_set_hash() {
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42])
        .build()
        .setup()
        .await;

    let invalid_ix = axelar_solana_gateway::instructions::initialize_payload_verification_session(
        metadata.payer.pubkey(),
        metadata.gateway_root_pda,
        random_bytes(), // payload_merkle_root
        random_bytes(), // invalid verifier_set_hash with no tracker account
    )
    .unwrap();

    let tx_outcome = metadata
        .send_tx(&[invalid_ix])
        .await
        .expect_err("a transaction with an invalid verifier set hash should fail");

    assert!(
        tx_outcome.result.is_err(),
        "expected account-related failure, but got a successful transaction instead",
    );

    // This marks the end of this test. What follows is a confidence check:
    //
    // We redo the same transaction, but this time with a valid verifier_set_hash (the one we have
    // from the metadata value) and observe the transaction passing without errors.

    let valid_ix = axelar_solana_gateway::instructions::initialize_payload_verification_session(
        metadata.payer.pubkey(),
        metadata.gateway_root_pda,
        random_bytes(),
        metadata.init_gateway_config_verifier_set_data().hash, // valid verifier set hash
    )
    .unwrap();

    assert!(
        metadata.send_tx(&[valid_ix]).await.is_ok(),
        "transaction with valid verifier set hash should succeed"
    );
}
