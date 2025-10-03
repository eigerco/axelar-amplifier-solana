//! Program state processor
use axelar_solana_encoding::types::messages::Message;
use axelar_solana_gateway::executable::validate_with_gmp_metadata;
use axelar_solana_gateway::state::message_payload::ImmutMessagePayload;
use interchain_token_transfer_gmp::{GMPPayload, SendToHub};
use itertools::{self, Itertools};
use program_utils::pda::BorshPda;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::invoke;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;

use crate::accounts::CallContractAccounts;
use crate::accounts::ExecuteAccounts;
use crate::instruction;
use crate::processor::interchain_token;
use crate::processor::interchain_transfer::process_inbound_transfer;
use crate::processor::link_token;
use crate::state::token_manager::TokenManager;
use crate::state::InterchainTokenService;
use crate::{
    assert_its_not_paused, assert_valid_its_root_pda, check_program_account, ITS_HUB_CHAIN_NAME,
};

pub(crate) fn process_execute<'a>(
    accounts: &'a [AccountInfo<'a>],
    message: Message,
) -> ProgramResult {
    let its_execute_accounts = ExecuteAccounts::try_from(accounts)?;
    validate_with_gmp_metadata(
        &its_execute_accounts.gateway_validation_accounts(),
        &message,
    )?;

    let its_root_config = InterchainTokenService::load(its_execute_accounts.its_root_pda)?;
    assert_valid_its_root_pda(its_execute_accounts.its_root_pda, its_root_config.bump)?;
    assert_its_not_paused(&its_root_config)?;

    if message.source_address != its_root_config.its_hub_address {
        msg!("Untrusted source address: {}", message.source_address);
        return Err(ProgramError::InvalidInstructionData);
    }

    let payload_account_data = its_execute_accounts
        .gateway_payload_account
        .try_borrow_data()?;
    let message_payload: ImmutMessagePayload<'_> = (**payload_account_data).try_into()?;

    let GMPPayload::ReceiveFromHub(inner) = GMPPayload::decode(message_payload.raw_payload)
        .map_err(|_err| ProgramError::InvalidInstructionData)?
    else {
        msg!("Unsupported GMP payload");
        return Err(ProgramError::InvalidInstructionData);
    };

    if !its_root_config.is_trusted_chain(&inner.source_chain) {
        msg!("Untrusted source chain: {}", inner.source_chain);
        return Err(ProgramError::InvalidInstructionData);
    }

    let payload =
        GMPPayload::decode(&inner.payload).map_err(|_err| ProgramError::InvalidInstructionData)?;

    validate_its_accounts(&its_execute_accounts.its_accounts(), &payload)?;

    match payload {
        GMPPayload::InterchainTransfer(transfer) => process_inbound_transfer(
            its_execute_accounts.try_into()?,
            message,
            &transfer,
            inner.source_chain,
        ),
        GMPPayload::DeployInterchainToken(deploy) => interchain_token::process_inbound_deploy(
            its_execute_accounts.try_into()?,
            deploy.token_id.0,
            deploy.name,
            deploy.symbol,
            deploy.decimals,
            0,
        ),
        GMPPayload::LinkToken(payload) => {
            link_token::process_inbound(its_execute_accounts.try_into()?, &payload)
        }
        GMPPayload::SendToHub(_)
        | GMPPayload::ReceiveFromHub(_)
        | GMPPayload::RegisterTokenMetadata(_) => Err(ProgramError::InvalidInstructionData),
    }
}

