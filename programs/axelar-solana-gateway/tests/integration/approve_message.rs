use core::str::FromStr;

use axelar_solana_encoding::hasher::SolanaSyscallHasher;
use axelar_solana_encoding::types::execute_data::{MerkleisedMessage, MerkleisedPayload};
use axelar_solana_encoding::types::messages::Messages;
use axelar_solana_encoding::types::payload::Payload;
use axelar_solana_encoding::LeafHash;
use axelar_solana_gateway::error::GatewayError;
use axelar_solana_gateway::instructions::approve_message;
use axelar_solana_gateway::processor::GatewayEvent;
use axelar_solana_gateway::state::incoming_message::{command_id, IncomingMessage, MessageStatus};
use axelar_solana_gateway::{get_incoming_message_pda, get_validate_message_signing_pda};
use axelar_solana_gateway_test_fixtures::gateway::{
    get_gateway_events, make_messages, make_verifier_set, GetGatewayError, ProgramInvocationState,
};
use axelar_solana_gateway_test_fixtures::SolanaAxelarIntegration;
use itertools::Itertools;
use pretty_assertions::assert_eq;
use solana_program_test::tokio;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;

#[tokio::test]
async fn successfully_approves_messages() {
    // Setup
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42, 42])
        .build()
        .setup()
        .await;
    let message_count = 10;
    let messages = make_messages(message_count);
    let payload = Payload::Messages(Messages(messages.clone()));
    let execute_data = metadata.construct_execute_data(&metadata.signers.clone(), payload);
    let verification_session_pda = metadata
        .init_payload_session_and_verify(&execute_data)
        .await
        .unwrap();
    let mut counter = 0;
    let MerkleisedPayload::NewMessages { messages } = execute_data.payload_items else {
        unreachable!()
    };
    for message_info in messages {
        let hash = message_info.leaf.message.hash::<SolanaSyscallHasher>();
        let command_id = command_id(
            &message_info.leaf.message.cc_id.chain,
            &message_info.leaf.message.cc_id.id,
        );
        let (incoming_message_pda, incoming_message_pda_bump) =
            get_incoming_message_pda(&command_id);

        let message = message_info.leaf.clone().message;
        let ix = approve_message(
            message_info,
            execute_data.payload_merkle_root,
            metadata.gateway_root_pda,
            metadata.payer.pubkey(),
            verification_session_pda,
            incoming_message_pda,
        )
        .unwrap();
        let tx = metadata.send_tx(&[ix]).await.unwrap();

        // Assert event
        let expected_event = axelar_solana_gateway::processor::MessageEvent {
            command_id,
            cc_id_chain: message.cc_id.chain.clone(),
            cc_id_id: message.cc_id.id.clone(),
            source_address: message.source_address.clone(),
            destination_address: Pubkey::from_str(&message.destination_address).unwrap(),
            payload_hash: message.payload_hash,
            destination_chain: message.destination_chain,
        };
        let emitted_events = get_gateway_events(&tx).pop().unwrap();
        let ProgramInvocationState::Succeeded(vec_events) = emitted_events else {
            panic!("unexpected event")
        };
        let [(_, GatewayEvent::MessageApproved(emitted_event))] = vec_events.as_slice() else {
            panic!("unexpected event")
        };
        assert_eq!(emitted_event, &expected_event);

        let (_, signing_pda_bump) =
            get_validate_message_signing_pda(expected_event.destination_address, command_id);

        // Assert PDA state for message approval
        let account = metadata.incoming_message(incoming_message_pda).await;
        let expected_message = IncomingMessage::new(
            incoming_message_pda_bump,
            signing_pda_bump,
            MessageStatus::approved(),
            hash,
            message.payload_hash,
        );

        assert_eq!(account, expected_message);
        counter += 1;
    }
    assert_eq!(counter, message_count);
}

