use anchor_discriminators_macros::InstructionDiscriminator;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, PartialEq, InstructionDiscriminator)]
enum InterchainTokenServiceInstruction {
    /// Initializes the interchain token service program.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. [writable,signer] The address of payer / sender
    /// 1. [] Program data account
    /// 2. [] Gateway root account
    /// 3. [writable] ITS root account
    /// 4. [] System program account
    /// 5. [] The account that will become the operator of the ITS
    /// 6. [writable] The address of the account that will store the roles of the operator account.
    Initialize {
        /// The name of the chain the ITS is running on.
        chain_name: String,

        /// The address of the ITS Hub
        its_hub_address: String,
    },

    /// Pauses or unpauses the interchain token service.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. [writable,signer] The address of the payer, needs to be the ITS owner.
    /// 1. [] The program data account.
    /// 2. [] Gateway root account
    /// 3. [writable] ITS root pda.
    SetPauseStatus {
        /// The new pause status.
        paused: bool,
    },
    /// Sets a chain as trusted, allowing communication between this ITS and the ITS of that chain.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. [writable,signer] The address of the payer, needs to be the ITS owner.
    /// 1. [] The program data account.
    /// 2. [] Gateway root account
    /// 3. [writable] ITS root pda.
    /// 4. [] The system program account.
    SetTrustedChain {
        /// The name of the chain to be trusted.
        chain_name: String,
    },

    /// Unsets a chain as trusted, disallowing communication between this ITS and the ITS of that chain.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. [writable,signer] The address of the payer, needs to be the ITS owner.
    /// 1. [] The program data account.
    /// 2. [] Gateway root account
    /// 3. [writable] ITS root pda.
    /// 4. [] The system program account.
    RemoveTrustedChain {
        /// The name of the chain from which trust is removed.
        chain_name: String,
    },

    /// Approves the deployment of remote token with a destination minter
    ///
    /// 0. [writable,signer] The address of the payer, needs to have minter role on the token
    ///    manager.
    /// 1. [] The token manager account associated with the token
    /// 2. [] The account that holds the payer roles on the token manager
    /// 3. [writable] The account that will hold the approval of the deployment
    /// 4. [] The system program account
    ApproveDeployRemoteInterchainToken {
        /// The address of the account that deployed the `InterchainToken`
        deployer: Pubkey,
        /// The salt used to deploy the `InterchainToken`
        salt: [u8; 32],
        /// The remote chain where the `InterchainToken` will be deployed.
        destination_chain: String,
        /// The approved address of the minter on the destination chain
        destination_minter: Vec<u8>,
    },

    /// Revokes an approval of a deployment of remote token with a destination minter
    ///
    /// 0. [writable,signer] The address of the payer, needs to have minter role on the token
    ///    manager.
    /// 1. [] The token manager account associated with the token
    /// 2. [] The account that holds the payer roles on the token manager
    /// 3. [writable] The account holding the approval of the deployment that should be revoked
    /// 4. [] The system program account
    RevokeDeployRemoteInterchainToken {
        /// The address of the account that deployed the `InterchainToken`
        deployer: Pubkey,
        /// The salt used to deploy the `InterchainToken`
        salt: [u8; 32],
        /// The remote chain where the `InterchainToken` would be deployed.
        destination_chain: String,
    },

    /// Registers a canonical token as an interchain token and deploys its token manager.
    ///
    /// 0. [writable,signer] The address of the payer
    /// 1. [] The Metaplex metadata account associated with the mint
    /// 2. [] The GMP gateway root account
    /// 3. [] The system program account
    /// 4. [] The ITS root account
    /// 5. [writable] The token manager account derived from the `token_id` that will be initialized
    /// 6. [] The mint account (token address) of the original token
    /// 7. [] The token manager Associated Token Account
    /// 8. [] The token program account that was used to create the mint (`spl_token` vs `spl_token_2022`)
    /// 9. [] The Associated Token Account program account (`spl_associated_token_account`)
    /// 10. [] The rent sysvar account
    /// 11. [] The Metaplex metadata program account (`mpl_token_metadata`)
    RegisterCanonicalInterchainToken,

