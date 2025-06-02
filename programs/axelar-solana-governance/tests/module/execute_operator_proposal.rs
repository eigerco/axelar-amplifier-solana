use axelar_solana_gateway_test_fixtures::base::FindLog;
use axelar_solana_gateway_test_fixtures::base::TestFixture;
use axelar_solana_governance::events::GovernanceEvent;
use axelar_solana_governance::instructions::builder::{IxBuilder, ProposalRelated};
use borsh::to_vec;
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::operator_keypair;
use crate::helpers::{
    approve_ix_at_gateway, assert_msg_present_in_logs, deploy_governance_program, events,
    gmp_memo_metadata, gmp_sample_metadata, init_contract_with_operator,
    ix_builder_with_memo_proposal_data, ix_builder_with_sample_proposal_data, setup_programs,
};

/// This is a full flow test that tests the execution of a proposal that reaches
/// the memo program by transferring funds.
///
/// On a normal proposal execution flow, this test would fail, as the ETA of the
/// proposal (see test fixtures) wouldn't be satisfied. However, the operator
/// proposal execution doesn't take into account the ETA of the proposal.
#[tokio::test]
async fn test_full_flow_operator_proposal_execution() {
    // Get the operator key pair;
    let operator = operator_keypair();

    let (mut sol_integration, config_pda, counter_pda) = Box::pin(setup_programs()).await;

    let (memo_signing_pda, _) =
        axelar_solana_gateway::get_call_contract_signing_pda(axelar_solana_memo_program::ID);
    // Using the memo program as target proposal program.
    let memo_program_accounts = &[
        AccountMeta::new_readonly(axelar_solana_memo_program::id(), false),
        AccountMeta::new_readonly(counter_pda, false),
        AccountMeta::new_readonly(memo_signing_pda, false),
        AccountMeta::new_readonly(sol_integration.gateway_root_pda, false),
        AccountMeta::new_readonly(axelar_solana_gateway::id(), false),
        AccountMeta::new_readonly(sol_integration.fixture.payer.pubkey(), true),
    ];

    // Send the proposal via GMP acting as Axelar governance infrastructure
    let ix_builder = ix_builder_with_memo_proposal_data(memo_program_accounts, 0, None);
    let meta = gmp_memo_metadata();
    let mut gmp_call_data = ix_builder
        .clone()
        .gmp_ix()
        .with_msg_metadata(meta.clone())
        .schedule_time_lock_proposal(&sol_integration.fixture.payer.pubkey(), &config_pda)
        .build();

    approve_ix_at_gateway(&mut sol_integration, &mut gmp_call_data).await;

    let res = sol_integration.fixture.send_tx(&[gmp_call_data.ix]).await;
    assert!(res.is_ok());

    // Put the proposal under operator management, acting here as Axelar governance
    // infrastructure.
    let meta = gmp_memo_metadata();
    let mut gmp_call_data = ix_builder
        .clone()
        .gmp_ix()
        .with_msg_metadata(meta.clone())
        .approve_operator_proposal(&sol_integration.fixture.payer.pubkey(), &config_pda)
        .build();
    approve_ix_at_gateway(&mut sol_integration, &mut gmp_call_data).await;
    let res = sol_integration.fixture.send_tx(&[gmp_call_data.ix]).await;
    assert!(res.is_ok());

    // Send the operator execute proposal instruction
    let ix = ix_builder
        .clone()
        .execute_operator_proposal(
            &sol_integration.fixture.payer.pubkey(),
            &config_pda,
            &operator.pubkey(),
        )
        .build();

    let res = sol_integration
        .fixture
        .send_tx_with_custom_signers(
            &[ix],
            &[operator, sol_integration.fixture.payer.insecure_clone()],
        )
        .await;

    assert!(res.is_ok());

    // Assert event was emitted
    let mut emitted_events = events(&res.clone().unwrap());
    assert_eq!(emitted_events.len(), 1);
    let expected_event = operator_proposal_executed_event(&ix_builder);
    let got_event: GovernanceEvent = emitted_events.pop().unwrap().parse().unwrap();
    assert_eq!(expected_event, got_event);
    assert_msg_present_in_logs(res.unwrap(), "Instruction: SendToGateway");

    // Ensure the proposal account is closed
    let proposal_account = sol_integration
        .try_get_account_no_checks(&ix_builder.proposal_pda())
        .await
        .unwrap();
    assert!(proposal_account.is_none());
    // Ensure the proposal marker account is closed
    let proposal_marker_account = sol_integration
        .try_get_account_no_checks(&ix_builder.proposal_operator_marker_pda())
        .await
        .unwrap();
    assert!(proposal_marker_account.is_none());
}

