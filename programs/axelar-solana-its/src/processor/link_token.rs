//! This module is responsible for functions related to custom token linking

use event_cpi_macros::{emit_cpi, event_cpi_accounts};
use interchain_token_transfer_gmp::{GMPPayload, LinkToken, RegisterTokenMetadata};
use program_utils::pda::BorshPda;
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::set_return_data;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use spl_token_2022::extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions};
use spl_token_2022::state::Mint;

use crate::accounts::{
    CallContractAccounts, DeployCanonicalTokenAccounts, DeployCustomTokenAccounts,
    DeployTokenManagerAccounts,
};
use crate::processor::gmp;
use crate::processor::interchain_token;
use crate::processor::token_manager::DeployTokenManagerInternal;
use crate::state::token_manager::TokenManager;
use crate::state::{token_manager, InterchainTokenService};
use crate::{
    assert_its_not_paused, assert_valid_its_root_pda, assert_valid_token_manager_pda, events,
};
use event_cpi::EventAccounts;

pub(crate) fn process_inbound<'a>(
    accounts: DeployTokenManagerAccounts<'a>,
    payload: &LinkToken,
) -> ProgramResult {
    let token_manager_type: token_manager::Type = payload.token_manager_type.try_into()?;
    if token_manager::Type::NativeInterchainToken == token_manager_type {
        return Err(ProgramError::InvalidInstructionData);
    }

    let token_address = Pubkey::new_from_array(
        payload
            .destination_token_address
            .as_ref()
            .try_into()
            .map_err(|_err| ProgramError::InvalidAccountData)?,
    );
    let operator = match payload.link_params.as_ref().try_into() {
        Ok(operator_bytes) => Some(Pubkey::new_from_array(operator_bytes)),
        Err(_err) => None,
    };

    let deploy_token_manager = DeployTokenManagerInternal::new(
        payload.token_manager_type.try_into()?,
        payload.token_id.0,
        token_address,
        operator,
        None,
    );

    let its_root_pda_bump = InterchainTokenService::load(accounts.its_root_pda)?.bump;

    assert_valid_its_root_pda(accounts.its_root_pda, its_root_pda_bump)?;

    let (_, token_manager_pda_bump) =
        crate::find_token_manager_pda(accounts.its_root_pda.key, payload.token_id.as_ref());

    crate::processor::token_manager::deploy(
        &accounts,
        &deploy_token_manager,
        token_manager_pda_bump,
    )
}

pub(crate) fn process_outbound<'a>(
    accounts: &'a [AccountInfo<'a>],
    salt: [u8; 32],
    destination_chain: String,
    destination_token_address: Vec<u8>,
    token_manager_type: token_manager::Type,
    link_params: Vec<u8>,
    gas_value: u64,
    signing_pda_bump: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let deployer = next_account_info(accounts_iter)?;
    let token_manager_account = next_account_info(accounts_iter)?;
    let gmp_accounts = CallContractAccounts::try_from(accounts_iter.as_slice())?;

    let its_root_config = InterchainTokenService::load(gmp_accounts.its_root_account)?;
    assert_valid_its_root_pda(gmp_accounts.its_root_account, its_root_config.bump)?;
    if destination_chain == its_root_config.chain_name {
        msg!("Cannot link to another token on the same chain");
        return Err(ProgramError::InvalidInstructionData);
    }

    if !payer.is_signer {
        msg!("Payer should be signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !deployer.is_signer {
        msg!("Deployer should be signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("Instruction: ProcessOutbound");
    let deploy_salt = crate::linked_token_deployer_salt(deployer.key, &salt);
    let token_id = crate::interchain_token_id_internal(&deploy_salt);

    let event_accounts_iter = &mut gmp_accounts.event_accounts().into_iter();
    event_cpi_accounts!(event_accounts_iter);

    emit_cpi!(events::InterchainTokenIdClaimed {
        token_id,
        deployer: *deployer.key,
        salt: deploy_salt,
    });

    let token_manager = TokenManager::load(token_manager_account)?;

    assert_valid_token_manager_pda(
        token_manager_account,
        gmp_accounts.its_root_account.key,
        &token_id,
        token_manager.bump,
    )?;

    let link_started_events = events::LinkTokenStarted {
        token_id,
        destination_chain,
        source_token_address: token_manager.token_address,
        destination_token_address,
        token_manager_type: token_manager_type.into(),
        params: link_params,
    };
    emit_cpi!(link_started_events);

    let message = GMPPayload::LinkToken(LinkToken {
        selector: LinkToken::MESSAGE_TYPE_ID
            .try_into()
            .map_err(|_err| ProgramError::ArithmeticOverflow)?,
        token_id: token_id.into(),
        token_manager_type: token_manager_type.into(),
        source_token_address: token_manager.token_address.to_bytes().into(),
        destination_token_address: link_started_events.destination_token_address.into(),
        link_params: link_started_events.params.into(),
    });

    gmp::process_call_contract(
        payer,
        &gmp_accounts,
        &message,
        link_started_events.destination_chain,
        gas_value,
        signing_pda_bump,
        true,
    )?;

    set_return_data(&token_id);

    Ok(())
}

pub(crate) fn register_token_metadata<'a>(
    accounts: &'a [AccountInfo<'a>],
    gas_value: u64,
    signing_pda_bump: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let mint_account = next_account_info(accounts_iter)?;

    let gmp_accounts = CallContractAccounts::try_from(accounts_iter.as_slice())?;
    msg!("Instruction: RegisterTokenMetadata");

    let event_accounts_iter = &mut gmp_accounts.event_accounts().into_iter();
    event_cpi_accounts!(event_accounts_iter);

    let mint_data = mint_account.try_borrow_data()?;
    let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;
    let payload = GMPPayload::RegisterTokenMetadata(RegisterTokenMetadata {
        selector: RegisterTokenMetadata::MESSAGE_TYPE_ID
            .try_into()
            .map_err(|_err| ProgramError::ArithmeticOverflow)?,
        token_address: mint_account.key.to_bytes().into(),
        decimals: mint.base.decimals,
    });

    emit_cpi!(events::TokenMetadataRegistered {
        token_address: *mint_account.key,
        decimals: mint.base.decimals,
    });

    gmp::process_call_contract(
        payer,
        &gmp_accounts,
        &payload,
        crate::ITS_HUB_CHAIN_NAME.to_owned(),
        gas_value,
        signing_pda_bump,
        false,
    )
}

