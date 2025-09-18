use solana_program::hash;

fn compute_discriminator(prefix: &str, name: &str) -> [u8; 8] {
    let mut out = [0u8; 8];
    out.copy_from_slice(&hash::hash([prefix, name].concat().as_bytes()).to_bytes()[..8]);
    out
}

pub fn compute_instruction_discriminator(name: &str) -> [u8; 8] {
    compute_discriminator("global:", name)
}

pub fn compute_account_discriminator(name: &str) -> [u8; 8] {
    compute_discriminator("account:", name)
}

pub fn prepend_discriminator(discriminator: [u8; 8], instruction_data: &[u8]) -> Vec<u8> {
    [discriminator.as_slice(), instruction_data].concat()
}
