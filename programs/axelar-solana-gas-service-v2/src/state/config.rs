use anchor_lang::prelude::*;

/// Empty discriminator to keep backwards compatibility with v1
#[account(zero_copy, discriminator = &[])]
#[derive(InitSpace, PartialEq, Eq, Debug)]
pub struct Config {
    /// Operator with permission to give refunds & withdraw funds
    pub operator: Pubkey,
    /// The bump seed used to derive the PDA, ensuring the address is valid.
    pub bump: u8,
}

impl Config {
    pub const SEED_PREFIX: &'static [u8; 11] = b"gas-service";
}
