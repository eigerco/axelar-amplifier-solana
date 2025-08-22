//! State module contains data structures that keep state within the ITS
//! program.

use core::any::type_name;
use core::mem::size_of;

use borsh::{BorshDeserialize, BorshSerialize};
use program_utils::pda::BorshPda;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{Pack, Sealed};

/// Signed PDA to prove that ITS called an executable indeed. Only stores it's bumb.
#[derive(Debug, Eq, PartialEq, Clone, BorshSerialize, BorshDeserialize)]
pub struct InterchainTransferExecute {
    /// The interchain transfer execute PDA bump seed.
    pub bump: u8,
}

impl InterchainTransferExecute {
    /// Creates a new `TokenManager` struct.
    #[must_use]
    pub const fn new(bump: u8) -> Self {
        Self { bump }
    }
}

impl Pack for InterchainTransferExecute {
    const LEN: usize = size_of::<u8>();

    #[allow(clippy::unwrap_used)]
    fn pack_into_slice(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap();
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, solana_program::program_error::ProgramError> {
        let mut mut_src: &[u8] = src;
        Self::deserialize(&mut mut_src).map_err(|err| {
            msg!(
                "Error: failed to deserialize account as {}: {}",
                type_name::<Self>(),
                err
            );
            ProgramError::InvalidAccountData
        })
    }
}

impl Sealed for InterchainTransferExecute {}
impl BorshPda for InterchainTransferExecute {}
