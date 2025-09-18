// Instruction discriminators
pub const APPROVE_MESSAGE: [u8; 8] = [65, 154, 132, 135, 105, 5, 173, 21];
pub const ROTATE_SIGNERS: [u8; 8] = [122, 196, 231, 159, 163, 24, 207, 166];
pub const CALL_CONTRACT: [u8; 8] = [177, 150, 85, 130, 129, 92, 188, 211];
pub const INITIALIZE_CONFIG: [u8; 8] = [208, 127, 21, 1, 194, 190, 196, 70];
pub const INITIALIZE_PAYLOAD_VERIFICATION_SESSION: [u8; 8] = [136, 201, 241, 74, 8, 237, 63, 231];
pub const VERIFY_SIGNATURE: [u8; 8] = [91, 139, 24, 69, 251, 162, 245, 112];
pub const VALIDATE_MESSAGE: [u8; 8] = [237, 229, 200, 193, 7, 229, 212, 127];
pub const INITIALIZE_MESSAGE_PAYLOAD: [u8; 8] = [153, 242, 239, 43, 32, 226, 223, 110];
pub const WRITE_MESSAGE_PAYLOAD: [u8; 8] = [226, 202, 39, 173, 218, 248, 154, 54];
pub const COMMIT_MESSAGE_PAYLOAD: [u8; 8] = [106, 121, 76, 190, 254, 20, 146, 48];
pub const CLOSE_MESSAGE_PAYLOAD: [u8; 8] = [156, 62, 134, 162, 77, 226, 201, 222];
pub const TRANSFER_OPERATORSHIP: [u8; 8] = [17, 238, 86, 208, 233, 122, 195, 186];

// PDA discriminators
pub const CONFIG_PDA_DISCRIMINATOR: [u8; 8] = [0x5b, 0xf7, 0x42, 0x1b, 0x18, 0x01, 0x30, 0xb0];
pub const VERIFIER_SET_TRACKER_PDA_DISCRIMINATOR: [u8; 8] =
    [0x29, 0x08, 0xa3, 0x9d, 0xe5, 0xe9, 0x14, 0xb5];
pub const INCOMING_MESSAGE_PDA_DISCRIMINATOR: [u8; 8] =
    [0x1e, 0x90, 0x7d, 0x6f, 0xd3, 0xdf, 0x5b, 0xaa];
pub const VERIFICATION_SESSION_ACCOUNT_PDA_DISCRIMINATOR: [u8; 8] =
    [0x4b, 0xdf, 0x18, 0x11, 0x40, 0x23, 0xb3, 0xd1];
