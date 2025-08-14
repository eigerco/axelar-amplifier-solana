use axelar_solana_encoding::types::messages::Message;
use axelar_solana_gateway::get_incoming_message_pda;
use axelar_solana_gateway::state::incoming_message::command_id;
use axelar_solana_gateway_test_fixtures::{
    SolanaAxelarIntegration, SolanaAxelarIntegrationMetadata,
};
use evm_contracts_test_suite::chain::TestBlockchain;
use evm_contracts_test_suite::evm_contracts_rs::contracts::axelar_amplifier_gateway;
use evm_contracts_test_suite::evm_weighted_signers::WeightedSigners;
use evm_contracts_test_suite::{get_domain_separator, ContractMiddleware};
use solana_program_test::BanksTransactionResultWithMetadata;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;

mod from_evm_to_solana;
mod from_solana_to_evm;

pub struct MemoProgramWrapper {
    pub solana_chain: SolanaAxelarIntegrationMetadata,
    pub counter_pda: Pubkey,
}

async fn axelar_solana_setup() -> MemoProgramWrapper {
    let mut solana_chain = SolanaAxelarIntegration::builder()
        .initial_signer_weights(vec![555, 222])
        .programs_to_deploy(vec![(
            "axelar_solana_memo_program.so".into(),
            axelar_solana_memo_program::id(),
        )])
        .build()
        .setup()
        .await;
    let (counter_pda, counter_bump) = axelar_solana_memo_program::get_counter_pda();
    solana_chain
        .fixture
        .send_tx(&[axelar_solana_memo_program::instruction::initialize(
            &solana_chain.fixture.payer.pubkey(),
            &(counter_pda, counter_bump),
        )
        .unwrap()])
        .await
        .unwrap();

    MemoProgramWrapper {
        solana_chain,
        counter_pda,
    }
}

async fn axelar_evm_setup() -> (
    TestBlockchain,
    evm_contracts_test_suite::EvmSigner,
    axelar_amplifier_gateway::AxelarAmplifierGateway<ContractMiddleware>,
    WeightedSigners,
    [u8; 32],
) {
    use evm_contracts_test_suite::ethers::signers::Signer;

    let evm_chain = evm_contracts_test_suite::chain::TestBlockchain::new();
    let alice = evm_chain.construct_provider_with_signer(0);
    let operators1 =
        evm_contracts_test_suite::evm_weighted_signers::create_operator_set(&evm_chain, 0..5);
    let operators2 =
        evm_contracts_test_suite::evm_weighted_signers::create_operator_set(&evm_chain, 5..9);
    let evm_gateway = alice
        .deploy_axelar_amplifier_gateway(
            &[operators1, operators2.clone()],
            alice.wallet.address(),
            alice.wallet.address(),
        )
        .await
        .unwrap();

    (
        evm_chain,
        alice,
        evm_gateway,
        operators2,
        get_domain_separator(),
    )
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
