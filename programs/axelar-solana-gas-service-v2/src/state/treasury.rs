use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, PartialEq, Eq, Debug)]
pub struct Treasury {
    /// The bump seed used to derive the PDA, ensuring the address is valid.
    pub bump: u8,
}

impl Treasury {
    pub const SEED_PREFIX: &'static [u8] = b"treasury";
}
