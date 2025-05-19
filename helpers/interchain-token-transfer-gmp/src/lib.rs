use std::borrow::Cow;

pub use alloy_primitives;
use alloy_sol_types::{sol, SolValue};

/// The messages going through the Axelar Network between
/// InterchainTokenServices need to have a consistent format to be
/// understood properly. We chose to use abi encoding because it is easy to
/// use in EVM chains, which are at the front and center of programmable
/// blockchains, and because it is easy to implement in other ecosystems
/// which tend to be more gas efficient.
#[derive(Clone, Debug, PartialEq)]
pub enum GMPPayload {
    InterchainTransfer(InterchainTransfer),
    DeployInterchainToken(DeployInterchainToken),
    SendToHub(SendToHub),
    ReceiveFromHub(ReceiveFromHub),
    LinkToken(LinkToken),
    RegisterTokenMetadata(RegisterTokenMetadata),
}

sol! {
    /// This message has the following data encoded and should only be sent
    /// after the proper tokens have been procured by the service. It should
    /// result in the proper funds being transferred to the user at the
    /// destination chain.
    #[derive(Debug, PartialEq)]
    #[repr(C)]
    struct InterchainTransfer {
        /// Will always have a value of 0
        uint256 selector;
        /// The interchainTokenId of the token being transferred
        bytes32 token_id;
        /// The address of the sender, encoded as bytes to account for different chain
        /// architectures
        bytes source_address;
        /// The address of the recipient, encoded as bytes as well
        bytes destination_address;
        /// The amount of token being send, not accounting for decimals (1 ETH would be 1018)
        uint256 amount;
        /// Either empty, for just a transfer, or any data to be passed to the destination address
        /// as a contract call
        bytes data;
    }

    /// This message has the following data encoded and should only be sent
    /// after the interchainTokenId has been properly generated (a user should
    /// not be able to claim just any interchainTokenId)
    #[derive(Debug, PartialEq)]
    #[repr(C)]
    struct DeployInterchainToken {
        uint256 selector;
        /// The interchainTokenId of the token being deployed
        bytes32 token_id;
        /// The name for the token
        string name;
        /// The symbol for the token
        string symbol;
        /// The decimals for the token
        uint8 decimals;
        /// An address on the destination chain that can mint/burn the deployed
        /// token on the destination chain, empty for no minter
        bytes minter;
    }

    /// This message is used to route an ITS message via the ITS Hub. The ITS Hub applies certain
    /// security checks, and then routes it to the true destination chain. This mode is enabled if the
    /// trusted address corresponding to the destination chain is set to the ITS Hub identifier.
    #[derive(Debug, PartialEq)]
    #[repr(C)]
    struct SendToHub {
        /// Should always have a value of 3
        uint256 selector;

        /// The true destination chain for the ITS call
        string destination_chain;

        /// The actual ITS message that's being routed through ITS Hub
        bytes payload;
    }

    /// This message is used to receive an ITS message from the ITS Hub. The ITS Hub applies
    /// certain security checks, and then routes it to the ITS contract. The message is accepted if the
    /// trusted address corresponding to the original source chain is set to the ITS Hub identifier.
    #[derive(Debug, PartialEq)]
    #[repr(C)]
    struct ReceiveFromHub {
        /// Will always have a value of 4
        uint256 selector;

        /// The original source chain for the ITS call
        string source_chain;

        /// The actual ITS message that's being routed through ITS Hub
        bytes payload;
    }

    #[derive(Debug, PartialEq)]
    #[repr(C)]
    struct LinkToken {
        /// Will always have a value of 5
        uint256 selector;
        /// The token_id associated with the token being linked
        bytes32 token_id;
        /// The type of the token manager to use to send and receive tokens
        uint256 token_manager_type;
        /// The token address in the source chain
        bytes source_token_address;
        /// The token address in the destination chain
        bytes destination_token_address;
        /// Additional parameters to use to link the token. Currently it's just the address of the
        /// operator
        bytes link_params;
    }

    #[derive(Debug, PartialEq)]
    #[repr(C)]
    struct RegisterTokenMetadata {
        /// Will always have a value of 6
        uint256 selector;
        /// The token address
        bytes token_address;
        /// The number of decimals for the token
        uint8 decimals;
    }


}

