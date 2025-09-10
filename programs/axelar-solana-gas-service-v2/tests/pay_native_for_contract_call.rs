use mollusk_svm::{program::keyed_account_for_system_program, result::Check};
use {
    anchor_lang::{
        solana_program::instruction::Instruction, system_program, InstructionData, ToAccountMetas,
    },
    solana_sdk::{account::Account, pubkey::Pubkey},
    solana_sdk_ids::bpf_loader_upgradeable,
};
mod initialize;
use initialize::{init_gas_service, setup_mollusk, setup_operator};

#[test]
fn test_pay_native_contract_call() {
    // Setup

    let program_id = axelar_solana_gas_service_v2::id();
    let mut mollusk = setup_mollusk(&program_id, "axelar_solana_gas_service_v2");

    let operator = Pubkey::new_unique();
    let operator_account = Account::new(1_000_000_000, 0, &system_program::ID);

    let (operator_pda, operator_pda_account) =
        setup_operator(&mut mollusk, operator, &operator_account);

    let (treasury, treasury_pda) = init_gas_service(
        &mollusk,
        operator,
        &operator_account,
        operator_pda,
        &operator_pda_account,
    );

    // Instruction

    let payer = Pubkey::new_unique();
    let payer_balance = 1_000_000_000u64; // 1 SOL
    let payer_account = Account::new(payer_balance, 0, &system_program::ID);

    let gas_fee_amount = 300_000_000u64; // 0.3 SOL
    let refund_address = Pubkey::new_unique();

    let (event_authority, _bump) =
        Pubkey::find_program_address(&[b"__event_authority"], &program_id);
    let event_authority_account = Account::new(0, 0, &system_program::ID);

    let ix = Instruction {
        program_id,
        accounts: axelar_solana_gas_service_v2::accounts::PayNativeForContractCall {
            payer,
            treasury,
            system_program: system_program::ID,
            // Event authority
            event_authority: event_authority,
            // The current program account
            program: program_id,
        }
        .to_account_metas(None),
        data: axelar_solana_gas_service_v2::instruction::PayNativeForContractCall {
            destination_chain: "chain".to_string(),
            destination_address: "address".to_string(),
            payload_hash: [0u8; 32],
            refund_address,
            params: vec![],
            gas_fee_amount,
        }
        .data(),
    };

    let accounts = vec![
        (payer, payer_account.clone()),
        (treasury, treasury_pda.clone()),
        keyed_account_for_system_program(),
        // Event authority
        (event_authority, event_authority_account),
        // Current program account (executable)
        (
            program_id,
            Account {
                lamports: 1,
                data: vec![],
                owner: bpf_loader_upgradeable::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
    ];

    // Checks

    let checks = vec![
        Check::success(),
        // Balance subtracted
        Check::account(&payer)
            .lamports(payer_balance - gas_fee_amount)
            .build(),
        // Balance added
        Check::account(&treasury)
            .lamports(treasury_pda.lamports + gas_fee_amount)
            .build(),
    ];

    mollusk.process_and_validate_instruction(&ix, &accounts, &checks);

    // TODO(v2) check for CPI event emission
}