    /// Deploys a canonical interchain token on a remote chain.
    ///
    /// 0. [writable,signer] The account of the deployer, which is also paying for the transaction
    /// 1. [] The Metaplex metadata account associated with the mint
    /// 2. [] The GMP gateway root account
    /// 3. [] The system program account
    /// 4. [] The ITS root account
    /// 5. [writable] The token manager account associated with the interchain token
    /// 6. [writable] The mint account (token address) to deploy
    /// 7. [writable] The token manager Associated Token Account associated with the mint
    /// 8. [] The token program account that was used to create the mint (`spl_token` vs `spl_token_2022`)
    /// 9. [] The Associated Token Account program account (`spl_associated_token_account`)
    /// 10. [writable] The account holding the roles of the deployer on the ITS root account
    /// 11. [] The rent sysvar account
    /// 12. [] Optional account to set as operator on the `TokenManager`.
    /// 13. [writable] In case an operator is being set, this should be the account holding the roles of
    ///     the operator on the `TokenManager`
    DeployRemoteCanonicalInterchainToken {
        /// The remote chain where the `InterchainToken` should be deployed.
        destination_chain: String,
        /// The gas amount to be sent for deployment.
        gas_value: u64,
        /// The bump from the call contract signing account PDA derivation
        signing_pda_bump: u8,
    },

    /// Transfers interchain tokens.
    ///
    /// 0. [writable,signer] The address of the payer
    /// 1. [maybe signer] The address of the owner or delegate of the source account of the
    ///    transfer. In case it's the `TokenManager`, it shouldn't be set as signer as the signing
    ///    happens on chain.
    /// 2. [writable] The source account from which the tokens are being transferred
    /// 3. [] The mint account (token address)
    /// 4. [] The token manager account associated with the interchain token
    /// 5. [writable] The token manager Associated Token Account associated with the mint
    /// 6. [] The token program account that was used to create the mint (`spl_token` vs `spl_token_2022`)
    /// 7. [writable] The account tracking the flow of this mint for the current epoch
    /// 8. [] The GMP gateway root account
    /// 9. [] The GMP gateway program account
    /// 10. [writable] The GMP gas configuration account
    /// 11. [] The GMP gas service program account
    /// 12. [] The system program account
    /// 13. [] The ITS root account
    /// 14. [] The GMP call contract signing account
    /// 15. [] The ITS program account
    InterchainTransfer {
        /// The token id associated with the token
        token_id: [u8; 32],

        /// The chain where the tokens are being transferred to.
        destination_chain: String,

        /// The address on the destination chain to send the tokens to.
        destination_address: Vec<u8>,

        /// Amount of tokens being transferred.
        amount: u64,

        /// The gas value to be paid for the deploy transaction
        gas_value: u64,

        /// The bump from the call contract signing account PDA derivation
        signing_pda_bump: u8,
    },

    /// Deploys an interchain token.
    ///
    /// 0. [writable,signer] The account of the deployer, which is also paying for the transaction
    /// 1. [] The GMP gateway root account
    /// 2. [] The system program account
    /// 3. [] The ITS root account
    /// 4. [writable] The token manager account associated with the interchain token
    /// 5. [writable] The mint account (token address) to deploy
    /// 6. [writable] The token manager Associated Token Account associated with the mint
    /// 7. [] The token program account (`spl_token_2022`)
    /// 8. [] The Associated Token Account program account (`spl_associated_token_account`)
    /// 9. [writable] The account holding the roles of the deployer on the ITS root account
    /// 10. [] The rent sysvar account
    /// 11. [] The instructions sysvar account
    /// 12. [] The Metaplex metadata program account (`mpl_token_metadata`)
    /// 13. [writable] The Metaplex metadata account associated with the mint
    /// 14. [] The account to set as minter of the token
    /// 15. [writable] The account holding the roles of the minter account on the `TokenManager`
    DeployInterchainToken {
        /// The salt used to derive the tokenId associated with the token
        salt: [u8; 32],

        /// Token name
        name: String,

        /// Token symbol
        symbol: String,

        /// Token decimals
        decimals: u8,

        /// Initial supply
        initial_supply: u64,
    },

