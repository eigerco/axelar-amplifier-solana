//! Instructions supported by the multicall program.

use axelar_executable::{AxelarMessagePayload, EncodingScheme, PayloadError};
use borsh::{BorshDeserialize, BorshSerialize};
use error::BuilderError;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

/// Instructions supported by the multicall program.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum MultiCallInstruction {
    /// `MultiCall` instruction
    MultiCall {
        /// [`Vec`] containing a serialized
        /// [`AxelarMessagePayload`](axelar_executable::AxelarMessagePayload) built using
        /// [`MultiCallPayloadBuilder`](crate::MultiCallPayloadBuilder).
        payload: Vec<u8>,
    },
}

/// Encoding and decoding of multicall program payloads.
pub mod encoding {
    use alloy_sol_types::{sol, SolValue};
    use borsh::BorshDeserialize;

    use super::*;

    /// Payload for a program call
    #[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
    pub struct ProgramPayload {
        /// The data to pass as instruction data to the program.
        pub instruction_data: Vec<u8>,
        /// The index of the program account in the top-level accounts slice.
        pub program_account_index: usize,
        /// The start index within the top-level accounts slice where the
        /// accounts for this program call are located.
        pub accounts_start_index: usize,
        /// The end index within the top-level accounts slice where the accounts
        /// for this program call are located.
        pub accounts_end_index: usize,
    }

    /// Multicall Program Payload
    #[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
    pub struct MultiCallPayload {
        /// Array of [`ProgramPayload`].
        pub payloads: Vec<ProgramPayload>,
    }

    sol! {
        /// Payload for a program call. Used for ABI encoding.
        #[repr(C)]
        #[derive(Debug, PartialEq, Eq)]
        struct AbiProgramPayload {
            /// The data to pass as instruction data to the program.
            bytes instruction_data;
            /// The index of the program account in the top-level accounts slice.
            uint64 program_account_index;
            /// The start index within the top-level accounts slice where the accounts for this program call are
            /// located.
            uint64 accounts_start_index;
            /// The end index within the top-level accounts slice where the accounts for this program call are
            /// located.
            uint64 accounts_end_index;
        }

        /// Multicall Program Payload. Used for ABI encoding.
        #[repr(C)]
        #[derive(Debug, PartialEq, Eq)]
        struct AbiMultiCallPayload {
            /// Array of [`AbiProgramPayload`].
            AbiProgramPayload[] payloads;
        }

    }

    impl TryFrom<AbiMultiCallPayload> for MultiCallPayload {
        type Error = PayloadError;

        fn try_from(value: AbiMultiCallPayload) -> Result<Self, Self::Error> {
            Ok(Self {
                payloads: value
                    .payloads
                    .into_iter()
                    .map(|payload| -> Result<ProgramPayload, PayloadError> {
                        Ok(ProgramPayload {
                            instruction_data: payload.instruction_data.into(),
                            program_account_index: usize::try_from(payload.program_account_index)
                                .map_err(|_err| PayloadError::Conversion)?,
                            accounts_start_index: usize::try_from(payload.accounts_start_index)
                                .map_err(|_err| PayloadError::Conversion)?,
                            accounts_end_index: usize::try_from(payload.accounts_end_index)
                                .map_err(|_err| PayloadError::Conversion)?,
                        })
                    })
                    .collect::<Result<Vec<ProgramPayload>, PayloadError>>()?,
            })
        }
    }

    impl TryFrom<MultiCallPayload> for AbiMultiCallPayload {
        type Error = PayloadError;
        fn try_from(value: MultiCallPayload) -> Result<Self, Self::Error> {
            Ok(Self {
                payloads: value
                    .payloads
                    .into_iter()
                    .map(|payload| -> Result<AbiProgramPayload, PayloadError> {
                        Ok(AbiProgramPayload {
                            instruction_data: payload.instruction_data.into(),
                            program_account_index: u64::try_from(payload.program_account_index)
                                .map_err(|_err| PayloadError::Conversion)?,

                            accounts_start_index: u64::try_from(payload.accounts_start_index)
                                .map_err(|_err| PayloadError::Conversion)?,
                            accounts_end_index: u64::try_from(payload.accounts_end_index)
                                .map_err(|_err| PayloadError::Conversion)?,
                        })
                    })
                    .collect::<Result<Vec<AbiProgramPayload>, PayloadError>>()?,
            })
        }
    }

