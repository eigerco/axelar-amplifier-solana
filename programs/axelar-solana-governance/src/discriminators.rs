// Instruction discriminators
pub const TRANSFER_OPERATORSHIP: [u8; 8] = [17, 238, 86, 208, 233, 122, 195, 186];
pub const WITHDRAW_TOKENS: [u8; 8] = [2, 4, 225, 61, 19, 182, 106, 170];
pub const EXECUTE_PROPOSAL: [u8; 8] = [186, 60, 116, 133, 108, 128, 111, 28];
pub const EXECUTE_OPERATOR_PROPOSAL: [u8; 8] = [122, 19, 234, 108, 32, 92, 20, 7];
pub const INITIALIZE_CONFIG: [u8; 8] = [208, 127, 21, 1, 194, 190, 196, 70];
pub const UPDATE_CONFIG: [u8; 8] = [29, 158, 252, 191, 10, 83, 219, 99];
pub const PROCESS_GMP: [u8; 8] = [25, 158, 155, 125, 212, 65, 112, 20];

// PDA discriminators
pub const GOVERNANCE_CONFIG_PDA_DISCRIMINATOR: [u8; 8] =
    [0x51, 0x3f, 0x7c, 0x6b, 0xd2, 0x64, 0x91, 0x46];
pub const EXECUTABLE_PROPOSAL_PDA_DISCRIMINATOR: [u8; 8] =
    [0x98, 0x89, 0x4c, 0x2f, 0xd0, 0xb4, 0x61, 0xb7];
