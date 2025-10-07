//! Processor for [`TokenManager`] related requests.

use event_utils::Event as _;
use program_utils::{
    pda::BorshPda, validate_rent_key, validate_spl_associated_token_account_key,
    validate_system_account_key,
};
use role_management::processor::{
    ensure_signer_roles, RoleAddAccounts, RoleRemoveAccounts, RoleTransferWithProposalAccounts,
};
use role_management::state::UserRoles;
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::invoke;
use solana_program::program_error::ProgramError;
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::check_spl_token_program_account;
use spl_token_2022::extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions};
use spl_token_2022::instruction::AuthorityType;
use spl_token_2022::state::Mint;

use crate::state::token_manager::{self, TokenManager};
use crate::state::InterchainTokenService;
use crate::{assert_valid_its_root_pda, events, Validate};
use crate::{assert_valid_token_manager_pda, seed_prefixes, FromAccountInfoSlice, Roles};

pub(crate) fn set_flow_limit<'a>(
    payer: &'a AccountInfo<'a>,
    token_manager_pda: &'a AccountInfo<'a>,
    its_root_pda: &'a AccountInfo<'a>,
    system_account: &'a AccountInfo<'a>,
    flow_limit: Option<u64>,
) -> ProgramResult {
    let mut token_manager = TokenManager::load(token_manager_pda)?;
    assert_valid_token_manager_pda(
        token_manager_pda,
        its_root_pda.key,
        &token_manager.token_id,
        token_manager.bump,
    )?;
    token_manager.flow_slot.flow_limit = flow_limit;
    token_manager.store(payer, token_manager_pda, system_account)?;

    // TODO: Current implementation doesn't support Option<T>. When updating the eventss to be emitted
    // through CPI, we need to emit this events.

    // events::FlowLimitSet {
    //     token_id: token_manager.token_id,
    //     operator: *accounts.flow_limiter.key,
    //     flow_limit,
    // }
    // .emit();

    Ok(())
}

pub(crate) struct DeployTokenManagerInternal {
    manager_type: token_manager::Type,
    token_id: [u8; 32],
    token_address: Pubkey,
    operator: Option<Pubkey>,
    minter: Option<Pubkey>,
}

impl DeployTokenManagerInternal {
    pub(crate) const fn new(
        manager_type: token_manager::Type,
        token_id: [u8; 32],
        token_address: Pubkey,
        operator: Option<Pubkey>,
        minter: Option<Pubkey>,
    ) -> Self {
        Self {
            manager_type,
            token_id,
            token_address,
            operator,
            minter,
        }
    }
}

/// Deploys a new [`TokenManager`] PDA.
///
/// # Errors
///
/// An error occurred when deploying the [`TokenManager`] PDA. The reason can be
/// derived from the logs.
pub(crate) fn deploy<'a>(
    accounts: &DeployTokenManagerAccounts<'a>,
    deploy_token_manager: &DeployTokenManagerInternal,
    token_manager_pda_bump: u8,
) -> ProgramResult {
    msg!("Instruction: TM Deploy");
    validate_mint_extensions(deploy_token_manager.manager_type, accounts.token_mint)?;

    crate::create_associated_token_account_idempotent(
        accounts.payer,
        accounts.token_mint,
        accounts.token_manager_ata,
        accounts.token_manager_pda,
        accounts.system_account,
        accounts.token_program,
    )?;

    if let Some(operator_from_message) = deploy_token_manager.operator {
        let (Some(operator), Some(operator_roles_pda)) =
            (accounts.operator, accounts.operator_roles_pda)
        else {
            return Err(ProgramError::InvalidArgument);
        };

        if operator_from_message.ne(operator.key) {
            msg!("Invalid operator provided");
            return Err(ProgramError::InvalidAccountData);
        }

        let mut roles = Roles::OPERATOR | Roles::FLOW_LIMITER;
        if deploy_token_manager.minter.is_some()
            && deploy_token_manager.manager_type == token_manager::Type::NativeInterchainToken
        {
            roles |= Roles::MINTER;
        }

        setup_roles(
            accounts.payer,
            accounts.token_manager_pda,
            operator.key,
            operator_roles_pda,
            accounts.system_account,
            roles,
        )?;
    }

    let token_manager = TokenManager::new(
        deploy_token_manager.manager_type,
        deploy_token_manager.token_id,
        deploy_token_manager.token_address,
        *accounts.token_manager_ata.key,
        token_manager_pda_bump,
    );
    token_manager.init(
        &crate::id(),
        accounts.system_account,
        accounts.payer,
        accounts.token_manager_pda,
        &[
            seed_prefixes::TOKEN_MANAGER_SEED,
            accounts.its_root_pda.key.as_ref(),
            &token_manager.token_id,
            &[token_manager.bump],
        ],
    )?;

    events::TokenManagerDeployed {
        token_id: deploy_token_manager.token_id,
        token_manager: *accounts.token_manager_pda.key,
        token_manager_type: deploy_token_manager.manager_type.into(),
        params: deploy_token_manager
            .operator
            .map(|op| op.to_bytes().to_vec())
            .unwrap_or_default(),
    }
    .emit();

    Ok(())
}

