//! # `InterchainTokenService` program
use bitflags::bitflags;
use borsh::{BorshDeserialize, BorshSerialize};
use program_utils::ensure_single_feature;
use program_utils::pda::BorshPda;
use program_utils::pda::ValidPDA;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::invoke;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use state::interchain_transfer_execute::InterchainTransferExecute;
use state::InterchainTokenService;

mod entrypoint;
pub mod event;
pub mod executable;
pub mod instruction;
pub mod processor;
pub mod state;

ensure_single_feature!("devnet-amplifier", "stagenet", "testnet", "mainnet");

#[cfg(feature = "devnet-amplifier")]
solana_program::declare_id!("itsqybuNsChBo3LgVhCWWnTJVJdoVTUJaodmqQcG6z7");

#[cfg(feature = "stagenet")]
solana_program::declare_id!("itsediSVCwwKc6UuxfrsEiF8AEuEFk34RFAscPEDEpJ");

#[cfg(feature = "testnet")]
solana_program::declare_id!("itsZEirFsnRmLejCsRRNZKHqWTzMsKGyYi6Qr962os4");

#[cfg(feature = "mainnet")]
solana_program::declare_id!("its1111111111111111111111111111111111111111");

pub(crate) const ITS_HUB_CHAIN_NAME: &str = "axelar";

pub(crate) trait Validate {
    fn validate(&self) -> Result<(), ProgramError>;
}

pub(crate) trait FromAccountInfoSlice<'a> {
    type Context;

    fn from_account_info_slice(
        accounts: &'a [AccountInfo<'a>],
        context: &Self::Context,
    ) -> Result<Self, ProgramError>
    where
        Self: Sized + Validate,
    {
        let obj = Self::extract_accounts(accounts, context)?;
        obj.validate()?;
        Ok(obj)
    }

    fn extract_accounts(
        accounts: &'a [AccountInfo<'a>],
        context: &Self::Context,
    ) -> Result<Self, ProgramError>
    where
        Self: Sized + Validate;
}

/// Seed prefixes for different PDAs initialized by the program
pub mod seed_prefixes {
    /// The seed prefix for deriving the ITS root PDA
    pub const ITS_SEED: &[u8] = b"interchain-token-service";

    /// The seed prefix for deriving the token manager PDA
    pub const TOKEN_MANAGER_SEED: &[u8] = b"token-manager";

    /// The seed prefix for deriving the interchain token PDA
    pub const INTERCHAIN_TOKEN_SEED: &[u8] = b"interchain-token";

    /// The seed prefix for deriving an interchain token id
    pub const PREFIX_INTERCHAIN_TOKEN_ID: &[u8] = b"interchain-token-id";

    /// The seed prefix for deriving an interchain token salt
    pub const PREFIX_INTERCHAIN_TOKEN_SALT: &[u8] = b"interchain-token-salt";

    /// The seed prefix for deriving an interchain token id for a canonical token
    pub const PREFIX_CANONICAL_TOKEN_SALT: &[u8] = b"canonical-token-salt";

    /// The seed prefix for deriving an interchain token id for a canonical token
    pub const PREFIX_CUSTOM_TOKEN_SALT: &[u8] = b"solana-custom-token-salt";

    /// The seed prefix for deriving the flow slot PDA
    pub const FLOW_SLOT_SEED: &[u8] = b"flow-slot";

    /// The seed prefix for deriving the deployment approval PDA
    pub const DEPLOYMENT_APPROVAL_SEED: &[u8] = b"deployment-approval";

    /// The seed prefix for deriving the deployment approval PDA
    pub const INTERCHAIN_TRANSFER_EXECUTE_SEED: &[u8] = b"interchain-transfer-execute";
}

bitflags! {
    /// Roles that can be assigned to a user.
    #[derive(Debug, Eq, PartialEq, Clone, Copy)]
    pub struct Roles: u8 {
        /// Can mint new tokens.
        const MINTER = 0b0000_0001;

        /// Can perform operations on the resource.
        const OPERATOR = 0b0000_0010;

        /// Can change the limit to the flow of tokens.
        const FLOW_LIMITER = 0b0000_0100;
    }
}

impl PartialEq<u8> for Roles {
    fn eq(&self, other: &u8) -> bool {
        self.bits().eq(other)
    }
}

impl PartialEq<Roles> for u8 {
    fn eq(&self, other: &Roles) -> bool {
        self.eq(&other.bits())
    }
}