    impl MultiCallPayload {
        /// Tries to decodes the payload from a slice using the specified
        /// encoding scheme.
        ///
        /// # Errors
        /// - [`PayloadError::InvalidEncodingScheme`] - The encoding scheme
        ///   passed is not supported.
        /// - [`PayloadError::BorshDeserializeError`] - The payload could not be
        ///   deserialized using Borsh.
        /// - [`PayloadError::AbiError`] - The payload could not be decoded
        ///   using the ABI.
        pub fn decode(data: &[u8], encoding: EncodingScheme) -> Result<Self, PayloadError> {
            match encoding {
                EncodingScheme::Borsh => Ok(borsh::from_slice(data)
                    .map_err(|_error| PayloadError::BorshDeserializeError)?),
                EncodingScheme::AbiEncoding => {
                    Ok(AbiMultiCallPayload::abi_decode(data, true)?.try_into()?)
                }
                _ => Err(PayloadError::InvalidEncodingScheme),
            }
        }

        /// Tries to encode the payload using the specified encoding scheme.
        ///
        /// # Errors
        ///
        /// - [`PayloadError::InvalidEncodingScheme`] - The encoding scheme
        ///   passed is not supported.
        /// - [`PayloadError::BorshSerializeError`] - The payload could not be
        ///   serialized using Borsh.
        /// - [`PayloadError::AbiError`] - The payload could not be encoded
        ///   using the ABI.
        pub fn encode(self, encoding: EncodingScheme) -> Result<Vec<u8>, PayloadError> {
            match encoding {
                EncodingScheme::Borsh => {
                    Ok(borsh::to_vec(&self).map_err(|_error| PayloadError::BorshSerializeError)?)
                }
                EncodingScheme::AbiEncoding => {
                    Ok(AbiMultiCallPayload::try_from(self)?.abi_encode())
                }
                _ => Err(PayloadError::InvalidEncodingScheme),
            }
        }
    }
}

/// Builder for a multicall [`DataPayload`].
#[derive(Debug, Clone, Default)]
pub struct MultiCallPayloadBuilder {
    payloads: Vec<(Pubkey, Vec<AccountMeta>, Vec<u8>)>,
    encoding: Option<EncodingScheme>,
    encoded_payload_buffer: Vec<u8>,
}

impl MultiCallPayloadBuilder {
    /// Sets the [`EncodingScheme`] to be used for the payload encoding.
    #[must_use]
    pub const fn encoding_scheme(mut self, encoding: EncodingScheme) -> Self {
        self.encoding = Some(encoding);
        self
    }

    /// Adds a program instruction to the multicall payload.
    ///
    /// # Errors
    ///
    /// - [`PayloadError::BorshSerializeError`] - The instruction could not be
    ///   serialized using Borsh.
    pub fn add_instruction(
        mut self,
        program_id: Pubkey,
        accounts: Vec<AccountMeta>,
        instruction: Vec<u8>,
    ) -> Result<Self, PayloadError> {
        self.payloads.push((program_id, accounts, instruction));

        Ok(self)
    }

    /// Builds and returns the accounts and payload using the current builder state.
    ///
    /// This method clears the internal payloads vector but preserves the builder for potential reuse.
    ///
    /// # Errors
    ///
    /// - [`PayloadError::InvalidEncodingScheme`] - The encoding scheme was not set.
    /// - [`PayloadError::BorshSerializeError`] - The payload could not be borsh encoded.
    /// - [`PayloadError::AbiError`] - Error encoding the payload using the ABI encoder.
    pub fn build(&mut self) -> Result<AxelarMessagePayload<'_>, BuilderError> {
        let encoding = self.encoding.ok_or(PayloadError::InvalidEncodingScheme)?;
        let mut top_level_accounts = Vec::new();
        let mut program_payloads = Vec::with_capacity(self.payloads.len());

