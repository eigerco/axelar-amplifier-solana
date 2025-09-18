use solana_program::hash;

pub fn compute_instruction_discriminator(name: &str) -> [u8; 8] {
    hash::hash(format!("global:{name}").as_bytes()).to_bytes()[..8]
        .try_into()
        .unwrap()
}

pub fn compute_account_discriminator(name: &str) -> [u8; 8] {
    hash::hash(format!("account:{name}").as_bytes()).to_bytes()[..8]
        .try_into()
        .unwrap()
}

pub fn prepend_discriminator(discriminator: [u8; 8], instruction_data: &[u8]) -> Vec<u8> {
    [discriminator.as_slice(), instruction_data].concat()
}