impl BorshSerialize for Roles {
    fn serialize<W: std::io::prelude::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.bits().serialize(writer)
    }
}

impl BorshDeserialize for Roles {
    fn deserialize_reader<R: std::io::prelude::Read>(reader: &mut R) -> std::io::Result<Self> {
        let byte = u8::deserialize_reader(reader)?;
        Ok(Self::from_bits_truncate(byte))
    }
}

/// Checks that the supplied program ID is the correct one
///
/// # Errors
///
/// If the program ID passed doesn't match the current program ID
#[inline]
pub fn check_program_account(program_id: Pubkey) -> ProgramResult {
    if program_id != crate::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Tries to create the ITS root PDA using the provided bump, falling back to
/// `find_program_address` if the bump is `None` or invalid.
///
/// # Errors
///
/// If the bump is invalid.
pub fn its_root_pda(maybe_bump: Option<u8>) -> Result<(Pubkey, u8), ProgramError> {
    if let Some(bump) = maybe_bump {
        create_its_root_pda(bump).map(|pubkey| (pubkey, bump))
    } else {
        Ok(find_its_root_pda())
    }
}

/// Tries to create the ITS root PDA using the provided bump, falling back to
/// `find_program_address` if the bump invalid.
///
/// # Errors
///
/// If the bump is invalid.
pub fn create_its_root_pda(bump: u8) -> Result<Pubkey, ProgramError> {
    Ok(Pubkey::create_program_address(
        &[seed_prefixes::ITS_SEED, &[bump]],
        &crate::id(),
    )?)
}

/// Derives interchain token service root PDA
#[inline]
#[must_use]
pub fn find_its_root_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[seed_prefixes::ITS_SEED], &crate::id())
}

pub(crate) fn assert_valid_its_root_pda(
    its_root_pda_account: &AccountInfo<'_>,
    canonical_bump: u8,
) -> ProgramResult {
    let expected_its_root_pda = create_its_root_pda(canonical_bump)?;

    if expected_its_root_pda.ne(its_root_pda_account.key) {
        msg!("Invalid ITS root PDA provided");
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}

pub(crate) fn assert_its_not_paused(its_config: &InterchainTokenService) -> ProgramResult {
    if its_config.paused {
        msg!("The Interchain Token Service is currently paused.");
        return Err(ProgramError::Immutable);
    }

    Ok(())
}

/// Tries to create the PDA for a [`Tokenmanager`] using the provided bump,
/// falling back to `find_program_address` if the bump is `None` or invalid.
///
/// # Errors
///
/// If the bump is invalid.
pub fn token_manager_pda(
    its_root_pda: &Pubkey,
    token_id: &[u8; 32],
    maybe_bump: Option<u8>,
) -> Result<(Pubkey, u8), ProgramError> {
    if let Some(bump) = maybe_bump {
        create_token_manager_pda(its_root_pda, token_id, bump).map(|pubkey| (pubkey, bump))
    } else {
        Ok(find_token_manager_pda(its_root_pda, token_id))
    }
}

/// Tries to create the PDA for a [`Tokenmanager`] using the provided bump,
/// falling back to `find_program_address` if the bump is invalid.
///
/// # Errors
///
/// If the bump is invalid.
pub fn create_token_manager_pda(
    its_root_pda: &Pubkey,
    token_id: &[u8; 32],
    bump: u8,
) -> Result<Pubkey, ProgramError> {
    Ok(Pubkey::create_program_address(
        &[
            seed_prefixes::TOKEN_MANAGER_SEED,
            its_root_pda.as_ref(),
            token_id,
            &[bump],
        ],
        &crate::id(),
    )?)
}

/// Derives the PDA for a [`TokenManager`].
#[inline]
#[must_use]
pub fn find_token_manager_pda(its_root_pda: &Pubkey, token_id: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            seed_prefixes::TOKEN_MANAGER_SEED,
            its_root_pda.as_ref(),
            token_id,
        ],
        &crate::id(),
    )
}