    /// Deploys a remote interchain token
    ///
    /// 0. [writable,signer] The address of the payer
    /// 1. [] The mint account (token address)
    /// 2. [] The Metaplex metadata account associated with the mint
    /// 3. [] The instructions sysvar account
    /// 4. [] The Metaplex metadata program account (`mpl_token_metadata`)
    /// 5. [] The GMP gateway root account
    /// 6. [] The GMP gateway program account
    /// 7. [writable] The GMP gas configuration account
    /// 8. [] The GMP gas service program account
    /// 9. [] The system program account
    /// 10. [] The ITS root account
    /// 11. [] The GMP call contract signing account
    /// 12. [] The ITS program account
    DeployRemoteInterchainToken {
        /// The salt used to derive the tokenId associated with the token
        salt: [u8; 32],

        /// The chain where the `InterchainToken` should be deployed.
        destination_chain: String,

        /// The gas value to be paid for the deploy transaction
        gas_value: u64,

        /// Signing PDA bump
        signing_pda_bump: u8,
    },

    /// Deploys a remote interchain token with associated minter
    ///
    /// 0. [writable,signer] The address of the payer
    /// 1. [] The mint account (token address)
    /// 2. [] The Metaplex metadata account associated with the mint
    /// 3. [] The account of the minter that approved the deployment
    /// 4. [writable] The account holding the approval for the deployment
    /// 5. [] The account holding the roles of the minter on the token manager associated with the
    ///    interchain token
    /// 6. [] The token manager account associated with the interchain token
    /// 7. [] The instructions sysvar account
    /// 8. [] The Metaplex metadata program account (`mpl_token_metadata`)
    /// 9. [] The GMP gateway root account
    /// 10. [] The GMP gateway program account
    /// 11. [writable] The GMP gas configuration account
    /// 12. [] The GMP gas service program account
    /// 13. [] The system program account
    /// 14. [] The ITS root account
    /// 15. [] The GMP call contract signing account
    /// 16. [] The ITS program account
    DeployRemoteInterchainTokenWithMinter {
        /// The salt used to derive the tokenId associated with the token
        salt: [u8; 32],

        /// The chain where the `InterchainToken` should be deployed.
        destination_chain: String,

        /// The minter on the destination chain
        destination_minter: Vec<u8>,

        /// The gas value to be paid for the deploy transaction
        gas_value: u64,

        /// Signing PDA bump
        signing_pda_bump: u8,
    },

    /// Registers token metadata.
    ///
    /// 0. [writable,signer] The address of the payer
    /// 1. [] The mint account (token address)
    /// 2. [] The token program account that was used to create the mint (`spl_token` vs `spl_token_2022`)
    /// 3. [] The GMP gateway root account
    /// 4. [] The GMP gateway program account
    /// 5. [writable] The GMP gas configuration account
    /// 6. [] The GMP gas service program account
    /// 7. [] The system program account
    /// 8. [] The ITS root account
    /// 9. [] The GMP call contract signing account
    /// 10. [] The ITS program account
    RegisterTokenMetadata {
        /// The gas value to be paid for the GMP transaction
        gas_value: u64,
        /// The signing PDA bump
        signing_pda_bump: u8,
    },

    /// Registers a custom token with ITS, deploying a new [`TokenManager`] to manage it.
    ///
    /// 0. [writable,signer] The account of the deployer, which is also paying for the transaction
    /// 1. [] The Metaplex metadata account associated with the mint
    /// 2. [] The GMP gateway root account
    /// 3. [] The system program account
    /// 4. [] The ITS root account
    /// 5. [writable] The token manager account associated with the interchain token
    /// 6. [writable] The mint account (token address) to deploy
    /// 7. [writable] The token manager Associated Token Account associated with the mint
    /// 8. [] The token program account that was used to create the mint (`spl_token` vs `spl_token_2022`)
    /// 9. [] The Associated Token Account program account (`spl_associated_token_account`)
    /// 10. [writable] The account holding the roles of the deployer on the ITS root account
    /// 11. [] The rent sysvar account
    /// 12. [] Optional account to set as operator on the `TokenManager`.
    /// 13. [writable] In case an operator is being set, this should be the account holding the roles of
    ///     the operator on the `TokenManager`
    RegisterCustomToken {
        /// Salt used to derive the `token_id` associated with the token.
        salt: [u8; 32],
        /// The token manager type.
        // token_manager_type: state::token_manager::Type,
        /// The operator account
        operator: Option<Pubkey>,
    },