#[tokio::test]
async fn fail_individual_approval_if_done_many_times() {
    // Setup
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42, 42])
        .build()
        .setup()
        .await;

    let messages_batch_one = make_messages(1);
    let messages_batch_two = {
        let mut new_messages = make_messages(1);
        new_messages.extend_from_slice(&messages_batch_one);
        new_messages
    };

    // approve the initial message batch
    let _m = metadata
        .sign_session_and_approve_messages(&metadata.signers.clone(), &messages_batch_one)
        .await
        .unwrap();

    // approve the second message batch
    let payload = Payload::Messages(Messages(messages_batch_two.clone()));
    let execute_data_batch_two =
        metadata.construct_execute_data(&metadata.signers.clone(), payload);
    let verification_session_pda = metadata
        .init_payload_session_and_verify(&execute_data_batch_two)
        .await
        .unwrap();
    let MerkleisedPayload::NewMessages {
        messages: merkle_messages_batch_two,
    } = execute_data_batch_two.payload_items
    else {
        unreachable!()
    };
    let mut events_counter = 0;
    let mut message_counter = 0;
    for message_info in merkle_messages_batch_two {
        let hash = message_info.leaf.message.hash::<SolanaSyscallHasher>();
        let tx = metadata
            .approve_message(
                execute_data_batch_two.payload_merkle_root,
                message_info.clone(),
                verification_session_pda,
            )
            .await;

        let tx = match tx {
            Ok(tx) => tx,
            Err(err) => {
                let gateway_error = err.get_gateway_error().unwrap();
                assert_eq!(gateway_error, GatewayError::MessageAlreadyInitialised);
                continue;
            }
        };
        message_counter += 1;

        let destination_address =
            Pubkey::from_str(&message_info.leaf.message.destination_address).unwrap();

        let emitted_events = get_gateway_events(&tx).pop().unwrap();
        let ProgramInvocationState::Succeeded(vec_events) = emitted_events else {
            panic!("unexpected event")
        };
        if let [(_, GatewayEvent::MessageApproved(_emitted_event))] = vec_events.as_slice() {
            events_counter += 1;
        };

        // Assert PDA state for message approval
        let command_id = command_id(
            &message_info.leaf.message.cc_id.chain,
            &message_info.leaf.message.cc_id.id,
        );
        let (incoming_message_pda, incoming_message_pda_bump) =
            get_incoming_message_pda(&command_id);
        let (_, signing_pda_bump) =
            get_validate_message_signing_pda(destination_address, command_id);

        let account = metadata.incoming_message(incoming_message_pda).await;
        let expected_message = IncomingMessage::new(
            incoming_message_pda_bump,
            signing_pda_bump,
            MessageStatus::approved(),
            hash,
            message_info.leaf.message.payload_hash,
        );
        assert_eq!(account, expected_message);
    }

    assert_eq!(
        events_counter,
        messages_batch_two.len() - messages_batch_one.len(),
        "expected new unique events in the second batch"
    );
    assert_eq!(
        message_counter,
        messages_batch_two.len() - messages_batch_one.len(),
        "expected only unique messages from second batch to be processed"
    );
}

// the same message can only be approved once, subsequent calls will fail
#[tokio::test]
async fn fail_approvals_many_times_same_batch() {
    // Setup
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42, 42])
        .build()
        .setup()
        .await;

    let messages = make_messages(2);

    // verify the signatures
    let payload = Payload::Messages(Messages(messages.clone()));
    let execute_data = metadata.construct_execute_data(&metadata.signers.clone(), payload);
    let verification_session_pda = metadata
        .init_payload_session_and_verify(&execute_data)
        .await
        .unwrap();

    // approve the messages initially
    let MerkleisedPayload::NewMessages { messages } = execute_data.payload_items.clone() else {
        unreachable!()
    };

    for message_info in messages {
        metadata
            .approve_message(
                execute_data.payload_merkle_root,
                message_info.clone(),
                verification_session_pda,
            )
            .await
            .unwrap();
    }

    // try to approve the messages again (will fail)
    let MerkleisedPayload::NewMessages { messages } = execute_data.payload_items.clone() else {
        unreachable!()
    };

    for message_info in messages {
        let tx = metadata
            .approve_message(
                execute_data.payload_merkle_root,
                message_info.clone(),
                verification_session_pda,
            )
            .await
            .unwrap_err();
        let gateway_error = tx.get_gateway_error().unwrap();
        assert_eq!(gateway_error, GatewayError::MessageAlreadyInitialised);
    }
}