pub(crate) fn assert_valid_token_manager_pda(
    token_manager_pda_account: &AccountInfo<'_>,
    its_root_pda: &Pubkey,
    token_id: &[u8; 32],
    canonical_bump: u8,
) -> ProgramResult {
    let expected_token_manager_pda =
        create_token_manager_pda(its_root_pda, token_id, canonical_bump)?;
    if expected_token_manager_pda.ne(token_manager_pda_account.key) {
        msg!("Invalid TokenManager PDA provided");
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}

pub(crate) fn assert_valid_deploy_approval_pda(
    deploy_approval_pda_account: &AccountInfo<'_>,
    minter: &Pubkey,
    token_id: &[u8; 32],
    destination_chain: &str,
    canonical_bump: u8,
) -> ProgramResult {
    let expected_deploy_approval_pda =
        create_deployment_approval_pda(minter, token_id, destination_chain, canonical_bump)?;

    if expected_deploy_approval_pda.ne(deploy_approval_pda_account.key) {
        msg!("Invalid DeploymentApproval PDA provided");
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}

/// Tries to create the PDA for an `InterchainToken` using the provided bump,
/// falling back to `find_program_address` if the bump is `None` or invalid.
///
/// The Interchain Token PDA is used as the mint account for native Interchain Tokens
///
/// # Errors
///
/// If the bump is invalid.
pub fn interchain_token_pda(
    its_root_pda: &Pubkey,
    token_id: &[u8],
    maybe_bump: Option<u8>,
) -> Result<(Pubkey, u8), ProgramError> {
    if let Some(bump) = maybe_bump {
        create_interchain_token_pda(its_root_pda, token_id, bump).map(|pubkey| (pubkey, bump))
    } else {
        Ok(find_interchain_token_pda(its_root_pda, token_id))
    }
}

/// Tries to create the PDA for an `InterchainToken` using the provided bump,
/// falling back to `find_program_address` if the bump is invalid.
///
/// The Interchain Token PDA is used as the mint account for native Interchain Tokens
///
/// # Errors
///
/// If the bump is invalid.
#[inline]
pub fn create_interchain_token_pda(
    its_root_pda: &Pubkey,
    token_id: &[u8],
    bump: u8,
) -> Result<Pubkey, ProgramError> {
    Ok(Pubkey::create_program_address(
        &[
            seed_prefixes::INTERCHAIN_TOKEN_SEED,
            its_root_pda.as_ref(),
            token_id,
            &[bump],
        ],
        &crate::id(),
    )?)
}

/// Derives the PDA for an interchain token account.
///
/// The Interchain Token PDA is used as the mint account for native Interchain Tokens
#[inline]
#[must_use]
pub fn find_interchain_token_pda(its_root_pda: &Pubkey, token_id: &[u8]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            seed_prefixes::INTERCHAIN_TOKEN_SEED,
            its_root_pda.as_ref(),
            token_id,
        ],
        &crate::id(),
    )
}

/// Tries to create the PDA for a `FlowSlot` using the provided bump,
/// falling back to `find_program_address` if the bump is invalid.
///
/// # Errors
///
/// If the bump is invalid.
#[inline]
pub fn create_flow_slot_pda(
    token_manager_pda: &Pubkey,
    epoch: u64,
    bump: u8,
) -> Result<Pubkey, ProgramError> {
    Ok(Pubkey::create_program_address(
        &[
            seed_prefixes::FLOW_SLOT_SEED,
            token_manager_pda.as_ref(),
            &epoch.to_ne_bytes(),
            &[bump],
        ],
        &crate::id(),
    )?)
}

/// Derives the PDA for a `FlowSlot`.
#[inline]
#[must_use]
pub fn find_flow_slot_pda(token_manager_pda: &Pubkey, epoch: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            seed_prefixes::FLOW_SLOT_SEED,
            token_manager_pda.as_ref(),
            &epoch.to_ne_bytes(),
        ],
        &crate::id(),
    )
}

/// Tries to create the PDA for a `FlowSlot` using the provided bump,
/// falling back to `find_program_address` if the bump is `None` or invalid.
///
/// # Errors
///
/// If the bump is invalid.
pub fn flow_slot_pda(
    token_manager_pda: &Pubkey,
    epoch: u64,
    maybe_bump: Option<u8>,
) -> Result<(Pubkey, u8), ProgramError> {
    if let Some(bump) = maybe_bump {
        create_flow_slot_pda(token_manager_pda, epoch, bump).map(|pubkey| (pubkey, bump))
    } else {
        Ok(find_flow_slot_pda(token_manager_pda, epoch))
    }
}