fn setup_roles<'a>(
    payer: &AccountInfo<'a>,
    token_manager_pda: &AccountInfo<'a>,
    user: &Pubkey,
    user_roles_pda: &AccountInfo<'a>,
    system_account: &AccountInfo<'a>,
    roles: Roles,
) -> ProgramResult {
    let (derived_user_roles_pda, user_roles_pda_bump) =
        role_management::find_user_roles_pda(&crate::id(), token_manager_pda.key, user);

    if derived_user_roles_pda.ne(user_roles_pda.key) {
        msg!("Invalid user roles PDA provided");
        return Err(ProgramError::InvalidAccountData);
    }

    if let Ok(mut existing_roles) = UserRoles::<Roles>::load(user_roles_pda) {
        existing_roles.add(roles);
        existing_roles.store(payer, user_roles_pda, system_account)?;
    } else {
        let user_roles = UserRoles::new(roles, user_roles_pda_bump);
        user_roles.init(
            &crate::id(),
            system_account,
            payer,
            user_roles_pda,
            &[
                role_management::seed_prefixes::USER_ROLES_SEED,
                token_manager_pda.key.as_ref(),
                user.as_ref(),
                &[user_roles_pda_bump],
            ],
        )?;
    }

    Ok(())
}

pub(crate) fn validate_mint_extensions(
    ty: token_manager::Type,
    token_mint: &AccountInfo<'_>,
) -> ProgramResult {
    let mint_data = token_mint.try_borrow_data()?;
    let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;

    if matches!(
        (
            ty,
            mint.get_extension_types()?
                .contains(&ExtensionType::TransferFeeConfig)
        ),
        (token_manager::Type::LockUnlock, true) | (token_manager::Type::LockUnlockFee, false)
    ) {
        msg!("The mint is not compatible with the type");
        return Err(ProgramError::InvalidInstructionData);
    }

    Ok(())
}

pub(crate) fn validate_token_manager_type(
    ty: token_manager::Type,
    token_mint: &AccountInfo<'_>,
    token_manager_pda: &AccountInfo<'_>,
) -> ProgramResult {
    let mint_data = token_mint.try_borrow_data()?;
    let mint = Mint::unpack_from_slice(&mint_data)?;

    match (mint.mint_authority, ty) {
        (
            COption::None,
            token_manager::Type::NativeInterchainToken
            | token_manager::Type::MintBurn
            | token_manager::Type::MintBurnFrom,
        ) => {
            msg!("Mint authority is required for the given token manager type");
            Err(ProgramError::InvalidInstructionData)
        }
        (
            COption::Some(key),
            token_manager::Type::NativeInterchainToken
            | token_manager::Type::MintBurn
            | token_manager::Type::MintBurnFrom,
        ) if &key != token_manager_pda.key => {
            msg!("TokenManager is not the mint authority, which is required for this token manager type");
            Err(ProgramError::InvalidInstructionData)
        }
        _ => Ok(()),
    }
}

