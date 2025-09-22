use borsh::BorshSerialize;

pub mod hash;

// https://github.com/solana-foundation/anchor/blob/18d0ca0ce9b78c03ef370406c6ba86e28e4591ab/lang/syn/src/codegen/program/common.rs#L5-L7
// Namespace for calculating instruction sighash signatures for any instruction
// not affecting program state.
pub const SIGHASH_GLOBAL_NAMESPACE: &str = "global";

// We don't technically use sighash, because the input arguments aren't given.
// Rust doesn't have method overloading so no need to use the arguments.
// However, we do namespace methods in the preeimage so that we can use
// different traits with the same method name.
pub fn sighash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{namespace}:{name}");

    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(&hash::hash(preimage.as_bytes()).to_bytes()[..8]);
    sighash
}

/// Unique identifier for a type.
///
/// This is not a trait you should derive manually, as various Anchor macros already derive it
/// internally.
///
/// Prior to Anchor v0.31, discriminators were always 8 bytes in size. However, starting with Anchor
/// v0.31, it is possible to override the default discriminators, and discriminator length is no
/// longer fixed, which means this trait can also be implemented for non-Anchor programs.
///
/// It's important that the discriminator is always unique for the type you're implementing it
/// for. While the discriminator can be at any length (including zero), the IDL generation does not
/// currently allow empty discriminators for safety and convenience reasons. However, the trait
/// definition still allows empty discriminators because some non-Anchor programs, e.g. the SPL
/// Token program, don't have account discriminators. In that case, safety checks should never
/// depend on the discriminator.
pub trait Discriminator {
    /// Discriminator slice.
    ///
    /// See [`Discriminator`] trait documentation for more information.
    const DISCRIMINATOR: &'static [u8];
}

/// Calculates the data for an instruction invocation, where the data is
/// `Discriminator + BorshSerialize(args)`. `args` is a borsh serialized
/// struct of named fields for each argument given to an instruction.
pub trait InstructionData: Discriminator + BorshSerialize {
    fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(256);
        data.extend_from_slice(Self::DISCRIMINATOR);
        self.serialize(&mut data).unwrap();
        data
    }

    /// Clears `data` and writes instruction data to it.
    ///
    /// We use a `Vec<u8>`` here because of the additional flexibility of re-allocation (only if
    /// necessary), and becausex the data field in `Instruction` expects a `Vec<u8>`.
    fn write_to(&self, mut data: &mut Vec<u8>) {
        data.clear();
        data.extend_from_slice(Self::DISCRIMINATOR);
        self.serialize(&mut data).unwrap();
    }
}