pub(crate) fn assert_valid_flow_slot_pda(
    flow_slot_pda_account: &AccountInfo<'_>,
    token_manager_pda: &Pubkey,
    current_flow_epoch: u64,
    canonical_bump: u8,
) -> ProgramResult {
    let expected_flow_slot_pda =
        create_flow_slot_pda(token_manager_pda, current_flow_epoch, canonical_bump)?;

    if expected_flow_slot_pda.ne(flow_slot_pda_account.key) {
        msg!("Invalid flow limit slot PDA provided");
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}

/// Tries to create the PDA for a `DeploymentApproval` using the provided bump,
/// falling back to `find_program_address` if the bump is invalid.
///
/// # Errors
///
/// If the bump is invalid.
#[inline]
pub fn create_deployment_approval_pda(
    minter: &Pubkey,
    token_id: &[u8],
    destination_chain: &str,
    bump: u8,
) -> Result<Pubkey, ProgramError> {
    Ok(Pubkey::create_program_address(
        &[
            seed_prefixes::DEPLOYMENT_APPROVAL_SEED,
            minter.as_ref(),
            token_id,
            destination_chain.as_bytes(),
            &[bump],
        ],
        &crate::id(),
    )?)
}

/// Derives the PDA for a `DeploymentApproval`.
#[inline]
#[must_use]
pub fn find_deployment_approval_pda(
    minter: &Pubkey,
    token_id: &[u8],
    destination_chain: &str,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            seed_prefixes::DEPLOYMENT_APPROVAL_SEED,
            minter.as_ref(),
            token_id,
            destination_chain.as_bytes(),
        ],
        &crate::id(),
    )
}

/// Tries to create the PDA for a `DeploymentApproval` using the provided bump,
/// falling back to `find_program_address` if the bump is `None` or invalid.
///
/// # Errors
///
/// If the bump is invalid.
pub fn deployment_approval_pda(
    minter: &Pubkey,
    token_id: &[u8],
    destination_chain: &str,
    maybe_bump: Option<u8>,
) -> Result<(Pubkey, u8), ProgramError> {
    if let Some(bump) = maybe_bump {
        create_deployment_approval_pda(minter, token_id, destination_chain, bump)
            .map(|pubkey| (pubkey, bump))
    } else {
        Ok(find_deployment_approval_pda(
            minter,
            token_id,
            destination_chain,
        ))
    }
}

/// Tries to create the PDA for a [`Tokenmanager`] using the provided bump,
/// falling back to `find_program_address` if the bump is `None` or invalid.
///
/// # Errors
///
/// If the bump is invalid.
pub fn interchain_transfer_execute_pda(
    its_root_pda: &Pubkey,
    destination_program: &Pubkey,
    maybe_bump: Option<u8>,
) -> Result<(Pubkey, u8), ProgramError> {
    if let Some(bump) = maybe_bump {
        create_interchain_transfer_execute_pda(its_root_pda, destination_program, bump)
            .map(|pubkey| (pubkey, bump))
    } else {
        Ok(find_interchain_transfer_execute_pda(
            its_root_pda,
            destination_program,
        ))
    }
}

/// Tries to create the PDA for a [`Tokenmanager`] using the provided bump,
/// falling back to `find_program_address` if the bump is invalid.
///
/// # Errors
///
/// If the bump is invalid.
pub fn create_interchain_transfer_execute_pda(
    its_root_pda: &Pubkey,
    destination_program: &Pubkey,
    bump: u8,
) -> Result<Pubkey, ProgramError> {
    Ok(Pubkey::create_program_address(
        &[
            seed_prefixes::INTERCHAIN_TRANSFER_EXECUTE_SEED,
            its_root_pda.as_ref(),
            &destination_program.to_bytes(),
            &[bump],
        ],
        &crate::id(),
    )?)
}

/// Derives the PDA for a [`TokenManager`].
#[inline]
#[must_use]
pub fn find_interchain_transfer_execute_pda(
    its_root_pda: &Pubkey,
    destination_program: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            seed_prefixes::INTERCHAIN_TRANSFER_EXECUTE_SEED,
            its_root_pda.as_ref(),
            destination_program.as_ref(),
        ],
        &crate::id(),
    )
}