pub(crate) fn handover_mint_authority(
    accounts: &[AccountInfo<'_>],
    token_id: [u8; 32],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;
    let mint = next_account_info(accounts_iter)?;
    let its_root = next_account_info(accounts_iter)?;
    let token_manager = next_account_info(accounts_iter)?;
    let minter_roles = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;

    msg!("Instruction: HandoverMintAuthority");

    validate_system_account_key(system_account.key)?;
    spl_token_2022::check_spl_token_program_account(token_program.key)?;

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let its_root_config = InterchainTokenService::load(its_root)?;
    assert_valid_its_root_pda(its_root, its_root_config.bump)?;

    let token_manager_config = TokenManager::load(token_manager)?;
    assert_valid_token_manager_pda(
        token_manager,
        its_root.key,
        &token_id,
        token_manager_config.bump,
    )?;

    if !matches!(
        token_manager_config.ty,
        token_manager::Type::MintBurn | token_manager::Type::MintBurnFrom
    ) {
        msg!("Invalid TokenManager type for instruction");
        return Err(ProgramError::InvalidAccountData);
    }

    if token_program.key != mint.owner {
        return Err(ProgramError::InvalidAccountData);
    }

    if token_manager_config.token_address != *mint.key {
        msg!("TokenManager PDA does not match the provided Mint account");
        return Err(ProgramError::InvalidAccountData);
    }

    let maybe_mint_authority = {
        let mint_data = mint.try_borrow_data()?;
        let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;

        mint.base.mint_authority
    };

    match maybe_mint_authority {
        COption::None => {
            msg!("Cannot hand over mint authority of a TokenManager for non-mintable token");
            Err(ProgramError::InvalidArgument)
        }
        COption::Some(mint_authority) if mint_authority == *authority.key => {
            // The given authority is the mint authority. The mint authority needs to be transferred
            // to the `TokenManager` and the `minter` role is added to the payer
            // on the `TokenManager`. Future minting by the user needs to go
            // through ITS.
            let authority_transfer_ix = spl_token_2022::instruction::set_authority(
                token_program.key,
                mint.key,
                Some(token_manager.key),
                AuthorityType::MintTokens,
                authority.key,
                &[],
            )?;

            invoke(&authority_transfer_ix, &[mint.clone(), authority.clone()])?;

            setup_roles(
                payer,
                token_manager,
                authority.key,
                minter_roles,
                system_account,
                Roles::MINTER,
            )?;

            Ok(())
        }
        COption::Some(_) => {
            msg!("Signer is not the mint authority");
            Err(ProgramError::InvalidArgument)
        }
    }?;

    Ok(())
}

#[derive(Debug)]
pub(crate) struct DeployTokenManagerAccounts<'a> {
    pub(crate) payer: &'a AccountInfo<'a>,
    pub(crate) system_account: &'a AccountInfo<'a>,
    pub(crate) its_root_pda: &'a AccountInfo<'a>,
    pub(crate) token_manager_pda: &'a AccountInfo<'a>,
    pub(crate) token_mint: &'a AccountInfo<'a>,
    pub(crate) token_manager_ata: &'a AccountInfo<'a>,
    pub(crate) token_program: &'a AccountInfo<'a>,
    pub(crate) ata_program: &'a AccountInfo<'a>,
    pub(crate) rent_sysvar: &'a AccountInfo<'a>,
    pub(crate) operator: Option<&'a AccountInfo<'a>>,
    pub(crate) operator_roles_pda: Option<&'a AccountInfo<'a>>,
}

impl Validate for DeployTokenManagerAccounts<'_> {
    fn validate(&self) -> Result<(), ProgramError> {
        validate_system_account_key(self.system_account.key)?;
        check_spl_token_program_account(self.token_program.key)?;
        validate_spl_associated_token_account_key(self.ata_program.key)?;
        validate_rent_key(self.rent_sysvar.key)?;

        if !self.payer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if self.token_program.key != self.token_mint.owner {
            msg!("Mint and program account mismatch");
            return Err(ProgramError::IncorrectProgramId);
        }

        if &get_associated_token_address_with_program_id(
            self.token_manager_pda.key,
            self.token_mint.key,
            self.token_program.key,
        ) != self.token_manager_ata.key
        {
            msg!("Wrong ata account key");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

impl<'a> FromAccountInfoSlice<'a> for DeployTokenManagerAccounts<'a> {
    type Context = Option<&'a AccountInfo<'a>>;

    fn extract_accounts(
        accounts: &'a [AccountInfo<'a>],
        maybe_payer: &Self::Context,
    ) -> Result<Self, ProgramError>
    where
        Self: Sized + Validate,
    {
        let accounts_iter = &mut accounts.iter();
        let payer = if let Some(payer) = maybe_payer {
            payer
        } else {
            next_account_info(accounts_iter)?
        };

        Ok(Self {
            payer,
            system_account: next_account_info(accounts_iter)?,
            its_root_pda: next_account_info(accounts_iter)?,
            token_manager_pda: next_account_info(accounts_iter)?,
            token_mint: next_account_info(accounts_iter)?,
            token_manager_ata: next_account_info(accounts_iter)?,
            token_program: next_account_info(accounts_iter)?,
            ata_program: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            operator: next_account_info(accounts_iter).ok(),
            operator_roles_pda: next_account_info(accounts_iter).ok(),
        })
    }
}

