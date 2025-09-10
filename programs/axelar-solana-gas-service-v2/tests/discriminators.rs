use anchor_lang::prelude::*;
use anchor_spl::token::spl_token;
use axelar_solana_gas_service::instructions::*;
use axelar_solana_gas_service_v2::{instruction, GasServiceDiscriminators};

macro_rules! test_discriminator {
    ($name:expr, $v1_ix:expr, $expected:expr, $v2_discriminator:expr) => {
        assert_eq!(&$v1_ix.data[0..$expected.len()], $expected);
        assert_eq!($v2_discriminator, $expected);
        println!("✓ {}: {:?}", $name, &$v1_ix.data[0..$expected.len()]);
    };
}

#[test]
fn test_discriminators_backwards_compatible() {
    let payer = Pubkey::new_unique();
    let operator = Pubkey::new_unique();
    let mint = Pubkey::new_unique();
    let sender_ata = Pubkey::new_unique();
    let token_program_id = spl_token::ID;

    // Simple instructions (1 byte discriminators)
    test_discriminator!(
        "Initialize",
        init_config(&payer, &operator).unwrap(),
        GasServiceDiscriminators::INITIALIZE,
        instruction::Initialize::DISCRIMINATOR
    );

    // Native Token instructions (2 byte discriminators)
    test_discriminator!(
        "Native PayForContractCall",
        pay_native_for_contract_call_instruction(
            &payer,
            "ethereum".to_string(),
            "0x123".to_string(),
            [0u8; 32],
            Pubkey::default(),
            vec![],
            1000,
        )
        .unwrap(),
        GasServiceDiscriminators::NATIVE_PAY_FOR_CONTRACT_CALL,
        instruction::PayNativeForContractCall::DISCRIMINATOR
    );

    test_discriminator!(
        "Native AddGas",
        add_native_gas_instruction(&payer, [0u8; 64], 0, 500, Pubkey::default()).unwrap(),
        GasServiceDiscriminators::NATIVE_ADD_GAS,
        instruction::AddNativeGas::DISCRIMINATOR
    );

    test_discriminator!(
        "Native CollectFees",
        collect_native_fees_instruction(&operator, &payer, 100).unwrap(),
        GasServiceDiscriminators::NATIVE_COLLECT_FEES,
        instruction::CollectNativeFees::DISCRIMINATOR
    );

    test_discriminator!(
        "Native Refund",
        refund_native_fees_instruction(&operator, &payer, [1u8; 64], 1, 200).unwrap(),
        GasServiceDiscriminators::NATIVE_REFUND,
        instruction::RefundNativeFees::DISCRIMINATOR
    );

    // SPL Token instructions (2 byte discriminators)
    test_discriminator!(
        "SPL PayForContractCall",
        pay_spl_for_contract_call_instruction(
            &payer,
            &sender_ata,
            &mint,
            &token_program_id,
            "ethereum".to_string(),
            "0x456".to_string(),
            [1u8; 32],
            Pubkey::default(),
            vec![1, 2, 3],
            2000,
            &[],
            6,
        )
        .unwrap(),
        GasServiceDiscriminators::SPL_PAY_FOR_CONTRACT_CALL,
        instruction::PaySplForContractCall::DISCRIMINATOR
    );

    test_discriminator!(
        "SPL AddGas",
        add_spl_gas_instruction(
            &payer,
            &sender_ata,
            &mint,
            &token_program_id,
            &[],
            [2u8; 64],
            2,
            1500,
            Pubkey::default(),
            6,
        )
        .unwrap(),
        GasServiceDiscriminators::SPL_ADD_GAS,
        instruction::AddSplGas::DISCRIMINATOR
    );

    test_discriminator!(
        "SPL CollectFees",
        collect_spl_fees_instruction(&operator, &token_program_id, &mint, &payer, 300, 6).unwrap(),
        GasServiceDiscriminators::SPL_COLLECT_FEES,
        instruction::CollectSplFees::DISCRIMINATOR
    );

    test_discriminator!(
        "SPL Refund",
        refund_spl_fees_instruction(
            &operator,
            &token_program_id,
            &mint,
            &payer,
            [3u8; 64],
            3,
            400,
            6,
        )
        .unwrap(),
        GasServiceDiscriminators::SPL_REFUND,
        instruction::RefundSplFees::DISCRIMINATOR
    );

    println!("🎉 All discriminators validated using helper functions!");
}
