use axelar_solana_encoding::types::messages::Messages;
use axelar_solana_encoding::types::payload::Payload;
use axelar_solana_gateway::get_gateway_root_config_pda;
use axelar_solana_gateway::state::signature_verification::SignatureVerification;
use axelar_solana_gateway_test_fixtures::gateway::{
    make_messages, make_verifier_set, random_bytes,
};
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

#[tokio::test]
async fn test_different_verifier_sets_produce_different_verification_session_pdas() {
    // Arrange: Set up test environment with two distinct verifier sets
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42])
        .build()
        .setup()
        .await;

    // Create two different verifier sets with distinct weights and epochs
    let verifier_set_a = make_verifier_set(&[500, 200], 1, metadata.domain_separator);
    let verifier_set_b = make_verifier_set(&[500, 23], 101, metadata.domain_separator);

    // Use the same message payload for both verifier sets to demonstrate PDA isolation
    let message_payload = Payload::Messages(Messages(make_messages(5)));

    // Construct execution data - this will generate different payload merkle roots
    // because the verifier set hash is included in the payload construction
    let execute_data_a = metadata.construct_execute_data(&verifier_set_a, message_payload.clone());
    let execute_data_b = metadata.construct_execute_data(&verifier_set_b, message_payload);

    // Verify that different verifier sets produce different payload hashes
    assert_ne!(
        execute_data_a.payload_merkle_root, execute_data_b.payload_merkle_root,
        "Different verifier sets must produce different payload merkle roots"
    );

    assert_ne!(
        execute_data_a.signing_verifier_set_merkle_root,
        execute_data_b.signing_verifier_set_merkle_root,
        "Verifier sets should have different hashes"
    );

    // Act & Assert: Initialize verification sessions for both verifier sets
    let mut verification_pdas = Vec::new();

    for execute_data in [execute_data_a, execute_data_b] {
        // Derive the expected PDA for this payload
        let (expected_pda, _) = axelar_solana_gateway::get_signature_verification_pda(
            &execute_data.payload_merkle_root,
        );
        verification_pdas.push(expected_pda);

        // Initialize the verification session
        let ix = axelar_solana_gateway::instructions::initialize_payload_verification_session(
            metadata.payer.pubkey(),
            metadata.gateway_root_pda,
            execute_data.payload_merkle_root,
            execute_data.signing_verifier_set_merkle_root,
        )
        .unwrap();

        let _tx_result = metadata.send_tx(&[ix]).await.unwrap();

        // Verify the verification session was created successfully
        let verification_session = metadata.signature_verification_session(expected_pda).await;

        // Ensure the session has the correct verifier set hash
        assert_eq!(
            verification_session
                .signature_verification
                .signing_verifier_set_hash,
            execute_data.signing_verifier_set_merkle_root,
            "Verification session must store the correct signing verifier set hash"
        );
    }

    // Assert: Verify that both verifier sets produced unique PDAs
    assert_ne!(
        verification_pdas[0], verification_pdas[1],
        "Different verifier sets must produce distinct verification session PDAs"
    );
}