/// Either create the interchain_transfer_execute pda or read it, and ensure it is derived properly.
pub(crate) fn assert_valid_interchain_transfer_execute_pda<'a>(
    interchain_transfer_execute_pda_account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_account: &AccountInfo<'a>,
    its_root_pda: &Pubkey,
    destination_program: &Pubkey,
) -> Result<u8, ProgramError> {
    let bump = if interchain_transfer_execute_pda_account.is_initialized_pda(&crate::id()) {
        let interchain_transfer_execute =
            InterchainTransferExecute::load(interchain_transfer_execute_pda_account)?;

        let expected_token_manager_pda = create_interchain_transfer_execute_pda(
            its_root_pda,
            destination_program,
            interchain_transfer_execute.bump,
        )?;
        if expected_token_manager_pda.ne(interchain_transfer_execute_pda_account.key) {
            msg!("Invalid InterchainTransferExecute PDA provided");
            return Err(ProgramError::InvalidArgument);
        }
        interchain_transfer_execute.bump
    } else {
        let (expected_token_manager_pda, bump) =
            find_interchain_transfer_execute_pda(its_root_pda, destination_program);
        if expected_token_manager_pda.ne(interchain_transfer_execute_pda_account.key) {
            msg!("Invalid InterchainTransferExecute PDA provided");
            return Err(ProgramError::InvalidArgument);
        }
        let interchain_transfer_execute = InterchainTransferExecute::new(bump);
        interchain_transfer_execute.init(
            &crate::id(),
            system_account,
            payer,
            interchain_transfer_execute_pda_account,
            &[
                seed_prefixes::INTERCHAIN_TRANSFER_EXECUTE_SEED,
                its_root_pda.as_ref(),
                destination_program.as_ref(),
                &[bump],
            ],
        )?;
        bump
    };

    Ok(bump)
}

/// Creates an associated token account for the given program address and token
/// mint, if it doesn't already exist.
///
/// # Errors
///
/// Returns an error if the account already exists, but with a different owner.
pub(crate) fn create_associated_token_account_idempotent<'a>(
    payer: &AccountInfo<'a>,
    token_mint_account: &AccountInfo<'a>,
    associated_token_account: &AccountInfo<'a>,
    wallet: &AccountInfo<'a>,
    system_account: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
) -> ProgramResult {
    let create_ata_ix =
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            payer.key,
            wallet.key,
            token_mint_account.key,
            token_program.key,
        );

    invoke(
        &create_ata_ix,
        &[
            payer.clone(),
            associated_token_account.clone(),
            wallet.clone(),
            token_mint_account.clone(),
            system_account.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}

#[must_use]
pub(crate) fn canonical_interchain_token_deploy_salt(mint: &Pubkey) -> [u8; 32] {
    solana_program::keccak::hashv(&[seed_prefixes::PREFIX_CANONICAL_TOKEN_SALT, mint.as_ref()])
        .to_bytes()
}

pub(crate) fn interchain_token_deployer_salt(deployer: &Pubkey, salt: &[u8; 32]) -> [u8; 32] {
    solana_program::keccak::hashv(&[
        seed_prefixes::PREFIX_INTERCHAIN_TOKEN_SALT,
        deployer.as_ref(),
        salt,
    ])
    .to_bytes()
}

pub(crate) fn linked_token_deployer_salt(deployer: &Pubkey, salt: &[u8; 32]) -> [u8; 32] {
    solana_program::keccak::hashv(&[
        seed_prefixes::PREFIX_CUSTOM_TOKEN_SALT,
        deployer.as_ref(),
        salt,
    ])
    .to_bytes()
}

pub(crate) fn interchain_token_id_internal(salt: &[u8; 32]) -> [u8; 32] {
    solana_program::keccak::hashv(&[seed_prefixes::PREFIX_INTERCHAIN_TOKEN_ID, salt]).to_bytes()
}

/// Calculates the tokenId that would correspond to a link for a given deployer
/// with a specified salt
#[must_use]
pub fn interchain_token_id(deployer: &Pubkey, salt: &[u8; 32]) -> [u8; 32] {
    let deploy_salt = interchain_token_deployer_salt(deployer, salt);

    interchain_token_id_internal(&deploy_salt)
}

/// Computes the ID for a canonical interchain token based on its address
#[must_use]
pub fn canonical_interchain_token_id(mint: &Pubkey) -> [u8; 32] {
    let salt = canonical_interchain_token_deploy_salt(mint);

    interchain_token_id_internal(&salt)
}

/// Computes the ID for a linked custom token based on its deployer and salt
#[must_use]
pub fn linked_token_id(deployer: &Pubkey, salt: &[u8; 32]) -> [u8; 32] {
    let salt = linked_token_deployer_salt(deployer, salt);

    interchain_token_id_internal(&salt)
}