        // Since this method now borrows `&mut self` instead of consuming `self`, we use `mem::take` to
        // get ownership of the payloads while keeping `self` in a valid state so the returned `AxelarMessagePayload`
        // can reference `self.encoded_payload_buffer`.
        for (program_id, mut accounts, instruction_data) in core::mem::take(&mut self.payloads) {
            if accounts.is_empty() {
                return Err(BuilderError::NotAccountForIxError);
            }

            let current_index = top_level_accounts.len();

            top_level_accounts.push(AccountMeta {
                pubkey: program_id,
                is_signer: false,
                is_writable: false,
            });

            let account_start_index = current_index
                .checked_add(1)
                .ok_or(PayloadError::Conversion)?;
            let account_end_index = account_start_index
                .checked_add(accounts.len())
                .ok_or(PayloadError::Conversion)?;

            let program_payload = encoding::ProgramPayload {
                instruction_data,
                program_account_index: current_index,
                accounts_start_index: account_start_index,
                accounts_end_index: account_end_index,
            };

            top_level_accounts.append(&mut accounts);
            program_payloads.push(program_payload);
        }

        self.encoded_payload_buffer = encoding::MultiCallPayload {
            payloads: program_payloads,
        }
        .encode(encoding)?;

        Ok(AxelarMessagePayload::new(
            &self.encoded_payload_buffer,
            &top_level_accounts,
            encoding,
        ))
    }
}

/// Error types for the multicall program builder.
pub mod error {
    use axelar_executable::PayloadError;
    use thiserror::Error;

    /// Error types for the multicall program builder.
    #[derive(Error, Debug, PartialEq)]
    pub enum BuilderError {
        /// Error types for the multicall program builder payload.
        #[error("Payload encoding scheme not set")]
        PayloadError(PayloadError),
        /// Error when an ix does not have any accounts.
        #[error("Program ix must have at least one account")]
        NotAccountForIxError,
    }

    impl From<PayloadError> for BuilderError {
        fn from(error: PayloadError) -> Self {
            Self::PayloadError(error)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use axelar_executable::EncodingScheme;

    use crate::instructions::encoding::{MultiCallPayload, ProgramPayload};

    #[test]
    fn multicall_payload_encode_decode_roundtrip() {
        for encoding in [EncodingScheme::Borsh, EncodingScheme::AbiEncoding] {
            let payload = MultiCallPayload {
                payloads: vec![
                    ProgramPayload {
                        instruction_data: vec![1, 2, 3],
                        program_account_index: 0,
                        accounts_start_index: 1,
                        accounts_end_index: 2,
                    },
                    ProgramPayload {
                        instruction_data: vec![4, 5, 6],
                        program_account_index: 3,
                        accounts_start_index: 4,
                        accounts_end_index: 5,
                    },
                ],
            };

            let encoded = payload.clone().encode(encoding).unwrap();
            let decoded = MultiCallPayload::decode(&encoded, encoding).unwrap();

            assert_eq!(payload, decoded);
        }
    }

    #[test]
    fn test_multicall_do_not_allow_empty_accounts_on_ixs() {
        let builder = super::MultiCallPayloadBuilder::default();
        let program_id = solana_program::system_program::id();
        let accounts = vec![]; // No accounts

        let mut res = builder
            .encoding_scheme(EncodingScheme::Borsh)
            .add_instruction(program_id, accounts, vec![1, 2, 3])
            .unwrap();

        let err = res.build().err().unwrap();
        assert_eq!(err, super::error::BuilderError::NotAccountForIxError);
    }
}