impl InterchainTransfer {
    pub const MESSAGE_TYPE_ID: u8 = 0;
}

impl DeployInterchainToken {
    pub const MESSAGE_TYPE_ID: u8 = 1;
}

impl SendToHub {
    pub const MESSAGE_TYPE_ID: u8 = 3;
}

impl ReceiveFromHub {
    pub const MESSAGE_TYPE_ID: u8 = 4;
}

impl LinkToken {
    pub const MESSAGE_TYPE_ID: u8 = 5;
}

impl RegisterTokenMetadata {
    pub const MESSAGE_TYPE_ID: u8 = 6;
}

impl GMPPayload {
    pub fn decode(bytes: &[u8]) -> Result<Self, alloy_sol_types::Error> {
        let variant = alloy_primitives::U256::abi_decode(&bytes[0..32], true)?;

        match variant.byte(0) {
            InterchainTransfer::MESSAGE_TYPE_ID => Ok(GMPPayload::InterchainTransfer(
                InterchainTransfer::abi_decode_params(bytes, true)?,
            )),
            DeployInterchainToken::MESSAGE_TYPE_ID => Ok(GMPPayload::DeployInterchainToken(
                DeployInterchainToken::abi_decode_params(bytes, true)?,
            )),
            SendToHub::MESSAGE_TYPE_ID => Ok(GMPPayload::SendToHub(SendToHub::abi_decode_params(
                bytes, true,
            )?)),
            ReceiveFromHub::MESSAGE_TYPE_ID => Ok(GMPPayload::ReceiveFromHub(
                ReceiveFromHub::abi_decode_params(bytes, true)?,
            )),
            RegisterTokenMetadata::MESSAGE_TYPE_ID => Ok(GMPPayload::RegisterTokenMetadata(
                RegisterTokenMetadata::abi_decode_params(bytes, true)?,
            )),
            LinkToken::MESSAGE_TYPE_ID => Ok(GMPPayload::LinkToken(LinkToken::abi_decode_params(
                bytes, true,
            )?)),
            _ => Err(alloy_sol_types::Error::custom(
                "Invalid selector for InterchainTokenService message",
            )),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            GMPPayload::InterchainTransfer(data) => data.abi_encode_params(),
            GMPPayload::DeployInterchainToken(data) => data.abi_encode_params(),
            GMPPayload::SendToHub(data) => data.abi_encode_params(),
            GMPPayload::ReceiveFromHub(data) => data.abi_encode_params(),
            GMPPayload::LinkToken(data) => data.abi_encode_params(),
            GMPPayload::RegisterTokenMetadata(data) => data.abi_encode_params(),
        }
    }

    pub fn token_id(&self) -> Result<[u8; 32], alloy_sol_types::Error> {
        match self {
            GMPPayload::InterchainTransfer(data) => Ok(*data.token_id),
            GMPPayload::DeployInterchainToken(data) => Ok(*data.token_id),
            GMPPayload::SendToHub(inner) => GMPPayload::decode(&inner.payload)?.token_id(),
            GMPPayload::ReceiveFromHub(inner) => GMPPayload::decode(&inner.payload)?.token_id(),
            GMPPayload::LinkToken(data) => Ok(*data.token_id),
            GMPPayload::RegisterTokenMetadata(_) => Err(alloy_sol_types::Error::Other(
                Cow::Borrowed("RegisterTokenMetadata does not have a token_id"),
            )),
        }
    }
}

impl From<InterchainTransfer> for GMPPayload {
    fn from(data: InterchainTransfer) -> Self {
        GMPPayload::InterchainTransfer(data)
    }
}

impl From<DeployInterchainToken> for GMPPayload {
    fn from(data: DeployInterchainToken) -> Self {
        GMPPayload::DeployInterchainToken(data)
    }
}

#[cfg(test)]
mod tests {

    use alloy_primitives::U256;

    use super::*;

    #[test]
    fn u256_into_u8() {
        let u256 = U256::from(42);
        let byte = u256.byte(0);
        assert_eq!(byte, 42);
    }