    /// Link a local token derived from salt and payer to another token on the `destination_chain`,
    /// at the `destination_token_address`.
    ///
    /// 0. [writable,signer] The address of the payer
    /// 1. [] The `TokenManager` account associated with the token being linked
    /// 2. [] The GMP gateway root account
    /// 3. [] The GMP gateway program account
    /// 4. [writable] The GMP gas configuration account
    /// 5. [] The GMP gas service program account
    /// 6. [] The system program account
    /// 7. [] The ITS root account
    /// 8. [] The GMP call contract signing account
    /// 9. [] The ITS program account
    LinkToken {
        /// Salt used to derive the `token_id` associated with the token.
        salt: [u8; 32],
        /// The chain where the token is being linked to.
        destination_chain: String,
        /// The address of the token on the destination chain.
        destination_token_address: Vec<u8>,
        /// The type of token manager used on the destination chain.
        // token_manager_type: state::token_manager::Type,
        /// The params required on the destination chain.
        link_params: Vec<u8>,
        /// The gas value to be paid for the GMP transaction
        gas_value: u64,
        /// The signing PDA bump
        signing_pda_bump: u8,
    },

    /// Transfers tokens to a contract on the destination chain and call the give instruction on
    /// it. This instruction is is the same as [`InterchainTransfer`], but will fail if call data
    /// is empty.
    ///
    /// 0. [writable,signer] The address of the payer
    /// 1. [maybe signer] The address of the owner or delegate of the source account of the
    ///    transfer. In case it's the `TokenManager`, it shouldn't be set as signer as the signing
    ///    happens on chain.
    /// 2. [writable] The source account from which the tokens are being transferred
    /// 3. [] The mint account (token address)
    /// 4. [] The token manager account associated with the interchain token
    /// 5. [writable] The token manager Associated Token Account associated with the mint
    /// 6. [] The token program account that was used to create the mint (`spl_token` vs `spl_token_2022`)
    /// 7. [writable] The account tracking the flow of this mint for the current epoch
    /// 8. [] The GMP gateway root account
    /// 9. [] The GMP gateway program account
    /// 10. [writable] The GMP gas configuration account
    /// 11. [] The GMP gas service program account
    /// 12. [] The system program account
    /// 13. [] The ITS root account
    /// 14. [] The GMP call contract signing account
    /// 15. [] The ITS program account
    CallContractWithInterchainToken {
        /// The token id associated with the token
        token_id: [u8; 32],

        /// The chain where the tokens are being transferred to.
        destination_chain: String,

        /// The address on the destination chain to send the tokens to.
        destination_address: Vec<u8>,

        /// Amount of tokens being transferred.
        amount: u64,

        /// Call data
        data: Vec<u8>,

        /// The gas value to be paid for the deploy transaction
        gas_value: u64,

        /// Signing PDA bump
        signing_pda_bump: u8,
    },

    /// Sets the flow limit for an interchain token.
    ///
    /// 0. [writable,signer] The address of the payer
    /// 1. [] The ITS root account
    /// 2. [writable] The token manager account associated with the interchain token
    /// 3. [writable] The account holding the roles of the payer on the ITS root account
    /// 4. [writable] The account holding the roles of the payer on the `TokenManager`
    SetFlowLimit {
        /// The new flow limit.
        flow_limit: u64,
    },

