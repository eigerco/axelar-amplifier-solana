use mollusk_svm::{program::keyed_account_for_system_program, result::Check};
use {
    anchor_lang::{
        prelude::ProgramError, solana_program::instruction::Instruction, system_program,
        InstructionData, ToAccountMetas,
    },
    solana_sdk::{account::Account, pubkey::Pubkey},
    solana_sdk_ids::bpf_loader_upgradeable,
};
mod initialize;
use initialize::{init_gas_service, setup_mollusk, setup_operator};

#[test]
fn test_add_native_gas() {
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

    let sender = Pubkey::new_unique();
    let sender_balance = 1_000_000_000u64; // 1 SOL
    let sender_account = Account::new(sender_balance, 0, &system_program::ID);

    let tx_hash = [0u8; 64];
    let log_index = 0u64;
    let gas_fee_amount = 300_000_000u64; // 0.3 SOL
    let refund_address = Pubkey::new_unique();

    let (event_authority, _bump) =
        Pubkey::find_program_address(&[b"__event_authority"], &program_id);
    let event_authority_account = Account::new(0, 0, &system_program::ID);

    let ix = Instruction {
        program_id,
        accounts: axelar_solana_gas_service_v2::accounts::AddNativeGas {
            sender,
            treasury,
            system_program: system_program::ID,
            // Event authority
            event_authority: event_authority,
            // The current program account
            program: program_id,
        }
        .to_account_metas(None),
        data: axelar_solana_gas_service_v2::instruction::AddNativeGas {
            tx_hash,
            log_index,
            gas_fee_amount,
            refund_address,
        }
        .data(),
    };

    let accounts = vec![
        (sender, sender_account.clone()),
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
        Check::account(&sender)
            .lamports(sender_balance - gas_fee_amount)
            .build(),
        // Balance added
        Check::account(&treasury)
            .lamports(treasury_pda.lamports + gas_fee_amount)
            .build(),
    ];

    mollusk.process_and_validate_instruction(&ix, &accounts, &checks);

    // TODO(v2) check for CPI event emission
}

#[test]
fn test_add_native_gas_fails_for_zero() {
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

    let sender = Pubkey::new_unique();
    let sender_balance = 1_000_000_000u64; // 1 SOL
    let sender_account = Account::new(sender_balance, 0, &system_program::ID);

    let tx_hash = [0u8; 64];
    let log_index = 0u64;
    let gas_fee_amount = 0u64; // 0 SOL
    let refund_address = Pubkey::new_unique();

    let (event_authority, _bump) =
        Pubkey::find_program_address(&[b"__event_authority"], &program_id);
    let event_authority_account = Account::new(0, 0, &system_program::ID);

    let ix = Instruction {
        program_id,
        accounts: axelar_solana_gas_service_v2::accounts::AddNativeGas {
            sender,
            treasury,
            system_program: system_program::ID,
            // Event authority
            event_authority: event_authority,
            // The current program account
            program: program_id,
        }
        .to_account_metas(None),
        data: axelar_solana_gas_service_v2::instruction::AddNativeGas {
            tx_hash,
            log_index,
            gas_fee_amount,
            refund_address,
        }
        .data(),
    };

    let accounts = vec![
        (sender, sender_account.clone()),
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
        Check::err(ProgramError::InvalidInstructionData),
        // Balance unchanged
        Check::account(&sender).lamports(sender_balance).build(),
        // Balance unchanged
        Check::account(&treasury)
            .lamports(treasury_pda.lamports)
            .build(),
    ];

    mollusk.process_and_validate_instruction(&ix, &accounts, &checks);
}

#[test]
fn test_add_native_gas_fails_insufficient_balance() {
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

    let sender = Pubkey::new_unique();
    let sender_balance = 300_000_000u64; // 0.3 SOL
    let sender_account = Account::new(sender_balance, 0, &system_program::ID);

    let tx_hash = [0u8; 64];
    let log_index = 0u64;
    let gas_fee_amount = 500_000_000u64; // 0.3 SOL
    let refund_address = Pubkey::new_unique();

    let (event_authority, _bump) =
        Pubkey::find_program_address(&[b"__event_authority"], &program_id);
    let event_authority_account = Account::new(0, 0, &system_program::ID);

    let ix = Instruction {
        program_id,
        accounts: axelar_solana_gas_service_v2::accounts::AddNativeGas {
            sender,
            treasury,
            system_program: system_program::ID,
            // Event authority
            event_authority: event_authority,
            // The current program account
            program: program_id,
        }
        .to_account_metas(None),
        data: axelar_solana_gas_service_v2::instruction::AddNativeGas {
            tx_hash,
            log_index,
            gas_fee_amount,
            refund_address,
        }
        .data(),
    };

    let accounts = vec![
        (sender, sender_account.clone()),
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
        // TODO(v2) figure out where does this custom error code come from
        // we should avoid magic numbers. This comes from system_program::transfer
        Check::err(ProgramError::Custom(1)),
        // Balance unchanged
        Check::account(&sender).lamports(sender_balance).build(),
        // Balance unchanged
        Check::account(&treasury)
            .lamports(treasury_pda.lamports)
            .build(),
    ];

    mollusk.process_and_validate_instruction(&ix, &accounts, &checks);
}
