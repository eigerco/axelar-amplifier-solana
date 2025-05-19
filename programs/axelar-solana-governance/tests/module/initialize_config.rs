use axelar_solana_gateway_test_fixtures::base::TestFixture;
use axelar_solana_governance::instructions::builder::IxBuilder;
use axelar_solana_governance::state::{GovernanceConfig, VALID_PROPOSAL_DELAY_RANGE};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use crate::fixtures::MINIMUM_PROPOSAL_DELAY;
use crate::helpers::{
    assert_msg_present_in_logs, deploy_governance_program,
    deploy_governance_program_with_upgrade_authority,
};

#[tokio::test]
async fn test_successfully_initialize_config() {
    // Setup
    let mut fixture = TestFixture::new(ProgramTest::default()).await;
    deploy_governance_program(&mut fixture).await;

    let (config_pda, _) = GovernanceConfig::pda();

    let config = GovernanceConfig::new(
        [0_u8; 32],
        [0_u8; 32],
        MINIMUM_PROPOSAL_DELAY,
        Pubkey::new_unique().to_bytes(),
    );

    let ix = IxBuilder::new()
        .initialize_config(&fixture.payer.pubkey(), &config_pda, config.clone())
        .build();

    let res = fixture.send_tx(&[ix]).await;

    // Assert
    assert!(res.is_ok());
    let root_pda_data = fixture
        .get_account_with_borsh::<GovernanceConfig>(&config_pda)
        .await
        .unwrap();
    assert_eq!(&config.address_hash, &root_pda_data.address_hash);
    assert_eq!(&config.chain_hash, &root_pda_data.chain_hash);
    assert_eq!(
        &config.minimum_proposal_eta_delay,
        &root_pda_data.minimum_proposal_eta_delay
    );
    assert_eq!(&config.operator, &root_pda_data.operator);
}

#[tokio::test]
async fn test_program_checks_config_pda_successfully_derived() {
    // Setup
    let mut fixture = TestFixture::new(ProgramTest::default()).await;
    deploy_governance_program(&mut fixture).await;

    let config = GovernanceConfig::new(
        [0_u8; 32],
        [0_u8; 32],
        MINIMUM_PROPOSAL_DELAY,
        Pubkey::new_unique().to_bytes(),
    );

    let ix = IxBuilder::new()
        .initialize_config(
            &fixture.payer.pubkey(),
            &Pubkey::new_unique(),
            config.clone(),
        )
        .build(); // Wrong PDA

    let res = fixture.send_tx(&[ix]).await;

    // Assert
    assert!(res.is_err());
    assert_msg_present_in_logs(
        res.err().unwrap(),
        "Derived PDA does not match provided PDA",
    );
}

#[tokio::test]
async fn test_program_overrides_config_bump() {
    // Setup
    let mut fixture = TestFixture::new(ProgramTest::default()).await;
    deploy_governance_program(&mut fixture).await;

    let (config_pda, _) = GovernanceConfig::pda();

    let config = GovernanceConfig::new(
        [0_u8; 32],
        [0_u8; 32],
        MINIMUM_PROPOSAL_DELAY,
        Pubkey::new_unique().to_bytes(),
    );

    let ix = IxBuilder::new()
        .initialize_config(&fixture.payer.pubkey(), &config_pda, config.clone())
        .build(); // Wrong PDA

    let res = fixture.send_tx(&[ix]).await;
    assert!(res.is_ok());

    let config = fixture
        .get_account_with_borsh::<GovernanceConfig>(&config_pda)
        .await
        .unwrap();

    // Assert
    assert!(config.bump != 0);
}

#[tokio::test]
async fn test_only_deployer_can_initialize_program() {
    // Setup
    let mut fixture = TestFixture::new(ProgramTest::default()).await;
    deploy_governance_program_with_upgrade_authority(&mut fixture, &Pubkey::new_unique()).await; // Wrong deployer

    let (config_pda, _) = GovernanceConfig::pda();

    let config = GovernanceConfig::new(
        [0_u8; 32],
        [0_u8; 32],
        MINIMUM_PROPOSAL_DELAY,
        Pubkey::new_unique().to_bytes(),
    );

    let ix = IxBuilder::new()
        .initialize_config(&fixture.payer.pubkey(), &config_pda, config.clone())
        .build();

    let res = fixture.send_tx(&[ix]).await;

    // Assert
    assert!(res.is_err());
    assert_msg_present_in_logs(
        res.err().unwrap(),
        "Given authority is not the program upgrade authority",
    );
}

#[tokio::test]
async fn test_upper_bound_for_proposal_delay() {
    // Setup
    let mut fixture = TestFixture::new(ProgramTest::default()).await;
    deploy_governance_program(&mut fixture).await;

    let (config_pda, _) = GovernanceConfig::pda();

    let config = GovernanceConfig::new(
        [0_u8; 32],
        [0_u8; 32],
        VALID_PROPOSAL_DELAY_RANGE.end() + 1, // Go up the upper limit, this should fail
        Pubkey::new_unique().to_bytes(),
    );

    let ix = IxBuilder::new()
        .initialize_config(&fixture.payer.pubkey(), &config_pda, config.clone())
        .build();

    let res = fixture.send_tx(&[ix]).await;

    // Assert
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
    // Setup
    let mut fixture = TestFixture::new(ProgramTest::default()).await;
    deploy_governance_program(&mut fixture).await;

    let (config_pda, _) = GovernanceConfig::pda();

    let config = GovernanceConfig::new(
        [0_u8; 32],
        [0_u8; 32],
        VALID_PROPOSAL_DELAY_RANGE.start() - 1, // Go down the lower limit, this should fail
        Pubkey::new_unique().to_bytes(),
    );

    let ix = IxBuilder::new()
        .initialize_config(&fixture.payer.pubkey(), &config_pda, config.clone())
        .build();

    let res = fixture.send_tx(&[ix]).await;

    // Assert
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