    /// Transfers operatorship to another account.
    ///
    /// 0. [] System program account.
    /// 1. [writable, signer] Payer account.
    /// 2. [] PDA for the payer roles on the resource which the operatorship is being transferred
    ///    from.
    /// 3. [] PDA for the resource.
    /// 4. [] Account to transfer operatorship to.
    /// 5. [writable] PDA with the roles on the resource the
    ///    operatorship is being transferred to.
    TransferOperatorship,

    /// Proposes operatorship transfer to another account.
    ///
    /// 0. [] System program account.
    /// 1. [writable, signer] Payer account.
    /// 2. [] PDA for the payer roles on the resource.
    /// 3. [] PDA for the resource.
    /// 4. [] Account to transfer operatorship to.
    /// 5. [writable] PDA with the roles on the resource for the accounts the
    ///    operatorship is being transferred to.
    /// 6. [] Account which the operatorship is being transferred from.
    /// 7. [writable] PDA with the roles on the resource for the account the
    ///    operatorship is being transferred from.
    /// 8. [writable] PDA for the proposal
    ProposeOperatorship,

    /// Accepts operatorship transfer from another account.
    ///
    /// 0. [] System program account.
    /// 1. [writable, signer] Payer account.
    /// 2. [] PDA for the payer roles on the resource.
    /// 3. [] PDA for the resource.
    /// 4. [] Account to transfer operatorship to.
    /// 5. [writable] PDA with the roles on the resource for the accounts the
    ///    operatorship is being transferred to.
    /// 6. [] Account which the operatorship is being transferred from.
    /// 7. [writable] PDA with the roles on the resource for the account the
    ///    operatorship is being transferred from.
    /// 8. [writable] PDA for the proposal
    AcceptOperatorship,

    /// Adds a flow limiter to a [`TokenManager`].
    ///
    /// 0. [] System program account.
    /// 1. [writable, signer] Payer account (must have operator role).
    /// 2. [] PDA for the payer roles on the token manager.
    /// 3. [] PDA for the token manager.
    /// 4. [] Account to add as flow limiter.
    /// 5. [writable] PDA with the roles on the token manager for the flow limiter being added.
    AddTokenManagerFlowLimiter,

    /// Removes a flow limiter from a [`TokenManager`].
    ///
    /// 0. [] System program account.
    /// 1. [writable, signer] Payer account (must have operator role).
    /// 2. [] PDA for the payer roles on the token manager.
    /// 3. [] PDA for the token manager.
    /// 4. [] Account to remove as flow limiter.
    /// 5. [writable] PDA with the roles on the token manager for the flow limiter being removed.
    RemoveTokenManagerFlowLimiter,

    /// Sets the flow limit for an interchain token.
    ///
    /// 0. [signer] Payer account.
    /// 1. [] ITS root PDA account.
    /// 2. [writable] The [`TokenManager`] PDA account.
    /// 3. [] The PDA account with the user roles on the [`TokenManager`].
    /// 4. [] The PDA account with the user roles on ITS.
    SetTokenManagerFlowLimit {
        /// The new flow limit.
        flow_limit: u64,
    },

    /// Transfers operatorship to another account.
    ///
    /// 0. [] ITS root PDA.
    /// 1. [] System program account.
    /// 2. [writable, signer] Payer account.
    /// 3. [] PDA for the payer roles on the resource which the operatorship is being transferred
    ///    from.
    /// 4. [] PDA for the resource.
    /// 5. [] Account to transfer operatorship to.
    /// 6. [writable] PDA with the roles on the resource the
    ///    operatorship is being transferred to.
    TransferTokenManagerOperatorship,

    /// Proposes operatorship transfer to another account.
    ///
    /// 0. [] System program account.
    /// 1. [writable, signer] Payer account.
    /// 2. [] PDA for the payer roles on the resource.
    /// 3. [] PDA for the resource.
    /// 4. [] Account to transfer operatorship to.
    /// 5. [writable] PDA with the roles on the resource for the accounts the
    ///    operatorship is being transferred to.
    /// 6. [] Account which the operatorship is being transferred from.
    /// 7. [writable] PDA with the roles on the resource for the account the
    ///    operatorship is being transferred from.
    /// 8. [writable] PDA for the proposal
    ProposeTokenManagerOperatorship,

