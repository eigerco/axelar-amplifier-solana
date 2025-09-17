use borsh::{BorshDeserialize, BorshSerialize};
use event_cpi::{CpiEvent, Discriminator};
use event_cpi_macro::event;
use solana_program::pubkey::Pubkey;

/// Represents the event emitted when native gas is paid for a contract call.
#[event]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NativeGasPaidForContractCallEvent {
    /// The Gas service config PDA
    pub config_pda: Pubkey,
    /// Destination chain on the Axelar network
    pub destination_chain: String,
    /// Destination address on the Axelar network
    pub destination_address: String,
    /// The payload hash for the event we're paying for
    pub payload_hash: [u8; 32],
    /// The refund address
    pub refund_address: Pubkey,
    /// Extra parameters to be passed
    pub params: Vec<u8>,
    /// The amount of SOL to send
    pub gas_fee_amount: u64,
}

#[test]
fn test_discriminator() {
    let event = NativeGasPaidForContractCallEvent {
        config_pda: Pubkey::new_unique(),
        destination_chain: "chain".to_string(),
        destination_address: "address".to_string(),
        payload_hash: [0u8; 32],
        refund_address: Pubkey::new_unique(),
        params: vec![1, 2, 3],
        gas_fee_amount: 100,
    };

    println!(
        "Discriminator: {:?}",
        NativeGasPaidForContractCallEvent::DISCRIMINATOR
    );

    let data = event.data();
    assert_eq!(
        &data[0..8],
        NativeGasPaidForContractCallEvent::DISCRIMINATOR
    );
}
