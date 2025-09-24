#![cfg(test)]

use anchor_discriminators_macros::InstructionDiscriminator;
use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;

solana_program::declare_id!("gtwi5T9x6rTWPtuuz6DA7ia1VmH8bdazm9QfDdi6DVp");

#[derive(Debug, PartialEq, Eq, InstructionDiscriminator)]
pub enum GasServiceInstruction {
    Initialize,

    TransferOperatorship,

    NativeAddGas {
        tx_hash: [u8; 64],
        log_index: u64,
        gas_fee_amount: u64,
        decimals: u8,
        refund_address: Pubkey,
    },
}

#[test]
fn test_discriminator() {
    let init = GasServiceInstruction::Initialize;
    assert_eq!(
        &[175, 175, 109, 31, 13, 152, 155, 237],
        init.discriminator()
    );
}

#[test]
fn test_serialization() {
    let init = GasServiceInstruction::Initialize;
    let init_data = borsh::to_vec(&init).unwrap();
    assert_eq!(init_data.len(), 8);

    let transfer = GasServiceInstruction::TransferOperatorship;
    // This was computed with Anchor, this ensures compatibility.
    assert_eq!(
        &[17, 238, 86, 208, 233, 122, 195, 186],
        transfer.discriminator()
    );

    let add_gas = GasServiceInstruction::NativeAddGas {
        tx_hash: [0u8; 64],
        log_index: 0,
        gas_fee_amount: 100,
        decimals: 9,
        refund_address: Pubkey::new_unique(),
    };
    let add_gas_data = borsh::to_vec(&add_gas).unwrap();

    assert_eq!(&add_gas_data[..8], add_gas.discriminator());

    let add_gas_deser = GasServiceInstruction::try_from_slice(&add_gas_data).unwrap();
    assert_eq!(add_gas, add_gas_deser);
}
