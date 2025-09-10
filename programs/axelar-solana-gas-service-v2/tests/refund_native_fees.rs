use mollusk_svm::result::Check;
use solana_sdk::account::WritableAccount;
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
fn test_refund_native_fees() {
    // Setup

    let program_id = axelar_solana_gas_service_v2::id();
    let mut mollusk = setup_mollusk(&program_id, "axelar_solana_gas_service_v2");

    let operator = Pubkey::new_unique();
    let operator_account = Account::new(1_000_000_000, 0, &system_program::ID);

    let (operator_pda, operator_pda_account) =
        setup_operator(&mut mollusk, operator, &operator_account);

    let (treasury, mut treasury_pda) = init_gas_service(
        &mollusk,
        operator,
        &operator_account,
        operator_pda,
        &operator_pda_account,
    );

    let treasury_balance = 10_000_000_000u64; // 10 SOL
    treasury_pda
        .checked_add_lamports(treasury_balance)
        .expect("Failed to add lamports to treasury");

    // Instruction

    let receiver = Pubkey::new_unique();
    let receiver_balance = 1_000_000_000u64; // 1 SOL
    let receiver_account = Account::new(receiver_balance, 0, &system_program::ID);

    let tx_hash = [0u8; 64];
    let log_index = 0u64;
    let fees = 500_000_000u64; // 0.5 SOL

    let (event_authority, _bump) =
        Pubkey::find_program_address(&[b"__event_authority"], &program_id);
    let event_authority_account = Account::new(0, 0, &system_program::ID);

    let ix = Instruction {
        program_id,
        accounts: axelar_solana_gas_service_v2::accounts::RefundNativeFees {
            operator,
            operator_pda,
            receiver,
            treasury,
            // Event authority
            event_authority: event_authority,
            // The current program account
            program: program_id,
        }
        .to_account_metas(None),
        data: axelar_solana_gas_service_v2::instruction::RefundNativeFees {
            tx_hash,
            log_index,
            fees,
        }
        .data(),
    };

    let accounts = vec![
        (operator, operator_account.clone()),
        (operator_pda, operator_pda_account.clone()),
        (receiver, receiver_account.clone()),
        (treasury, treasury_pda.clone()),
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
        // Balance added
        Check::account(&receiver)
            .lamports(receiver_balance + fees)
            .build(),
        // Balance subtracted
        Check::account(&treasury)
            .lamports(treasury_pda.lamports - fees)
            .build(),
    ];

    mollusk.process_and_validate_instruction(&ix, &accounts, &checks);

    // TODO(v2) check for CPI event emission
}