pub(crate) fn process_add_flow_limiter<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    msg!("Instruction: AddTokenManagerFlowLimiter");

    let accounts_iter = &mut accounts.iter();
    let its_config_pda = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let adder_user_account = next_account_info(accounts_iter)?;
    let adder_roles_account = next_account_info(accounts_iter)?;
    let resource = next_account_info(accounts_iter)?;
    let destination_user_account = next_account_info(accounts_iter)?;
    let destination_roles_account = next_account_info(accounts_iter)?;

    validate_system_account_key(system_account.key)?;
    let its_config = InterchainTokenService::load(its_config_pda)?;
    assert_valid_its_root_pda(its_config_pda, its_config.bump)?;
    if resource.key == its_config_pda.key {
        msg!("Resource is not a TokenManager");
        return Err(ProgramError::InvalidAccountData);
    }

    let token_manager = TokenManager::load(resource)?;
    assert_valid_token_manager_pda(
        resource,
        its_config_pda.key,
        &token_manager.token_id,
        token_manager.bump,
    )?;

    let role_management_accounts = RoleAddAccounts {
        system_account,
        payer,
        authority_user_account: adder_user_account,
        authority_roles_account: adder_roles_account,
        resource,
        target_user_account: destination_user_account,
        target_roles_account: destination_roles_account,
    };

    role_management::processor::add(
        &crate::id(),
        role_management_accounts,
        Roles::FLOW_LIMITER,
        Roles::OPERATOR,
    )
}

pub(crate) fn process_remove_flow_limiter<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    msg!("Instruction: RemoveTokenManagerFlowLimiter");

    let accounts_iter = &mut accounts.iter();
    let its_config_pda = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let remover_user_account = next_account_info(accounts_iter)?;
    let remover_roles_account = next_account_info(accounts_iter)?;
    let resource = next_account_info(accounts_iter)?;
    let origin_user_account = next_account_info(accounts_iter)?;
    let origin_roles_account = next_account_info(accounts_iter)?;

    validate_system_account_key(system_account.key)?;
    let its_config = InterchainTokenService::load(its_config_pda)?;
    assert_valid_its_root_pda(its_config_pda, its_config.bump)?;
    if resource.key == its_config_pda.key {
        msg!("Resource is not a TokenManager");
        return Err(ProgramError::InvalidAccountData);
    }

    let token_manager = TokenManager::load(resource)?;
    assert_valid_token_manager_pda(
        resource,
        its_config_pda.key,
        &token_manager.token_id,
        token_manager.bump,
    )?;

    let role_management_accounts = RoleRemoveAccounts {
        system_account,
        payer,
        authority_user_account: remover_user_account,
        authority_roles_account: remover_roles_account,
        resource,
        target_user_account: origin_user_account,
        target_roles_account: origin_roles_account,
    };

    role_management::processor::remove(
        &crate::id(),
        role_management_accounts,
        Roles::FLOW_LIMITER,
        Roles::OPERATOR,
    )
}

pub(crate) fn process_set_flow_limit<'a>(
    accounts: &'a [AccountInfo<'a>],
    flow_limit: Option<u64>,
) -> ProgramResult {
    msg!("Instruction: SetTokenManagerFlowLimit");

    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let flow_limiter = next_account_info(accounts_iter)?;
    let its_root_pda = next_account_info(accounts_iter)?;
    let token_manager_pda = next_account_info(accounts_iter)?;
    let token_manager_user_roles_pda = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;

    let its_config_pda = InterchainTokenService::load(its_root_pda)?;
    assert_valid_its_root_pda(its_root_pda, its_config_pda.bump)?;

    validate_system_account_key(system_account.key)?;

    ensure_signer_roles(
        &crate::id(),
        token_manager_pda,
        flow_limiter,
        token_manager_user_roles_pda,
        Roles::FLOW_LIMITER,
    )?;

    set_flow_limit(
        payer,
        token_manager_pda,
        its_root_pda,
        system_account,
        flow_limit,
    )
}

