use axelar_solana_gateway_test_fixtures::assert_msg_present_in_logs;
use axelar_solana_gateway_test_fixtures::base::TestFixture;
use axelar_solana_governance::instructions::builder::IxBuilder;
use axelar_solana_governance::state::{GovernanceConfig, VALID_PROPOSAL_DELAY_RANGE};
use borsh::BorshSerialize;
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::MINIMUM_PROPOSAL_DELAY;
use crate::helpers::deploy_governance_program;

#[tokio::test]
async fn test_successfully_update_config() {
    let mut fixture = TestFixture::new(ProgramTest::default()).await;
    deploy_governance_program(&mut fixture).await;
    let (config_pda, _) = GovernanceConfig::pda();

    init_gov_config(&mut fixture, &config_pda).await;

    let new_config = GovernanceConfig {
        bump: 40, // This will be preserved from the initial config.
        chain_hash: [1_u8; 32],
        address_hash: [2_u8; 32],
        minimum_proposal_eta_delay: MINIMUM_PROPOSAL_DELAY + 1,
        operator: Pubkey::new_unique().to_bytes(), // Trying to change the operator should have no effect
    };

    let ix = IxBuilder::new()
        .update_config(&fixture.payer.pubkey(), &config_pda, new_config.clone())
        .build();
    let res = fixture.send_tx(&[ix]).await;
    assert!(res.is_ok());
    let updated_config = fixture
        .get_account_with_borsh::<GovernanceConfig>(&config_pda)
        .await
        .unwrap();

    // We assert all allowed fields are updated.
    assert_eq!(&new_config.address_hash, &updated_config.address_hash);
    assert_eq!(&new_config.chain_hash, &updated_config.chain_hash);
    assert_eq!(
        &new_config.minimum_proposal_eta_delay,
        &updated_config.minimum_proposal_eta_delay
    );

    // We are sure the operator field and bump are not changed.
    let initial_config = gov_config_fixture(&fixture.payer.pubkey());
    assert_eq!(&initial_config.operator, &updated_config.operator);
    assert_ne!(initial_config.bump, updated_config.bump);
}

#[tokio::test]
async fn test_program_checks_config_pda_successfully_derived() {
    let program_test = ProgramTest::default();
    let mut fixture = TestFixture::new(program_test).await;
    deploy_governance_program(&mut fixture).await;
    let (config_pda, _) = GovernanceConfig::pda();
    init_gov_config(&mut fixture, &config_pda).await;

    let config = GovernanceConfig::new(
        [1_u8; 32],
        [2_u8; 32],
        MINIMUM_PROPOSAL_DELAY,
        Pubkey::new_unique().to_bytes(),
    );

    let wrong_config_pda = Keypair::new();

    // Store the the config in the wrong pda
    let mut fake_config_account =
        Account::new(1_000_000_000, 10000, &axelar_solana_governance::id());
    config.serialize(&mut fake_config_account.data).unwrap();
    fixture.set_account_state(&wrong_config_pda.pubkey(), fake_config_account);

    // fund the wrong config pda so the transaction does not fail because of insufficient funds
    let ix = solana_sdk::system_instruction::transfer(
        &fixture.payer.pubkey(),
        &wrong_config_pda.pubkey(),
        1_000_000_000,
    );
    fixture.send_tx(&[ix]).await.unwrap();
    // Set governance program as owner of the wrong config pda
    let ix = solana_sdk::system_instruction::assign(
        &wrong_config_pda.pubkey(),
        &axelar_solana_governance::id(),
    );
    fixture
        .send_tx_with_custom_signers(
            &[ix],
            &[
                wrong_config_pda.insecure_clone(),
                fixture.payer.insecure_clone(),
            ],
        )
        .await
        .unwrap();

    let ix = IxBuilder::new()
        .update_config(
            &fixture.payer.pubkey(),
            &wrong_config_pda.pubkey(), // Wrong config PDA
            config.clone(),
        )
        .build();
    let res = fixture.send_tx(&[ix]).await;
    assert!(res.is_err());
    assert_msg_present_in_logs(res.err().unwrap(), "Invalid config/root pda");
}

