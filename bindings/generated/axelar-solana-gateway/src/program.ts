import { PublicKey } from "@solana/web3.js";
import { Program, AnchorProvider } from "@coral-xyz/anchor";

import { AxelarSolanaGatewayCoder } from "./coder";

export const AXELAR_SOLANA_GATEWAY_PROGRAM_ID = new PublicKey(
  "gtwLjHAsfKAR6GWB4hzTUAA1w4SDdFMKamtGA5ttMEe"
);

interface GetProgramParams {
  programId?: PublicKey;
  provider?: AnchorProvider;
}

export function axelarSolanaGatewayProgram(
  params?: GetProgramParams
): Program<AxelarSolanaGateway> {
  return new Program<AxelarSolanaGateway>(
    IDL,
    params?.programId ?? AXELAR_SOLANA_GATEWAY_PROGRAM_ID,
    params?.provider,
    new AxelarSolanaGatewayCoder(IDL)
  );
}

type AxelarSolanaGateway = {
  version: "0.1.0";
  name: "axelar_solana_gateway";
  instructions: [
    {
      name: "approveMessage";
      accounts: [
        {
          name: "gatewayRootPda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "verificationSessionPda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "incomingMessagePda";
          isMut: true;
          isSigner: false;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "message";
          type: {
            defined: "MerkleisedMessage";
          };
        },
        {
          name: "payloadMerkleRoot";
          type: {
            array: ["u8", 32];
          };
        }
      ];
    },
    {
      name: "rotateSigners";
      accounts: [
        {
          name: "gatewayRootPda";
          isMut: true;
          isSigner: false;
        },
        {
          name: "verificationSessionAccount";
          isMut: false;
          isSigner: false;
        },
        {
          name: "currentVerifierSetTrackerPda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "newVerifierSetTrackerPda";
          isMut: true;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "operator";
          isMut: true;
          isSigner: true;
          isOptional: true;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "newVerifierSetMerkleRoot";
          type: {
            array: ["u8", 32];
          };
        }
      ];
    },
    {
      name: "callContract";
      accounts: [
        {
          name: "senderProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "senderCallContractPda";
          isMut: false;
          isSigner: true;
        },
        {
          name: "gatewayRootPda";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "destinationChain";
          type: "string";
        },
        {
          name: "destinationContractAddress";
          type: "string";
        },
        {
          name: "payload";
          type: "bytes";
        },
        {
          name: "signingPdaBump";
          type: "u8";
        }
      ];
    },
    {
      name: "callContractOffchainData";
      accounts: [
        {
          name: "senderProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "senderCallContractPda";
          isMut: false;
          isSigner: true;
        },
        {
          name: "gatewayRootPda";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "destinationChain";
          type: "string";
        },
        {
          name: "destinationContractAddress";
          type: "string";
        },
        {
          name: "payloadHash";
          type: {
            array: ["u8", 32];
          };
        },
        {
          name: "signingPdaBump";
          type: "u8";
        }
      ];
    },
    {
      name: "initializeConfig";
      accounts: [
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "upgradeAuthority";
          isMut: false;
          isSigner: true;
        },
        {
          name: "gatewayProgramData";
          isMut: false;
          isSigner: false;
        },
        {
          name: "gatewayConfigPda";
          isMut: true;
          isSigner: false;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "domainSeparator";
          type: {
            array: ["u8", 32];
          };
        },
        {
          name: "initialSignerSets";
          type: {
            vec: {
              array: ["u8", 32];
            };
          };
        },
        {
          name: "minimumRotationDelay";
          type: "u64";
        },
        {
          name: "operator";
          type: "publicKey";
        },
        {
          name: "previousVerifierRetention";
          type: {
            defined: "U256";
          };
        }
      ];
    },
    {
      name: "initializePayloadVerificationSession";
      accounts: [
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "gatewayConfigPda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "verificationSessionPda";
          isMut: true;
          isSigner: false;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "payloadMerkleRoot";
          type: {
            array: ["u8", 32];
          };
        }
      ];
    },
    {
      name: "verifySignature";
      accounts: [
        {
          name: "gatewayConfigPda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "verificationSessionPda";
          isMut: true;
          isSigner: false;
        },
        {
          name: "verifierSetTrackerPda";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "payloadMerkleRoot";
          type: {
            array: ["u8", 32];
          };
        },
        {
          name: "verifierInfo";
          type: {
            defined: "SigningVerifierSetInfo";
          };
        }
      ];
    },
    {
      name: "initializeMessagePayload";
      accounts: [
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "gatewayRootPda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "incomingMessagePda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "messagePayloadPda";
          isMut: true;
          isSigner: false;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "bufferSize";
          type: "u64";
        },
        {
          name: "commandId";
          type: {
            array: ["u8", 32];
          };
        }
      ];
    },
    {
      name: "writeMessagePayload";
      accounts: [
        {
          name: "authority";
          isMut: true;
          isSigner: true;
        },
        {
          name: "gatewayRootPda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "incomingMessagePda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "messagePayloadPda";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "offset";
          type: "u64";
        },
        {
          name: "bytes";
          type: "bytes";
        },
        {
          name: "commandId";
          type: {
            array: ["u8", 32];
          };
        }
      ];
    },
    {
      name: "commitMessagePayload";
      accounts: [
        {
          name: "authority";
          isMut: true;
          isSigner: true;
        },
        {
          name: "gatewayRootPda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "incomingMessagePda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "messagePayloadPda";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "commandId";
          type: {
            array: ["u8", 32];
          };
        }
      ];
    },
    {
      name: "closeMessagePayload";
      accounts: [
        {
          name: "authority";
          isMut: true;
          isSigner: false;
        },
        {
          name: "gatewayRootPda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "incomingMessagePda";
          isMut: false;
          isSigner: false;
        },
        {
          name: "messagePayloadPda";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "commandId";
          type: {
            array: ["u8", 32];
          };
        }
      ];
    },
    {
      name: "validateMessage";
      accounts: [
        {
          name: "incomingMessagePda";
          isMut: true;
          isSigner: false;
        },
        {
          name: "signingPda";
          isMut: false;
          isSigner: true;
        }
      ];
      args: [
        {
          name: "message";
          type: {
            defined: "Message";
          };
        }
      ];
    },
    {
      name: "transferOperatorship";
      accounts: [
        {
          name: "gatewayRootPda";
          isMut: true;
          isSigner: false;
        },
        {
          name: "currentOperatorOrGatewayProgramOwner";
          isMut: false;
          isSigner: true;
        },
        {
          name: "programdata";
          isMut: false;
          isSigner: false;
        },
        {
          name: "newOperator";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    }
  ];
  types: [
    {
      name: "MerkleisedMessage";
      type: {
        kind: "struct";
        fields: [
          {
            name: "leaf";
            type: {
              defined: "MessageLeaf";
            };
          },
          {
            name: "proof";
            type: "bytes";
          }
        ];
      };
    },
    {
      name: "MessageLeaf";
      type: {
        kind: "struct";
        fields: [
          {
            name: "message";
            type: {
              defined: "Message";
            };
          },
          {
            name: "position";
            type: "u16";
          },
          {
            name: "setSize";
            type: "u16";
          },
          {
            name: "domainSeparator";
            type: {
              array: ["u8", 32];
            };
          },
          {
            name: "signingVerifierSet";
            type: {
              array: ["u8", 32];
            };
          }
        ];
      };
    },
    {
      name: "Message";
      type: {
        kind: "struct";
        fields: [
          {
            name: "ccId";
            type: {
              defined: "CrossChainId";
            };
          },
          {
            name: "sourceAddress";
            type: "string";
          },
          {
            name: "destinationChain";
            type: "string";
          },
          {
            name: "destinationAddress";
            type: "string";
          },
          {
            name: "payloadHash";
            type: {
              array: ["u8", 32];
            };
          }
        ];
      };
    },
    {
      name: "CrossChainId";
      type: {
        kind: "struct";
        fields: [
          {
            name: "chain";
            type: "string";
          },
          {
            name: "id";
            type: "string";
          }
        ];
      };
    },
    {
      name: "SigningVerifierSetInfo";
      type: {
        kind: "struct";
        fields: [
          {
            name: "signature";
            type: {
              defined: "Signature";
            };
          },
          {
            name: "leaf";
            type: {
              defined: "VerifierSetLeaf";
            };
          },
          {
            name: "merkleProof";
            type: "bytes";
          }
        ];
      };
    },
    {
      name: "VerifierSetLeaf";
      type: {
        kind: "struct";
        fields: [
          {
            name: "nonce";
            type: "u64";
          },
          {
            name: "quorum";
            type: "u128";
          },
          {
            name: "signerPubkey";
            type: {
              defined: "PublicKey";
            };
          },
          {
            name: "signerWeight";
            type: "u128";
          },
          {
            name: "position";
            type: "u16";
          },
          {
            name: "setSize";
            type: "u16";
          },
          {
            name: "domainSeparator";
            type: {
              array: ["u8", 32];
            };
          }
        ];
      };
    },
    {
      name: "U256";
      type: {
        kind: "struct";
        fields: [
          {
            name: "value";
            type: {
              array: ["u64", 4];
            };
          }
        ];
      };
    },
    {
      name: "PublicKey";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Secp256k1";
            fields: [
              {
                array: ["u8", 33];
              }
            ];
          },
          {
            name: "Ed25519";
            fields: [
              {
                array: ["u8", 32];
              }
            ];
          }
        ];
      };
    },
    {
      name: "Signature";
      type: {
        kind: "enum";
        variants: [
          {
            name: "EcdsaRecoverable";
            fields: [
              {
                array: ["u8", 65];
              }
            ];
          },
          {
            name: "Ed25519";
            fields: [
              {
                array: ["u8", 64];
              }
            ];
          }
        ];
      };
    }
  ];
  errors: [
    {
      code: 0;
      name: "VerifierSetAlreadyInitialised";
      msg: "Verifier set already initialized";
    },
    {
      code: 1;
      name: "SlotAlreadyVerified";
      msg: "Slot has been previously verified";
    },
    {
      code: 2;
      name: "MessageAlreadyInitialised";
      msg: "Message already initialized";
    },
    {
      code: 3;
      name: "VerificationSessionPDAInitialised";
      msg: "Verification session PDA already initialized";
    },
    {
      code: 4;
      name: "VerifierSetTrackerAlreadyInitialised";
      msg: "Verifier set tracker PDA already initialized";
    },
    {
      code: 5;
      name: "SlotIsOutOfBounds";
      msg: "Slot is out of bounds";
    },
    {
      code: 6;
      name: "InvalidDigitalSignature";
      msg: "Digital signature verification failed";
    },
    {
      code: 7;
      name: "LeafNodeNotPartOfMerkleRoot";
      msg: "Leaf node not part of Merkle root";
    },
    {
      code: 8;
      name: "InvalidMerkleProof";
      msg: "Signer is not a member of the active verifier set";
    },
    {
      code: 9;
      name: "InvalidDestinationAddress";
      msg: "Invalid destination address";
    },
    {
      code: 10;
      name: "MessagePayloadAlreadyInitialized";
      msg: "Message Payload PDA was already initialized";
    },
    {
      code: 11;
      name: "MessagePayloadAlreadyCommitted";
      msg: "Message Payload has already been committed";
    },
    {
      code: 12;
      name: "EpochCalculationOverflow";
      msg: "Epoch calculation resulted in an underflow";
    },
    {
      code: 13;
      name: "VerifierSetTooOld";
      msg: "Verifier set too old";
    },
    {
      code: 14;
      name: "BytemuckDataLenInvalid";
      msg: "Invalid bytemucked data length";
    },
    {
      code: 15;
      name: "SigningSessionNotValid";
      msg: "Signing session not valid";
    },
    {
      code: 16;
      name: "InvalidVerificationSessionPDA";
      msg: "Invalid verification session PDA";
    },
    {
      code: 17;
      name: "InvalidVerifierSetTrackerProvided";
      msg: "Invalid verifier set tracker provided";
    },
    {
      code: 18;
      name: "ProofNotSignedByLatestVerifierSet";
      msg: "Proof not signed by latest verifier set";
    },
    {
      code: 19;
      name: "RotationCooldownNotDone";
      msg: "Rotation cooldown not done";
    },
    {
      code: 20;
      name: "InvalidProgramDataDerivation";
      msg: "Invalid program data derivation";
    },
    {
      code: 21;
      name: "InvalidLoaderContent";
      msg: "Invalid loader content";
    },
    {
      code: 22;
      name: "InvalidLoaderState";
      msg: "Invalid loader state";
    },
    {
      code: 23;
      name: "OperatorOrUpgradeAuthorityMustBeSigner";
      msg: "Operator or upgrade authority must be signer";
    },
    {
      code: 24;
      name: "InvalidOperatorOrAuthorityAccount";
      msg: "Invalid operator or authority account";
    },
    {
      code: 25;
      name: "MessageNotApproved";
      msg: "Message not approved";
    },
    {
      code: 26;
      name: "MessageHasBeenTamperedWith";
      msg: "Message has been tampered with";
    },
    {
      code: 27;
      name: "InvalidSigningPDA";
      msg: "Invalid signing PDA";
    },
    {
      code: 28;
      name: "CallerNotSigner";
      msg: "Caller not signer";
    }
  ];
};