pub(crate) fn process_transfer_operatorship<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    msg!("Instruction: TransferTokenManagerOperatorship");

    let accounts_iter = &mut accounts.iter();
    let its_config_pda = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let origin_user_account = next_account_info(accounts_iter)?;
    let origin_roles_account = next_account_info(accounts_iter)?;
    let token_manager_account = next_account_info(accounts_iter)?;
    let destination_user_account = next_account_info(accounts_iter)?;
    let destination_roles_account = next_account_info(accounts_iter)?;

    validate_system_account_key(system_account.key)?;

    if origin_user_account.key == destination_user_account.key {
        msg!("Source and destination accounts are the same");
        return Err(ProgramError::InvalidArgument);
    }

    let its_config = InterchainTokenService::load(its_config_pda)?;
    let token_manager = TokenManager::load(token_manager_account)?;

    assert_valid_its_root_pda(its_config_pda, its_config.bump)?;
    assert_valid_token_manager_pda(
        token_manager_account,
        its_config_pda.key,
        &token_manager.token_id,
        token_manager.bump,
    )?;

    let role_add_accounts = RoleAddAccounts {
        system_account,
        payer,
        authority_user_account: origin_user_account,
        authority_roles_account: origin_roles_account,
        resource: token_manager_account,
        target_user_account: destination_user_account,
        target_roles_account: destination_roles_account,
    };
    let role_remove_accounts = RoleRemoveAccounts {
        system_account,
        payer,
        authority_user_account: origin_user_account,
        authority_roles_account: origin_roles_account,
        resource: token_manager_account,
        target_user_account: origin_user_account,
        target_roles_account: origin_roles_account,
    };

    role_management::processor::add(
        &crate::id(),
        role_add_accounts,
        Roles::OPERATOR,
        Roles::OPERATOR,
    )?;

    role_management::processor::remove(
        &crate::id(),
        role_remove_accounts,
        Roles::OPERATOR,
        Roles::OPERATOR,
    )
}

pub(crate) fn process_propose_operatorship<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    msg!("Instruction: ProposeTokenManagerOperatorship");

    let accounts_iter = &mut accounts.iter();
    let its_config_pda = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let origin_user_account = next_account_info(accounts_iter)?;
    let origin_roles_account = next_account_info(accounts_iter)?;
    let token_manager_account = next_account_info(accounts_iter)?;
    let destination_user_account = next_account_info(accounts_iter)?;
    let destination_roles_account = next_account_info(accounts_iter)?;
    let proposal_account = next_account_info(accounts_iter)?;

    validate_system_account_key(system_account.key)?;

    let role_management_accounts = RoleTransferWithProposalAccounts {
        system_account,
        payer,
        resource: token_manager_account,
        destination_user_account,
        destination_roles_account,
        origin_user_account,
        origin_roles_account,
        proposal_account,
    };

    let its_config = InterchainTokenService::load(its_config_pda)?;
    assert_valid_its_root_pda(its_config_pda, its_config.bump)?;
    let token_manager = TokenManager::load(role_management_accounts.resource)?;
    assert_valid_token_manager_pda(
        role_management_accounts.resource,
        its_config_pda.key,
        &token_manager.token_id,
        token_manager.bump,
    )?;

    role_management::processor::propose(&crate::id(), role_management_accounts, Roles::OPERATOR)
}

pub(crate) fn process_accept_operatorship<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    msg!("Instruction: AcceptTokenManagerOperatorship");

    let accounts_iter = &mut accounts.iter();
    let its_config_pda = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let destination_user_account = next_account_info(accounts_iter)?;
    let destination_roles_account = next_account_info(accounts_iter)?;
    let token_manager_account = next_account_info(accounts_iter)?;
    let origin_user_account = next_account_info(accounts_iter)?;
    let origin_roles_account = next_account_info(accounts_iter)?;
    let proposal_account = next_account_info(accounts_iter)?;

    validate_system_account_key(system_account.key)?;

    let role_management_accounts = RoleTransferWithProposalAccounts {
        system_account,
        payer,
        resource: token_manager_account,
        destination_user_account,
        destination_roles_account,
        origin_user_account,
        origin_roles_account,
        proposal_account,
    };

    let its_config = InterchainTokenService::load(its_config_pda)?;
    assert_valid_its_root_pda(its_config_pda, its_config.bump)?;
    let token_manager = TokenManager::load(role_management_accounts.resource)?;
    assert_valid_token_manager_pda(
        role_management_accounts.resource,
        its_config_pda.key,
        &token_manager.token_id,
        token_manager.bump,
    )?;

    role_management::processor::accept(&crate::id(), role_management_accounts, Roles::OPERATOR)
}