#[tokio::test]
async fn test_only_operator_can_update_config() {
    let mut fixture = TestFixture::new(ProgramTest::default()).await;
    deploy_governance_program(&mut fixture).await;
    let (config_pda, _) = GovernanceConfig::pda();
    init_gov_config(&mut fixture, &config_pda).await;

    let config = GovernanceConfig::new(
        [1_u8; 32],
        [2_u8; 32],
        MINIMUM_PROPOSAL_DELAY,
        Pubkey::new_unique().to_bytes(),
    );

    // Transfer lamports to a new payer that is not the current operator
    let new_payer = solana_sdk::signature::Keypair::new();
    let ix = solana_sdk::system_instruction::transfer(
        &fixture.payer.pubkey(),
        &new_payer.pubkey(),
        1_000_000_000,
    );
    fixture.send_tx(&[ix]).await.unwrap();
    // try to update the config with the new payer that is not the current operator
    let ix = IxBuilder::new()
        .update_config(&new_payer.pubkey(), &config_pda, config.clone())
        .build();
    let res = fixture
        .send_tx_with_custom_signers(
            &[ix],
            &[fixture.payer.insecure_clone(), new_payer.insecure_clone()],
        )
        .await;
    assert!(res.is_err());
    assert_msg_present_in_logs(
        res.err().unwrap(),
        "Only the current operator can update the governance config",
    );
}

#[tokio::test]
async fn test_operator_must_sign_tx() {
    let mut fixture = TestFixture::new(ProgramTest::default()).await;
    deploy_governance_program(&mut fixture).await;
    let (config_pda, _) = GovernanceConfig::pda();
    init_gov_config(&mut fixture, &config_pda).await;

    let config = GovernanceConfig::new(
        [1_u8; 32],
        [2_u8; 32],
        MINIMUM_PROPOSAL_DELAY,
        Pubkey::new_unique().to_bytes(),
    );

    let non_signer_operator = Keypair::new();

    // try to update the config with the new payer that is not the current operator
    let mut ix = IxBuilder::new()
        .update_config(&non_signer_operator.pubkey(), &config_pda, config.clone())
        .build();

    ix.accounts[0].is_signer = false; // The operator is not a signer
    let res = fixture.send_tx(&[ix]).await;
    assert!(res.is_err());
    assert_msg_present_in_logs(
        res.err().unwrap(),
        "The operator account must sign the transaction",
    );
}

#[tokio::test]
async fn test_upper_bound_for_proposal_delay() {
    let mut fixture = TestFixture::new(ProgramTest::default()).await;
    deploy_governance_program(&mut fixture).await;
    let (config_pda, _) = GovernanceConfig::pda();
    init_gov_config(&mut fixture, &config_pda).await;

    let config = GovernanceConfig::new(
        [1_u8; 32],
        [2_u8; 32],
        VALID_PROPOSAL_DELAY_RANGE.end() + 1, // Go up the upper limit, this should fail
        Pubkey::new_unique().to_bytes(),
    );
    let ix = IxBuilder::new()
        .update_config(&fixture.payer.pubkey(), &config_pda, config.clone())
        .build();
    let res = fixture.send_tx(&[ix]).await;
    assert!(res.is_err());
    assert_msg_present_in_logs(
        res.err().unwrap(),
        &format!(
            "The minimum proposal ETA delay must be among {} and {} seconds",
            VALID_PROPOSAL_DELAY_RANGE.start(),
            VALID_PROPOSAL_DELAY_RANGE.end()
        ),
    );
}

#[tokio::test]
async fn test_lower_bound_for_proposal_delay() {
    let mut fixture = TestFixture::new(ProgramTest::default()).await;
    deploy_governance_program(&mut fixture).await;
    let (config_pda, _) = GovernanceConfig::pda();
    init_gov_config(&mut fixture, &config_pda).await;

    let config = GovernanceConfig::new(
        [1_u8; 32],
        [2_u8; 32],
        VALID_PROPOSAL_DELAY_RANGE.start() - 1, // Go down the lower limit, this should fail
        Pubkey::new_unique().to_bytes(),
    );
    let ix = IxBuilder::new()
        .update_config(&fixture.payer.pubkey(), &config_pda, config.clone())
        .build();
    let res = fixture.send_tx(&[ix]).await;
    assert!(res.is_err());
    assert_msg_present_in_logs(
        res.err().unwrap(),
        &format!(
            "The minimum proposal ETA delay must be among {} and {} seconds",
            VALID_PROPOSAL_DELAY_RANGE.start(),
            VALID_PROPOSAL_DELAY_RANGE.end()
        ),
    );
}

fn gov_config_fixture(operator: &Pubkey) -> GovernanceConfig {
    GovernanceConfig::new(
        [0_u8; 32],
        [0_u8; 32],
        MINIMUM_PROPOSAL_DELAY,
        operator.to_bytes(),
    )
}

async fn init_gov_config(fixture: &mut TestFixture, config_pda: &Pubkey) {
    let config = gov_config_fixture(&fixture.payer.pubkey());

    let ix = IxBuilder::new()
        .initialize_config(&fixture.payer.pubkey(), config_pda, config.clone())
        .build();

    fixture.send_tx(&[ix]).await.unwrap();
}
