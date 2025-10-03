//! Program state definitions

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::{IsInitialized, Pack, Sealed};

/// Program state account - simple counter for testing
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Counter {
    /// Counter value
    pub counter: u64,
    /// PDA bump seed
    pub bump: u8,
}

impl Sealed for Counter {}

impl IsInitialized for Counter {
    fn is_initialized(&self) -> bool {
        // Always initialized if account data exists
        true
    }
}

impl Pack for Counter {
    const LEN: usize = 8 + 1; // u64 + u8

    fn pack_into_slice(&self, output: &mut [u8]) {
        let encoded = borsh::to_vec(self).expect("Failed to serialize Counter");
        if output.len() < encoded.len() {
            panic!("Output slice too small");
        }
        output[..encoded.len()].copy_from_slice(&encoded);
    }

    fn unpack_from_slice(
        input: &[u8],
    ) -> Result<Self, solana_program::program_error::ProgramError> {
        borsh::from_slice(input)
            .map_err(|_| solana_program::program_error::ProgramError::InvalidAccountData)
    }
}
