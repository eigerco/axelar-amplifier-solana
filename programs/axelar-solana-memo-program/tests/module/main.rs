use axelar_solana_encoding::types::messages::Message;
use axelar_solana_gateway::{get_incoming_message_pda, state::incoming_message::command_id};
use axelar_solana_gateway_test_fixtures::{
    SolanaAxelarIntegration, SolanaAxelarIntegrationMetadata,
};
use solana_program_test::BanksTransactionResultWithMetadata;

mod initialize;
mod send_to_gateway;
mod validate_message;

pub async fn program_test() -> SolanaAxelarIntegrationMetadata {
    SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![555, 222])
        .programs_to_deploy(vec![(
            "axelar_solana_memo_program.so".into(),
            axelar_solana_memo_program::id(),
        )])
        .build()
        .setup()
        .await
}

// TODO Deduplicate
/// Call `execute` on an axelar-executable program
pub async fn execute_on_axelar_executable(
    metadata: &mut SolanaAxelarIntegrationMetadata,
    message: Message,
    raw_payload: &[u8],
) -> Result<BanksTransactionResultWithMetadata, BanksTransactionResultWithMetadata> {
    let message_payload_pda = metadata
        .upload_message_payload(&message, raw_payload)
        .await?;

    let (incoming_message_pda, _bump) =
        get_incoming_message_pda(&command_id(&message.cc_id.chain, &message.cc_id.id));
    let ix = axelar_executable::construct_axelar_executable_ix(
        &message,
        raw_payload,
        incoming_message_pda,
        message_payload_pda,
    )
    .unwrap();
    let execute_results = metadata.send_tx(&[ix]).await;

    // Close message payload and reclaim lamports
    metadata.close_message_payload(&message).await?;

    execute_results
}