fn operator_proposal_executed_event(builder: &IxBuilder<ProposalRelated>) -> GovernanceEvent {
    GovernanceEvent::OperatorProposalExecuted {
        hash: builder.proposal_hash(),
        target_address: builder.proposal_target_address().to_bytes(),
        call_data: to_vec(&builder.proposal_call_data()).unwrap(),
        native_value: builder.proposal_u256_le_native_value(),
    }
}

#[tokio::test]
async fn test_non_previously_approved_operator_proposal_execution_fails() {
    // Get the operator key pair;
    let operator = operator_keypair();

    let (mut sol_integration, config_pda, _) = Box::pin(setup_programs()).await;

    let ix_builder = ix_builder_with_sample_proposal_data();

    // Send the proposal via GMP acting as Axelar governance infrastructure
    // Get default fixtures
    let meta = gmp_sample_metadata();
    let mut gmp_call_data = ix_builder
        .clone()
        .gmp_ix()
        .with_msg_metadata(meta.clone())
        .schedule_time_lock_proposal(&sol_integration.fixture.payer.pubkey(), &config_pda)
        .build();
    approve_ix_at_gateway(&mut sol_integration, &mut gmp_call_data).await;
    let res = sol_integration.fixture.send_tx(&[gmp_call_data.ix]).await;
    assert!(res.is_ok());

    //  HERE, WE MISS THE STEP OF SETTING THE PROPOSAL UNDER OPERATOR MANAGEMENT, so
    // execution should fail.

    // Send the operator execute proposal instruction
    let ix = ix_builder
        .clone()
        .execute_operator_proposal(
            &sol_integration.fixture.payer.pubkey(),
            &config_pda,
            &operator.pubkey(),
        )
        .build();

    let res = sol_integration
        .fixture
        .send_tx_with_custom_signers(
            &[ix],
            &[operator, sol_integration.fixture.payer.insecure_clone()],
        )
        .await;
    assert!(res.is_err());
    assert_msg_present_in_logs(
        res.err().unwrap(),
        "Operator has no approval rights for this proposal",
    );
}

#[tokio::test]
async fn test_only_operator_can_execute_ix() {
    // Get the operator key pair;
    let operator = Keypair::new(); // Incorrect operator keypair

    let mut fixture = TestFixture::new(ProgramTest::default()).await;

    deploy_governance_program(&mut fixture).await;

    // Setup gov module (initialize contract)
    let (config_pda, _) =
        init_contract_with_operator(&mut fixture, operator_keypair().pubkey().to_bytes())
            .await
            .unwrap();
    let ix_builder = ix_builder_with_sample_proposal_data();

    let ix = ix_builder
        .clone()
        .execute_operator_proposal(&fixture.payer.pubkey(), &config_pda, &operator.pubkey())
        .build();

    let res = fixture
        .send_tx_with_custom_signers(&[ix], &[operator, fixture.payer.insecure_clone()])
        .await;
    assert!(res.is_err());
    assert_msg_present_in_logs(
        res.err().unwrap(),
        "Operator account must sign the transaction",
    );
}