pub(crate) fn process_call_contract<'a>(
    payer: &'a AccountInfo<'a>,
    accounts: &CallContractAccounts<'a>,
    payload: &GMPPayload,
    destination_chain: String,
    gas_value: u64,
    signing_pda_bump: u8,
    wrapped: bool,
) -> ProgramResult {
    let its_root_config = InterchainTokenService::load(accounts.its_root_account)?;
    assert_valid_its_root_pda(accounts.its_root_account, its_root_config.bump)?;
    assert_its_not_paused(&its_root_config)?;

    check_program_account(*accounts.program_account.key)?;

    if !its_root_config.is_trusted_chain(&destination_chain)
        && destination_chain != ITS_HUB_CHAIN_NAME
    {
        msg!("Untrusted destination chain: {}", destination_chain);
        return Err(ProgramError::InvalidInstructionData);
    }

    let signing_pda =
        axelar_solana_gateway::create_call_contract_signing_pda(crate::ID, signing_pda_bump)?;

    if signing_pda.ne(accounts.call_contract_signing_account.key) {
        msg!("invalid call contract signing account / signing pda bump");
        return Err(ProgramError::InvalidAccountData);
    }

    let payload = if wrapped {
        GMPPayload::SendToHub(SendToHub {
            selector: SendToHub::MESSAGE_TYPE_ID
                .try_into()
                .map_err(|_err| ProgramError::ArithmeticOverflow)?,
            destination_chain,
            payload: payload.encode().into(),
        })
        .encode()
    } else {
        payload.encode()
    };

    let payload_hash = solana_program::keccak::hashv(&[&payload]).to_bytes();
    let call_contract_ix = axelar_solana_gateway::instructions::call_contract(
        axelar_solana_gateway::id(),
        *accounts.gateway_root_account.key,
        crate::ID,
        Some((signing_pda, signing_pda_bump)),
        crate::ITS_HUB_CHAIN_NAME.to_owned(),
        its_root_config.its_hub_address.clone(),
        payload,
    )?;

    if gas_value > 0 {
        pay_gas(
            payer,
            accounts.gas_service_config_account,
            accounts.gas_service_event_authority,
            accounts.system_program,
            payload_hash,
            its_root_config.its_hub_address,
            gas_value,
        )?;
    }

    invoke_signed(
        &call_contract_ix,
        &[
            accounts.program_account.clone(),
            accounts.call_contract_signing_account.clone(),
            accounts.gateway_root_account.clone(),
            accounts.gateway_event_authority.clone(),
        ],
        &[&[
            axelar_solana_gateway::seed_prefixes::CALL_CONTRACT_SIGNING_SEED,
            &[signing_pda_bump],
        ]],
    )?;

    Ok(())
}

fn pay_gas<'a>(
    payer: &'a AccountInfo<'a>,
    gas_service_config: &'a AccountInfo<'a>,
    gas_service_event_authority: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
    payload_hash: [u8; 32],
    its_hub_address: String,
    gas_value: u64,
) -> ProgramResult {
    let gas_payment_ix =
        axelar_solana_gas_service::instructions::pay_native_for_contract_call_instruction(
            payer.key,
            crate::ITS_HUB_CHAIN_NAME.to_owned(),
            its_hub_address,
            payload_hash,
            *payer.key,
            gas_value,
        )?;

    invoke(
        &gas_payment_ix,
        &[
            payer.clone(),
            gas_service_config.clone(),
            system_program.clone(),
            gas_service_event_authority.clone(),
        ],
    )
}

fn validate_its_accounts(accounts: &[AccountInfo<'_>], payload: &GMPPayload) -> ProgramResult {
    const TOKEN_MANAGER_PDA_INDEX: usize = 2;
    const TOKEN_MINT_INDEX: usize = 3;
    const TOKEN_PROGRAM_INDEX: usize = 5;

    // In this case we cannot derive the mint account, so we just use what we got
    // and check later against the mint within the `TokenManager` PDA.
    let maybe_mint = if let GMPPayload::InterchainTransfer(_) = payload {
        accounts.get(TOKEN_MINT_INDEX).map(|account| *account.key)
    } else {
        None
    };

    let token_program = accounts
        .get(TOKEN_PROGRAM_INDEX)
        .map(|account| *account.key)
        .ok_or(ProgramError::InvalidAccountData)?;

    let derived_its_accounts =
        instruction::derive_its_accounts(payload, token_program, maybe_mint)?;

    for element in accounts.iter().zip_longest(derived_its_accounts.iter()) {
        match element {
            itertools::EitherOrBoth::Both(provided, derived) => {
                if provided.key != &derived.pubkey {
                    msg!("Mismatch");
                    return Err(ProgramError::InvalidAccountData);
                }
            }
            itertools::EitherOrBoth::Left(_) | itertools::EitherOrBoth::Right(_) => {
                msg!("Size mismatch");
                return Err(ProgramError::InvalidAccountData);
            }
        }
    }

    // Now we validate the mint account passed for `InterchainTransfer`
    if let Some(mint) = maybe_mint {
        let token_manager_pda = accounts
            .get(TOKEN_MANAGER_PDA_INDEX)
            .ok_or(ProgramError::InvalidAccountData)?;

        let token_manager = TokenManager::load(token_manager_pda)?;

        if token_manager.token_address.as_ref() != mint.as_ref() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    Ok(())
}