pub(crate) fn register_custom_token<'a>(
    accounts: &'a [AccountInfo<'a>],
    salt: [u8; 32],
    token_manager_type: token_manager::Type,
    operator: Option<Pubkey>,
) -> ProgramResult {
    if token_manager_type == token_manager::Type::NativeInterchainToken {
        return Err(ProgramError::InvalidInstructionData);
    }

    let custom_accounts = DeployCustomTokenAccounts::try_from(accounts)?;

    msg!("Instruction: RegisterCustomToken");

    let its_config = InterchainTokenService::load(custom_accounts.its_root_pda)?;
    assert_valid_its_root_pda(custom_accounts.its_root_pda, its_config.bump)?;
    assert_its_not_paused(&its_config)?;

    let deployer = *custom_accounts.deployer.key;
    let deploy_salt = crate::linked_token_deployer_salt(&deployer, &salt);

    register_token(
        custom_accounts.try_into()?,
        token_manager_type,
        deployer,
        operator,
        deploy_salt,
    )
}

pub(crate) fn register_canonical_interchain_token<'a>(
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let accounts = DeployCanonicalTokenAccounts::try_from(accounts)?;

    msg!("Instruction: RegisterCanonicalInterchainToken");

    let its_config = InterchainTokenService::load(accounts.its_root_pda)?;
    assert_valid_its_root_pda(accounts.its_root_pda, its_config.bump)?;
    assert_its_not_paused(&its_config)?;

    if let Err(_err) =
        interchain_token::get_token_metadata(accounts.token_mint, Some(accounts.metadata_account))
    {
        return Err(ProgramError::InvalidAccountData);
    }

    let mint_data = accounts.token_mint.try_borrow_data()?;
    let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;
    let has_fee_extension = mint
        .get_extension_types()?
        .contains(&ExtensionType::TransferFeeConfig);

    let token_manager_type = if has_fee_extension {
        token_manager::Type::LockUnlockFee
    } else {
        token_manager::Type::LockUnlock
    };

    let deploy_salt = crate::canonical_interchain_token_deploy_salt(accounts.token_mint.key);

    register_token(
        accounts.try_into()?,
        token_manager_type,
        crate::ID,
        None,
        deploy_salt,
    )
}

fn register_token<'a>(
    accounts: DeployTokenManagerAccounts<'a>,
    token_manager_type: token_manager::Type,
    deployer: Pubkey,
    operator: Option<Pubkey>,
    deploy_salt: [u8; 32],
) -> ProgramResult {
    let event_accounts_iter = &mut accounts.event_accounts().into_iter();
    event_cpi_accounts!(event_accounts_iter);

    let token_id = crate::interchain_token_id_internal(&deploy_salt);
    let (_, token_manager_pda_bump) =
        crate::find_token_manager_pda(accounts.its_root_pda.key, &token_id);
    crate::assert_valid_token_manager_pda(
        accounts.token_manager_pda,
        accounts.its_root_pda.key,
        &token_id,
        token_manager_pda_bump,
    )?;

    emit_cpi!(events::InterchainTokenIdClaimed {
        token_id,
        deployer: deployer,
        salt: deploy_salt,
    });

    let deploy_token_manager = DeployTokenManagerInternal::new(
        token_manager_type,
        token_id,
        *accounts.token_mint.key,
        operator,
        None,
    );

    crate::processor::token_manager::deploy(
        &accounts,
        &deploy_token_manager,
        token_manager_pda_bump,
    )?;

    set_return_data(&token_id);

    Ok(())
}