#[tokio::test]
async fn test_program_checks_proposal_pda_is_correctly_derived() {
    // Get the operator key pair;
    let (mut sol_integration, config_pda, _) = Box::pin(setup_programs()).await;

    let mut ix_builder = ix_builder_with_sample_proposal_data();
    let meta = gmp_sample_metadata();
    let mut gmp_call_data = ix_builder
        .clone()
        .gmp_ix()
        .with_msg_metadata(meta.clone())
        .schedule_time_lock_proposal(&sol_integration.fixture.payer.pubkey(), &config_pda)
        .build();
    approve_ix_at_gateway(&mut sol_integration, &mut gmp_call_data).await;
    let res = sol_integration.fixture.send_tx(&[gmp_call_data.ix]).await;
    assert!(res.is_ok());

    let meta = gmp_sample_metadata();
    let mut gmp_call_data = ix_builder
        .clone()
        .gmp_ix()
        .with_msg_metadata(meta.clone())
        .approve_operator_proposal(&sol_integration.fixture.payer.pubkey(), &config_pda)
        .build();
    approve_ix_at_gateway(&mut sol_integration, &mut gmp_call_data).await;
    let res = sol_integration.fixture.send_tx(&[gmp_call_data.ix]).await;
    assert!(res.is_ok());

    ix_builder.prop_target = Some([1_u8; 32].to_vec().try_into().unwrap());

    let operator = operator_keypair();
    let ix = ix_builder
        .execute_operator_proposal(
            &sol_integration.fixture.payer.pubkey(),
            &config_pda,
            &operator.pubkey(),
        )
        .build();

    let res = sol_integration
        .fixture
        .send_tx_with_custom_signers(
            &[ix],
            &[operator, sol_integration.fixture.payer.insecure_clone()],
        )
        .await;
    assert!(res.is_err());

    let meta = res.err().unwrap();

    assert!(meta
        .find_at_least_one_log(&[
            "Derived proposal PDA does not match provided one",
            "Provided seeds do not result in a valid address",
        ])
        .is_some());
}

#[tokio::test]
async fn test_program_checks_operator_pda_is_correctly_derived() {
    // Get the operator key pair;
    let (mut sol_integration, config_pda, _) = Box::pin(setup_programs()).await;

    let mut ix_builder = ix_builder_with_sample_proposal_data();
    let meta = gmp_sample_metadata();
    let mut gmp_call_data = ix_builder
        .clone()
        .gmp_ix()
        .with_msg_metadata(meta.clone())
        .schedule_time_lock_proposal(&sol_integration.fixture.payer.pubkey(), &config_pda)
        .build();
    approve_ix_at_gateway(&mut sol_integration, &mut gmp_call_data).await;
    let res = sol_integration.fixture.send_tx(&[gmp_call_data.ix]).await;
    assert!(res.is_ok());

    let meta = gmp_sample_metadata();
    let mut gmp_call_data = ix_builder
        .clone()
        .gmp_ix()
        .with_msg_metadata(meta.clone())
        .approve_operator_proposal(&sol_integration.fixture.payer.pubkey(), &config_pda)
        .build();
    approve_ix_at_gateway(&mut sol_integration, &mut gmp_call_data).await;
    let res = sol_integration.fixture.send_tx(&[gmp_call_data.ix]).await;
    assert!(res.is_ok());

    ix_builder.prop_operator_pda = Some(Pubkey::new_unique());

    let operator = operator_keypair();
    let ix = ix_builder
        .execute_operator_proposal(
            &sol_integration.fixture.payer.pubkey(),
            &config_pda,
            &operator.pubkey(),
        )
        .build();

    let res = sol_integration
        .fixture
        .send_tx_with_custom_signers(
            &[ix],
            &[operator, sol_integration.fixture.payer.insecure_clone()],
        )
        .await;
    assert!(res.is_err());
    assert_msg_present_in_logs(
        res.err().unwrap(),
        "Derived operator managed proposal PDA does not match provided one",
    );
}
