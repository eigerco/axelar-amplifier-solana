#![warn(missing_docs, unreachable_pub)]
#![deny(unused_must_use, rust_2018_idioms)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

//! Simple memo program example for the Axelar Gateway on Solana

mod entrypoint;
pub mod instruction;
pub mod processor;
pub mod state;
pub use solana_program;
use solana_program::pubkey::Pubkey;
use state::Counter;

solana_program::declare_id!("mem7LhKWbKydCPk1TwNzeCvVSpoVx2mqxNuvjGgWAbG");

/// Derives interchain token service root PDA
pub(crate) fn get_counter_pda_internal(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[], program_id)
}

/// Derives interchain token service root PDA
pub fn get_counter_pda() -> (Pubkey, u8) {
    get_counter_pda_internal(&crate::ID)
}

/// Assert counter PDA seeds
fn assert_counter_pda_seeds(counter_account: &Counter, counter_pda: &Pubkey) {
    let derived = Pubkey::create_program_address(&[&[counter_account.bump]], &crate::ID)
        .expect("failed to derive PDA");
    assert_eq!(&derived, counter_pda, "invalid pda for memo counter");
}
