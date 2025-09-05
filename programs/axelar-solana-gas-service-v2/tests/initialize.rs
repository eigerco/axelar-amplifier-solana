// #![cfg(feature = "test-sbf")]

use axelar_solana_gas_service_v2::state::Config;
use {
    anchor_lang::{
        solana_program::instruction::Instruction, system_program, Discriminator, InstructionData,
        Space, ToAccountMetas,
    },
    mollusk_svm::Mollusk,
    solana_sdk::{account::Account, pubkey::Pubkey},
};

// TODO(v2) extract to a common test utils crate
// or set the env var differently
pub fn setup_mollusk(program_id: &Pubkey, program_name: &str) -> Mollusk {
    std::env::set_var("SBF_OUT_DIR", "../../target/deploy");
    Mollusk::new(program_id, program_name)
}

#[test]
// TODO(v2) improve the test and use mollusk checks
fn test_initialize_success() {
    let program_id = axelar_solana_gas_service_v2::id();
    let mollusk = setup_mollusk(&program_id, "axelar_solana_gas_service_v2");

    // Create test accounts
    let payer = Pubkey::new_unique();
    let operator = Pubkey::new_unique();

    // Derive the config PDA
    let (config_pda, bump) = Pubkey::find_program_address(&[Config::SEED_PREFIX], &program_id);

    let ix = Instruction {
        program_id,
        accounts: axelar_solana_gas_service_v2::accounts::Initialize {
            payer,
            operator,
            config_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: axelar_solana_gas_service_v2::instruction::Initialize {}.data(),
    };

    // Set up accounts - payer needs SOL for rent
    let accounts = vec![
        (payer, Account::new(1_000_000_000, 0, &system_program::ID)),
        (operator, Account::new(0, 0, &system_program::ID)),
        (config_pda, Account::new(0, 0, &system_program::ID)),
        (
            system_program::ID,
            Account {
                lamports: 1,
                data: vec![],
                owner: solana_sdk::native_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        ),
    ];

    // Process the instruction
    let result = mollusk.process_instruction(&ix, &accounts);
    // Verify success
    assert!(result.program_result.is_ok());

    // Verify the config PDA was created correctly
    let config_account = result
        .get_account(&config_pda)
        .expect("Config PDA should exist");
    assert_eq!(config_account.owner, program_id);

    // The account should have the correct size
    let expected_size = Config::DISCRIMINATOR.len() + Config::INIT_SPACE;
    assert_eq!(config_account.data.len(), expected_size);

    // Verify config data
    let config_data = &config_account.data[..];
    let operator_bytes = &config_data[0..32];
    let bump_byte = config_data[32];

    assert_eq!(operator_bytes, operator.to_bytes());
    assert_eq!(bump_byte, bump);
}