    /// fixture from https://github.com/axelarnetwork/interchain-token-service/blob/0977738a1d7df5551cb3bd2e18f13c0e09944ff2/test/InterchainTokenService.js
    /// [ 0,
    ///   '0xcccdb55f29bb017269049e59732c01ac41239e7b61e8a83be5c0ae1143ed8064',
    ///   '0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266',
    ///   '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
    ///   1234,
    ///   '0x'
    /// ]
    const INTERCHAIN_TRANSFER: &str = "0000000000000000000000000000000000000000000000000000000000000000cccdb55f29bb017269049e59732c01ac41239e7b61e8a83be5c0ae1143ed806400000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000004d200000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000014f39fd6e51aad88f6f4ce6ab8827279cfffb922660000000000000000000000000000000000000000000000000000000000000000000000000000000000000014f39fd6e51aad88f6f4ce6ab8827279cfffb922660000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn interchain_transfer_decode() {
        // Setup
        let gmp = GMPPayload::decode(&hex::decode(INTERCHAIN_TRANSFER).unwrap()).unwrap();

        // Action
        let GMPPayload::InterchainTransfer(data) = gmp else {
            panic!("wrong variant");
        };

        // Assert
        assert_eq!(
            hex::encode(data.token_id.abi_encode()),
            "cccdb55f29bb017269049e59732c01ac41239e7b61e8a83be5c0ae1143ed8064"
        );
        assert_eq!(
            data.source_address,
            hex::decode("f39fd6e51aad88f6f4ce6ab8827279cfffb92266").unwrap()
        );
        assert_eq!(
            data.destination_address,
            hex::decode("f39fd6e51aad88f6f4ce6ab8827279cfffb92266").unwrap()
        );
        assert_eq!(data.amount, U256::from(1234));
        assert_eq!(data.data, Vec::<u8>::new());
    }

    #[test]
    fn interchain_transfer_encode() {
        assert_eq!(
            hex::encode(
                GMPPayload::decode(&hex::decode(INTERCHAIN_TRANSFER).unwrap())
                    .unwrap()
                    .encode()
            ),
            INTERCHAIN_TRANSFER,
            "encode-decode should be idempotent"
        );
    }

    /// fixture from https://github.com/axelarnetwork/interchain-token-service/blob/0977738a1d7df5551cb3bd2e18f13c0e09944ff2/test/InterchainTokenService.js
    /// [
    ///   1,
    ///   '0xd8a4ae903349d12f4f96391cb47ea769a5535e57963562a5ae0ef932b18137e2',
    ///   'Token Name',
    ///   'TN',
    ///   13,
    ///   '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266'
    /// ]
    const DEPLOY_INTERCHAIN_TOKEN: &str = "0000000000000000000000000000000000000000000000000000000000000001d8a4ae903349d12f4f96391cb47ea769a5535e57963562a5ae0ef932b18137e200000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000d0000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000000a546f6b656e204e616d65000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002544e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014f39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000";

    #[test]
    fn deploy_token_interchain_token_decode() {
        // Setup
        let gmp = GMPPayload::decode(&hex::decode(DEPLOY_INTERCHAIN_TOKEN).unwrap()).unwrap();

        // Action
        let GMPPayload::DeployInterchainToken(data) = gmp else {
            panic!("wrong variant");
        };

        // Assert
        assert_eq!(
            hex::encode(data.token_id.abi_encode()),
            "d8a4ae903349d12f4f96391cb47ea769a5535e57963562a5ae0ef932b18137e2"
        );
        assert_eq!(data.name, "Token Name".to_string());
        assert_eq!(data.symbol, "TN".to_string(),);
        assert_eq!(data.decimals, 13,);
        assert_eq!(
            data.minter,
            hex::decode("f39fd6e51aad88f6f4ce6ab8827279cfffb92266").unwrap()
        );
    }

    #[test]
    fn deploy_token_interchain_token_encode() {
        assert_eq!(
            hex::encode(
                GMPPayload::decode(&hex::decode(DEPLOY_INTERCHAIN_TOKEN).unwrap())
                    .unwrap()
                    .encode()
            ),
            DEPLOY_INTERCHAIN_TOKEN,
            "encode-decode should be idempotent"
        );
    }
}