const IDL: AxelarSolanaGateway = {
  version: "0.1.0",
  name: "axelar_solana_gateway",
  instructions: [
    {
      name: "approveMessage",
      accounts: [
        {
          name: "gatewayRootPda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "verificationSessionPda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "incomingMessagePda",
          isMut: true,
          isSigner: false,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "message",
          type: {
            defined: "MerkleisedMessage",
          },
        },
        {
          name: "payloadMerkleRoot",
          type: {
            array: ["u8", 32],
          },
        },
      ],
    },
    {
      name: "rotateSigners",
      accounts: [
        {
          name: "gatewayRootPda",
          isMut: true,
          isSigner: false,
        },
        {
          name: "verificationSessionAccount",
          isMut: false,
          isSigner: false,
        },
        {
          name: "currentVerifierSetTrackerPda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "newVerifierSetTrackerPda",
          isMut: true,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "operator",
          isMut: true,
          isSigner: true,
          isOptional: true,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "newVerifierSetMerkleRoot",
          type: {
            array: ["u8", 32],
          },
        },
      ],
    },
    {
      name: "callContract",
      accounts: [
        {
          name: "senderProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "senderCallContractPda",
          isMut: false,
          isSigner: true,
        },
        {
          name: "gatewayRootPda",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "destinationChain",
          type: "string",
        },
        {
          name: "destinationContractAddress",
          type: "string",
        },
        {
          name: "payload",
          type: "bytes",
        },
        {
          name: "signingPdaBump",
          type: "u8",
        },
      ],
    },
    {
      name: "callContractOffchainData",
      accounts: [
        {
          name: "senderProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "senderCallContractPda",
          isMut: false,
          isSigner: true,
        },
        {
          name: "gatewayRootPda",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "destinationChain",
          type: "string",
        },
        {
          name: "destinationContractAddress",
          type: "string",
        },
        {
          name: "payloadHash",
          type: {
            array: ["u8", 32],
          },
        },
        {
          name: "signingPdaBump",
          type: "u8",
        },
      ],
    },
    {
      name: "initializeConfig",
      accounts: [
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "upgradeAuthority",
          isMut: false,
          isSigner: true,
        },
        {
          name: "gatewayProgramData",
          isMut: false,
          isSigner: false,
        },
        {
          name: "gatewayConfigPda",
          isMut: true,
          isSigner: false,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "domainSeparator",
          type: {
            array: ["u8", 32],
          },
        },
        {
          name: "initialSignerSets",
          type: {
            vec: {
              array: ["u8", 32],
            },
          },
        },
        {
          name: "minimumRotationDelay",
          type: "u64",
        },
        {
          name: "operator",
          type: "publicKey",
        },
        {
          name: "previousVerifierRetention",
          type: {
            defined: "U256",
          },
        },
      ],
    },
    {
      name: "initializePayloadVerificationSession",
      accounts: [
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "gatewayConfigPda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "verificationSessionPda",
          isMut: true,
          isSigner: false,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "payloadMerkleRoot",
          type: {
            array: ["u8", 32],
          },
        },
      ],
    },
    {
      name: "verifySignature",
      accounts: [
        {
          name: "gatewayConfigPda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "verificationSessionPda",
          isMut: true,
          isSigner: false,
        },
        {
          name: "verifierSetTrackerPda",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "payloadMerkleRoot",
          type: {
            array: ["u8", 32],
          },
        },
        {
          name: "verifierInfo",
          type: {
            defined: "SigningVerifierSetInfo",
          },
        },
      ],
    },
    {
      name: "initializeMessagePayload",
      accounts: [
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "gatewayRootPda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "incomingMessagePda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "messagePayloadPda",
          isMut: true,
          isSigner: false,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "bufferSize",
          type: "u64",
        },
        {
          name: "commandId",
          type: {
            array: ["u8", 32],
          },
        },
      ],
    },
    {
      name: "writeMessagePayload",
      accounts: [
        {
          name: "authority",
          isMut: true,
          isSigner: true,
        },
        {
          name: "gatewayRootPda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "incomingMessagePda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "messagePayloadPda",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "offset",
          type: "u64",
        },
        {
          name: "bytes",
          type: "bytes",
        },
        {
          name: "commandId",
          type: {
            array: ["u8", 32],
          },
        },
      ],
    },
    {
      name: "commitMessagePayload",
      accounts: [
        {
          name: "authority",
          isMut: true,
          isSigner: true,
        },
        {
          name: "gatewayRootPda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "incomingMessagePda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "messagePayloadPda",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "commandId",
          type: {
            array: ["u8", 32],
          },
        },
      ],
    },
    {
      name: "closeMessagePayload",
      accounts: [
        {
          name: "authority",
          isMut: true,
          isSigner: false,
        },
        {
          name: "gatewayRootPda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "incomingMessagePda",
          isMut: false,
          isSigner: false,
        },
        {
          name: "messagePayloadPda",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "commandId",
          type: {
            array: ["u8", 32],
          },
        },
      ],
    },
    {
      name: "validateMessage",
      accounts: [
        {
          name: "incomingMessagePda",
          isMut: true,
          isSigner: false,
        },
        {
          name: "signingPda",
          isMut: false,
          isSigner: true,
        },
      ],
      args: [
        {
          name: "message",
          type: {
            defined: "Message",
          },
        },
      ],
    },
    {
      name: "transferOperatorship",
      accounts: [
        {
          name: "gatewayRootPda",
          isMut: true,
          isSigner: false,
        },
        {
          name: "currentOperatorOrGatewayProgramOwner",
          isMut: false,
          isSigner: true,
        },
        {
          name: "programdata",
          isMut: false,
          isSigner: false,
        },
        {
          name: "newOperator",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
  ],
  types: [
    {
      name: "MerkleisedMessage",
      type: {
        kind: "struct",
        fields: [
          {
            name: "leaf",
            type: {
              defined: "MessageLeaf",
            },
          },
          {
            name: "proof",
            type: "bytes",
          },
        ],
      },
    },
    {
      name: "MessageLeaf",
      type: {
        kind: "struct",
        fields: [
          {
            name: "message",
            type: {
              defined: "Message",
            },
          },
          {
            name: "position",
            type: "u16",
          },
          {
            name: "setSize",
            type: "u16",
          },
          {
            name: "domainSeparator",
            type: {
              array: ["u8", 32],
            },
          },
          {
            name: "signingVerifierSet",
            type: {
              array: ["u8", 32],
            },
          },
        ],
      },
    },
    {
      name: "Message",
      type: {
        kind: "struct",
        fields: [
          {
            name: "ccId",
            type: {
              defined: "CrossChainId",
            },
          },
          {
            name: "sourceAddress",
            type: "string",
          },
          {
            name: "destinationChain",
            type: "string",
          },
          {
            name: "destinationAddress",
            type: "string",
          },
          {
            name: "payloadHash",
            type: {
              array: ["u8", 32],
            },
          },
        ],
      },
    },
    {
      name: "CrossChainId",
      type: {
        kind: "struct",
        fields: [
          {
            name: "chain",
            type: "string",
          },
          {
            name: "id",
            type: "string",
          },
        ],
      },
    },
    {
      name: "SigningVerifierSetInfo",
      type: {
        kind: "struct",
        fields: [
          {
            name: "signature",
            type: {
              defined: "Signature",
            },
          },
          {
            name: "leaf",
            type: {
              defined: "VerifierSetLeaf",
            },
          },
          {
            name: "merkleProof",
            type: "bytes",
          },
        ],
      },
    },
    {
      name: "VerifierSetLeaf",
      type: {
        kind: "struct",
        fields: [
          {
            name: "nonce",
            type: "u64",
          },
          {
            name: "quorum",
            type: "u128",
          },
          {
            name: "signerPubkey",
            type: {
              defined: "PublicKey",
            },
          },
          {
            name: "signerWeight",
            type: "u128",
          },
          {
            name: "position",
            type: "u16",
          },
          {
            name: "setSize",
            type: "u16",
          },
          {
            name: "domainSeparator",
            type: {
              array: ["u8", 32],
            },
          },
        ],
      },
    },
    {
      name: "U256",
      type: {
        kind: "struct",
        fields: [
          {
            name: "value",
            type: {
              array: ["u64", 4],
            },
          },
        ],
      },
    },
    {
      name: "PublicKey",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Secp256k1",
            fields: [
              {
                array: ["u8", 33],
              },
            ],
          },
          {
            name: "Ed25519",
            fields: [
              {
                array: ["u8", 32],
              },
            ],
          },
        ],
      },
    },
    {
      name: "Signature",
      type: {
        kind: "enum",
        variants: [
          {
            name: "EcdsaRecoverable",
            fields: [
              {
                array: ["u8", 65],
              },
            ],
          },
          {
            name: "Ed25519",
            fields: [
              {
                array: ["u8", 64],
              },
            ],
          },
        ],
      },
    },
  ],
  errors: [
    {
      code: 0,
      name: "VerifierSetAlreadyInitialised",
      msg: "Verifier set already initialized",
    },
    {
      code: 1,
      name: "SlotAlreadyVerified",
      msg: "Slot has been previously verified",
    },
    {
      code: 2,
      name: "MessageAlreadyInitialised",
      msg: "Message already initialized",
    },
    {
      code: 3,
      name: "VerificationSessionPDAInitialised",
      msg: "Verification session PDA already initialized",
    },
    {
      code: 4,
      name: "VerifierSetTrackerAlreadyInitialised",
      msg: "Verifier set tracker PDA already initialized",
    },
    {
      code: 5,
      name: "SlotIsOutOfBounds",
      msg: "Slot is out of bounds",
    },
    {
      code: 6,
      name: "InvalidDigitalSignature",
      msg: "Digital signature verification failed",
    },
    {
      code: 7,
      name: "LeafNodeNotPartOfMerkleRoot",
      msg: "Leaf node not part of Merkle root",
    },
    {
      code: 8,
      name: "InvalidMerkleProof",
      msg: "Signer is not a member of the active verifier set",
    },
    {
      code: 9,
      name: "InvalidDestinationAddress",
      msg: "Invalid destination address",
    },
    {
      code: 10,
      name: "MessagePayloadAlreadyInitialized",
      msg: "Message Payload PDA was already initialized",
    },
    {
      code: 11,
      name: "MessagePayloadAlreadyCommitted",
      msg: "Message Payload has already been committed",
    },
    {
      code: 12,
      name: "EpochCalculationOverflow",
      msg: "Epoch calculation resulted in an underflow",
    },
    {
      code: 13,
      name: "VerifierSetTooOld",
      msg: "Verifier set too old",
    },
    {
      code: 14,
      name: "BytemuckDataLenInvalid",
      msg: "Invalid bytemucked data length",
    },
    {
      code: 15,
      name: "SigningSessionNotValid",
      msg: "Signing session not valid",
    },
    {
      code: 16,
      name: "InvalidVerificationSessionPDA",
      msg: "Invalid verification session PDA",
    },
    {
      code: 17,
      name: "InvalidVerifierSetTrackerProvided",
      msg: "Invalid verifier set tracker provided",
    },
    {
      code: 18,
      name: "ProofNotSignedByLatestVerifierSet",
      msg: "Proof not signed by latest verifier set",
    },
    {
      code: 19,
      name: "RotationCooldownNotDone",
      msg: "Rotation cooldown not done",
    },
    {
      code: 20,
      name: "InvalidProgramDataDerivation",
      msg: "Invalid program data derivation",
    },
    {
      code: 21,
      name: "InvalidLoaderContent",
      msg: "Invalid loader content",
    },
    {
      code: 22,
      name: "InvalidLoaderState",
      msg: "Invalid loader state",
    },
    {
      code: 23,
      name: "OperatorOrUpgradeAuthorityMustBeSigner",
      msg: "Operator or upgrade authority must be signer",
    },
    {
      code: 24,
      name: "InvalidOperatorOrAuthorityAccount",
      msg: "Invalid operator or authority account",
    },
    {
      code: 25,
      name: "MessageNotApproved",
      msg: "Message not approved",
    },
    {
      code: 26,
      name: "MessageHasBeenTamperedWith",
      msg: "Message has been tampered with",
    },
    {
      code: 27,
      name: "InvalidSigningPDA",
      msg: "Invalid signing PDA",
    },
    {
      code: 28,
      name: "CallerNotSigner",
      msg: "Caller not signer",
    },
  ],
};
