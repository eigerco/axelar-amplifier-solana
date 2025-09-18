// Instruction Discriminators
pub const INIT_CONFIG: [u8; 8] = [23, 235, 115, 232, 168, 96, 1, 231];
pub const TRANSFER_OPERATORSHIP: [u8; 8] = [17, 238, 86, 208, 233, 122, 195, 186];
pub const PAY_NATIVE_FOR_CONTRACT_CALL: [u8; 8] = [239, 120, 71, 21, 153, 26, 68, 249];
pub const ADD_NATIVE_GAS: [u8; 8] = [202, 252, 80, 193, 93, 140, 43, 236];
pub const COLLECT_NATIVE_FEES: [u8; 8] = [87, 79, 238, 90, 32, 194, 236, 58];
pub const REFUND_NATIVE_FEES: [u8; 8] = [28, 138, 70, 132, 164, 220, 42, 92];
pub const PAY_SPL_FOR_CONTRACT_CALL: [u8; 8] = [146, 234, 146, 22, 136, 172, 253, 13];
pub const ADD_SPL_GAS: [u8; 8] = [141, 229, 181, 108, 68, 251, 187, 81];
pub const COLLECT_SPL_FEES: [u8; 8] = [53, 127, 1, 71, 214, 130, 30, 212];
pub const REFUND_SPL_FEES: [u8; 8] = [68, 182, 235, 86, 189, 234, 222, 96];

// PDA discriminators
pub const CONFIG_PDA_DISCRIMINATOR: [u8; 8] = [0x9b, 0x0c, 0xaa, 0xe0, 0x1e, 0xfa, 0xcc, 0x82];