// cannot approve a message from a different payload
#[tokio::test]
async fn fails_to_approve_message_not_in_payload() {
    // Setup
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42, 42])
        .build()
        .setup()
        .await;

    // Create a payload with a batch of messages
    let payload = Payload::Messages(Messages(make_messages(2)));
    let execute_data = metadata.construct_execute_data(&metadata.signers.clone(), payload);
    let MerkleisedPayload::NewMessages {
        messages: approved_messages,
    } = execute_data.payload_items.clone()
    else {
        unreachable!();
    };
    let payload_merkle_root = execute_data.payload_merkle_root;

    // Initialize and sign the payload session
    let verification_session_pda = metadata
        .init_payload_session_and_verify(&execute_data)
        .await
        .unwrap();

    // Create a fake message that is not part of the payload
    let fake_payload = Payload::Messages(Messages(make_messages(1)));
    let fake_execute_data =
        metadata.construct_execute_data(&metadata.signers.clone(), fake_payload);
    let MerkleisedPayload::NewMessages {
        messages: fake_messages,
    } = fake_execute_data.payload_items
    else {
        unreachable!();
    };
    let fake_payload_merkle_root = fake_execute_data.payload_merkle_root;

    let fm = || fake_messages.clone().into_iter();
    let fake_leaves = || fm().map(|x| x.leaf).collect_vec();
    let fake_proofs = || fm().map(|x| x.proof).collect_vec();
    let ap = || approved_messages.clone().into_iter();
    let valid_leaves = || ap().map(|x| x.leaf).collect_vec();
    let valid_proofs = || ap().map(|x| x.proof).collect_vec();
    for (merkle_root, leaves, proofs) in [
        (fake_payload_merkle_root, fake_leaves(), fake_proofs()),
        (fake_payload_merkle_root, fake_leaves(), valid_proofs()),
        (fake_payload_merkle_root, valid_leaves(), valid_proofs()),
        (payload_merkle_root, fake_leaves(), fake_proofs()),
        (payload_merkle_root, fake_leaves(), valid_proofs()),
        (payload_merkle_root, valid_leaves(), fake_proofs()),
    ] {
        for (leaf, proof) in leaves.into_iter().zip(proofs.into_iter()) {
            let new_message_info = MerkleisedMessage { leaf, proof };
            metadata
                .approve_message(merkle_root, new_message_info, verification_session_pda)
                .await
                .unwrap_err();
        }
    }
}

// cannot approve a message using verifier set payload hash
#[tokio::test]
async fn fails_to_approve_message_using_verifier_set_as_the_root() {
    // Setup
    let mut metadata = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![42, 42])
        .build()
        .setup()
        .await;

    // Create a payload with a batch of messages
    let new_verifier_set = make_verifier_set(&[500, 200], 1, metadata.domain_separator);
    let payload = Payload::NewVerifierSet(new_verifier_set.verifier_set());
    let execute_data = metadata.construct_execute_data(&metadata.signers.clone(), payload);

    // Initialize and sign the payload session
    let verification_session_pda = metadata
        .init_payload_session_and_verify(&execute_data)
        .await
        .unwrap();
    let MerkleisedPayload::VerifierSetRotation {
        new_verifier_set_merkle_root,
    } = execute_data.payload_items
    else {
        unreachable!();
    };

    // Create a fake message that is not part of the payload
    let fake_payload = Payload::Messages(Messages(make_messages(1)));
    let fake_execute_data =
        metadata.construct_execute_data(&metadata.signers.clone(), fake_payload);
    let MerkleisedPayload::NewMessages {
        messages: fake_messages,
    } = fake_execute_data.payload_items
    else {
        unreachable!();
    };
    let fake_payload_merkle_root = fake_execute_data.payload_merkle_root;

    let fm = || fake_messages.clone().into_iter();
    let fake_leaves = || fm().map(|x| x.leaf).collect_vec();
    let fake_proofs = || fm().map(|x| x.proof).collect_vec();

    // Create a fake message that is not part of the payload
    for (merkle_root, leaves, proofs) in [
        (fake_payload_merkle_root, fake_leaves(), fake_proofs()),
        (new_verifier_set_merkle_root, fake_leaves(), fake_proofs()),
    ] {
        for (leaf, proof) in leaves.into_iter().zip(proofs.into_iter()) {
            let new_message_info = MerkleisedMessage { leaf, proof };
            metadata
                .approve_message(merkle_root, new_message_info, verification_session_pda)
                .await
                .unwrap_err();
        }
    }
}