    /// Accepts operatorship transfer from another account.
    ///
    /// 0. [] System program account.
    /// 1. [writable, signer] Payer account.
    /// 2. [] PDA for the payer roles on the resource.
    /// 3. [] PDA for the resource.
    /// 4. [] Account to transfer operatorship to.
    /// 5. [writable] PDA with the roles on the resource for the accounts the
    ///    operatorship is being transferred to.
    /// 6. [] Account which the operatorship is being transferred from.
    /// 7. [writable] PDA with the roles on the resource for the account the
    ///    operatorship is being transferred from.
    /// 8. [writable] PDA for the proposal
    AcceptTokenManagerOperatorship,

    /// Transfers the mint authority to the token manager allowing it to mint tokens and manage
    /// minters. The account transferring the authority gains minter role on the [`TokenManager`] and
    /// thus can then mint tokens through the ITS mitn instruction.
    ///
    /// 0. [writable, signer] Payer, current mint authority
    /// 1. [writable] The mint for which the authority is being handed over
    /// 2. [] ITS root account
    /// 3. [] The [`TokenManager`] account associated with the mint
    /// 4. [] The account that will hold the roles of the former authority on the [`TokenManager`]
    /// 5. [] The token program used to create the mint
    /// 6. [] The system program account
    HandoverMintAuthority {
        /// The id of the token registered with ITS for which the authority is being handed over.
        token_id: [u8; 32],
    },

    /// A proxy instruction to mint tokens whose mint authority is a
    /// `TokenManager`. Only users with the `minter` role on the mint account
    /// can mint tokens.
    ///
    /// 0. [writable] The mint account
    /// 1. [writable] The account to mint tokens to
    /// 2. [] The interchain token PDA associated with the mint
    /// 3. [] The token manager PDA
    /// 4. [signer] The minter account
    /// 5. [] The token program id
    MintInterchainToken {
        /// The amount of tokens to mint.
        amount: u64,
    },

    /// Transfers mintership to another account.
    ///
    /// 0. [] ITS root PDA.
    /// 1. [] System program account.
    /// 2. [writable, signer] Payer account.
    /// 3. [] PDA for the payer roles on the resource which the mintership is being transferred
    ///    from.
    /// 4. [] PDA for the resource.
    /// 5. [] Account to transfer mintership to.
    /// 6. [writable] PDA with the roles on the resource the
    ///    mintership is being transferred to.
    TransferInterchainTokenMintership,

    /// Proposes mintership transfer to another account.
    ///
    /// 0. [] System program account.
    /// 1. [writable, signer] Payer account.
    /// 2. [] PDA for the payer roles on the resource.
    /// 3. [] PDA for the resource.
    /// 4. [] Account to transfer operatorship to.
    /// 5. [writable] PDA with the roles on the resource for the accounts the
    ///    operatorship is being transferred to.
    /// 6. [] Account which the operatorship is being transferred from.
    /// 7. [writable] PDA with the roles on the resource for the account the
    ///    operatorship is being transferred from.
    /// 8. [writable] PDA for the proposal
    ProposeInterchainTokenMintership,

    /// Accepts mintership transfer from another account.
    ///
    /// 0. [] System program account.
    /// 1. [writable, signer] Payer account.
    /// 2. [] PDA for the payer roles on the resource.
    /// 3. [] PDA for the resource.
    /// 4. [] Account to transfer operatorship to.
    /// 5. [writable] PDA with the roles on the resource for the accounts the
    ///    operatorship is being transferred to.
    /// 6. [] Account which the operatorship is being transferred from.
    /// 7. [writable] PDA with the roles on the resource for the account the
    ///    operatorship is being transferred from.
    /// 8. [writable] PDA for the proposal
    AcceptInterchainTokenMintership,
}

#[test]
fn test_discriminator() {
    let ix = InterchainTokenServiceInstruction::TransferOperatorship;
    println!("Disc: {:?}", ix.discriminator());
}
