//! State module contains data structures that keep state within the ITS
//! program.

use core::any::type_name;
use core::mem::size_of;

use borsh::{BorshDeserialize, BorshSerialize};
use program_utils::pda::BorshPda;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{Pack, Sealed};

/// Signed PDA to prove that ITS called an executable indeed. Only stores it's bump.
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

impl BorshPda for InterchainTransferExecute {}
