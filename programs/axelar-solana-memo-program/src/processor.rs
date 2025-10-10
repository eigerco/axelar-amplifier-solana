//! Program state processor

use axelar_solana_encoding::types::messages::Message;
use axelar_solana_gateway::executable::{
    validate_message, AxelarMessagePayload, PROGRAM_ACCOUNTS_START_INDEX,
};
use axelar_solana_gateway::state::message_payload::ImmutMessagePayload;
use axelar_solana_its::executable::{
    AxelarInterchainTokenExecuteInfo, MaybeAxelarInterchainTokenExecutablePayload,
};
use borsh::{self, BorshDeserialize};
use mpl_token_metadata::accounts::Metadata;
use program_utils::{check_program_account, pda::ValidPDA};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::{msg, system_program};
use std::str::from_utf8;

use crate::assert_counter_pda_seeds;
use crate::instruction::AxelarMemoInstruction;
use crate::state::Counter;

/// Instruction processor
pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    input: &[u8],
) -> ProgramResult {
    check_program_account(program_id, crate::check_id)?;

    if let Some(message) =
        axelar_solana_gateway::executable::parse_axelar_message(input).transpose()?
    {
        msg!("Instruction: AxelarExecute");
        return process_message_from_axelar(program_id, accounts, &message);
    }

    if let Some((execute_info, call_data)) = input
        .try_get_axelar_interchain_token_executable_payload(accounts)
        .transpose()?
    {
        msg!("Instruction: AxelarInterchainTokenExecute");
        return process_message_from_axelar_with_token(
            program_id,
            accounts,
            &execute_info,
            call_data,
        );
    }

    msg!("Instruction: Native");
    let instruction = AxelarMemoInstruction::try_from_slice(input)?;
    process_native_ix(program_id, accounts, instruction)
}

/// Process a message submitted by the relayer which originates from the Axelar
/// network
pub fn process_message_from_axelar_with_token<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    execute_info: &AxelarInterchainTokenExecuteInfo,
    call_data: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let _its_root_pda = next_account_info(accounts_iter)?;
    let _message_payload_account = next_account_info(accounts_iter)?;
    let _token_program = next_account_info(accounts_iter)?;
    let _token_mint = next_account_info(accounts_iter)?;
    let _ata_account = next_account_info(accounts_iter)?;
    let mpl_token_metadata_account = next_account_info(accounts_iter)?;
    let instruction_accounts = accounts_iter.as_slice();
    let token_metadata = Metadata::from_bytes(&mpl_token_metadata_account.try_borrow_data()?)?;

    msg!("Processing memo with tokens:");
    msg!("amount: {}", execute_info.amount);
    msg!("symbol: {}", token_metadata.symbol);
    msg!("name: {}", token_metadata.name);
    msg!(
        "payload source address: {}",
        hex::encode(&execute_info.source_address)
    );

    let instruction: AxelarMemoInstruction = borsh::from_slice(&call_data)?;

    process_native_ix(program_id, instruction_accounts, instruction)
}

/// Process a message submitted by the relayer which originates from the Axelar
/// network
pub fn process_message_from_axelar(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    message: &Message,
) -> ProgramResult {
    validate_message(accounts, message)?;
    let (protocol_accounts, accounts) = accounts.split_at(PROGRAM_ACCOUNTS_START_INDEX);

    // Access the payload from the MessagePayload account.
    // It should be considered safe otherwise `validate_message` would have reverted.
    let message_payload_account = &protocol_accounts[2];
    let account_data = message_payload_account.try_borrow_data()?;
    let message_payload: ImmutMessagePayload<'_> = (**account_data).try_into()?;
    let axelar_payload = AxelarMessagePayload::decode(message_payload.raw_payload)?;
    let payload = axelar_payload.payload_without_accounts();

    let memo = from_utf8(payload).map_err(|err| {
        msg!("Invalid UTF-8, from byte {}", err.valid_up_to());
        ProgramError::InvalidInstructionData
    })?;

    process_memo(program_id, accounts, memo)?;

    Ok(())
}

/// Process a native instruction submitted by another program or user ON the
/// Solana network
pub fn process_native_ix(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    payload: AxelarMemoInstruction,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    match payload {
        AxelarMemoInstruction::SendToGateway {
            memo,
            destination_chain,
            destination_address,
        } => {
            msg!("Instruction: SendToGateway");
            let program_account = next_account_info(account_info_iter)?;
            let counter_pda = next_account_info(account_info_iter)?;
            let signing_pda_acc = next_account_info(account_info_iter)?;
            let gateway_root_pda = next_account_info(account_info_iter)?;
            let gateway_event_authority = next_account_info(account_info_iter)?;

            let gateway_program = next_account_info(account_info_iter)?;

            let counter_pda_account = counter_pda.check_initialized_pda::<Counter>(program_id)?;
            let signing_pda = axelar_solana_gateway::get_call_contract_signing_pda(crate::ID);
            assert_counter_pda_seeds(&counter_pda_account, counter_pda.key);
            if &signing_pda.0 != signing_pda_acc.key {
                msg!("invalid signing PDA");
                return Err(ProgramError::InvalidAccountData);
            }
            invoke_signed(
                &axelar_solana_gateway::instructions::call_contract(
                    *gateway_program.key,
                    *gateway_root_pda.key,
                    crate::ID,
                    Some(signing_pda),
                    destination_chain,
                    destination_address,
                    memo.into_bytes(),
                )?,
                &[
                    program_account.clone(),
                    signing_pda_acc.clone(),
                    gateway_root_pda.clone(),
                    gateway_event_authority.clone(),
                    gateway_program.clone(),
                ],
                &[&[
                    axelar_solana_gateway::seed_prefixes::CALL_CONTRACT_SIGNING_SEED,
                    &[signing_pda.1],
                ]],
            )?;
        }
        AxelarMemoInstruction::SendInterchainTransfer {
            token_id,
            destination_chain,
            destination_address,
            amount,
            gas_value,
        } => {
            msg!("Instruction: SendInterchainTransfer");
            process_send_interchain_transfer(
                program_id,
                accounts,
                token_id,
                destination_chain,
                destination_address,
                amount,
                gas_value,
            )?;
        }
        AxelarMemoInstruction::SendInterchainTransferWithWrongSeeds {
            token_id,
            destination_chain,
            destination_address,
            amount,
            gas_value,
        } => {
            msg!("Instruction: SendInterchainTransferWithWrongSeeds");
            process_send_interchain_transfer_with_wrong_seeds(
                program_id,
                accounts,
                token_id,
                destination_chain,
                destination_address,
                amount,
                gas_value,
            )?;
        }
        AxelarMemoInstruction::CallContractWithInterchainToken {
            token_id,
            destination_chain,
            destination_address,
            amount,
            data,
            gas_value,
        } => {
            msg!("Instruction: CallContractWithInterchainToken");
            process_call_contract_with_interchain_token(
                program_id,
                accounts,
                token_id,
                destination_chain,
                destination_address,
                amount,
                data,
                gas_value,
            )?;
        }
        AxelarMemoInstruction::Initialize { counter_pda_bump } => {
            msg!("Instruction: Initialize");
            process_initialize_memo_program_counter(program_id, accounts, counter_pda_bump)?;
        }
        AxelarMemoInstruction::ProcessMemo { memo } => {
            msg!("Instruction: Process Memo");
            process_memo(program_id, accounts, &memo)?
        }
    }

    Ok(())
}

fn process_memo(program_id: &Pubkey, accounts: &[AccountInfo<'_>], memo: &str) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let counter_pda = next_account_info(account_info_iter)?;

    // Iterate over the rest of the provided accounts
    for account_info in account_info_iter {
        // NOTE: The accounts WILL NEVER be signers, but they MAY be writable
        msg!(
            "Provided account {:?}-{}-{}",
            account_info.key,
            account_info.is_signer,
            account_info.is_writable
        );
    }

    log_memo(memo);

    let mut counter_pda_account = counter_pda.check_initialized_pda::<Counter>(program_id)?;
    counter_pda_account.counter += 1;
    let mut data = counter_pda.try_borrow_mut_data()?;
    counter_pda_account.pack_into_slice(&mut data);

    Ok(())
}

#[inline]
fn log_memo(memo: &str) {
    // If memo is longer than 10 characters, log just the first character.
    let char_count = memo.chars().count();
    if char_count > 10 {
        msg!(
            "Memo (len {}): {:?} x {} (too big to log)",
            memo.len(),
            memo.chars().next().unwrap(),
            char_count
        );
    } else {
        msg!("Memo (len {}): {:?}", memo.len(), memo);
    }
}

/// Process SendInterchainTransfer instruction - initiates an interchain token transfer
/// from the memo program's PDA
pub fn process_send_interchain_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    token_id: [u8; 32],
    destination_chain: String,
    destination_address: Vec<u8>,
    amount: u64,
    gas_value: u128,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer = next_account_info(accounts_iter)?;
    let counter_pda = next_account_info(accounts_iter)?;
    let its_root_pda = next_account_info(accounts_iter)?;
    let token_manager_pda = next_account_info(accounts_iter)?;
    let source_ata = next_account_info(accounts_iter)?;
    let token_manager_ata = next_account_info(accounts_iter)?;
    let gateway_root_pda = next_account_info(accounts_iter)?;
    let gateway_program_account = next_account_info(accounts_iter)?;
    let gas_service_root_pda = next_account_info(accounts_iter)?;
    let gas_service_program_account = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let call_contract_signing_account = next_account_info(accounts_iter)?;
    let its_program_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let its_event_authority = next_account_info(accounts_iter)?;
    let gateway_event_authority = next_account_info(accounts_iter)?;
    let gas_service_event_authority = next_account_info(accounts_iter)?;

    let counter_pda_account = counter_pda.check_initialized_pda::<Counter>(program_id)?;
    assert_counter_pda_seeds(&counter_pda_account, counter_pda.key);
    let counter_bump = counter_pda_account.bump;

    let expected_source_ata =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            counter_pda.key,
            token_mint.key,
            token_program.key,
        );
    if source_ata.key != &expected_source_ata {
        msg!(
            "Invalid source ATA. Expected: {}, Got: {}",
            expected_source_ata,
            source_ata.key
        );
        return Err(ProgramError::InvalidAccountData);
    }

    // Pass the memo program ID as the source so events show the memo is the source address
    // The counter PDA is derived with empty seeds
    let pda_seeds = vec![];
    let transfer_ix = axelar_solana_its::instruction::cpi_interchain_transfer(
        *payer.key,
        *counter_pda.key,
        *source_ata.key,
        token_id,
        destination_chain.clone(),
        destination_address,
        amount,
        *token_mint.key,
        *token_program.key,
        gas_value
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
        crate::ID,
        pda_seeds,
    )?;

    invoke_signed(
        &transfer_ix,
        &[
            payer.clone(),
            counter_pda.clone(),
            source_ata.clone(),
            token_mint.clone(),
            token_manager_pda.clone(),
            token_manager_ata.clone(),
            token_program.clone(),
            gateway_root_pda.clone(),
            gateway_program_account.clone(),
            gas_service_root_pda.clone(),
            gas_service_program_account.clone(),
            system_program.clone(),
            its_root_pda.clone(),
            call_contract_signing_account.clone(),
            its_program_account.clone(),
            its_event_authority.clone(),
            gateway_event_authority.clone(),
            gas_service_event_authority.clone(),
        ],
        &[&[&[counter_bump]]],
    )?;

    msg!("Interchain transfer initiated from memo program PDA");
    msg!("Token ID: {:?}", token_id);
    msg!("Destination chain: {}", destination_chain);
    msg!("Amount: {}", amount);

    Ok(())
}

/// This function is used to initialize the program.
pub fn process_initialize_memo_program_counter(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    bump: u8,
) -> Result<(), ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let counter_pda = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;

    let counter = crate::state::Counter { counter: 0, bump };

    // Check: System Program Account
    if !system_program::check_id(system_account.key) {
        return Err(ProgramError::IncorrectProgramId);
    }
    // Check: Memo counter PDA Account is not initialized
    counter_pda.check_uninitialized_pda()?;
    // Check: counter PDA account uses the canonical bump.
    assert_counter_pda_seeds(&counter, counter_pda.key);

    program_utils::pda::init_pda(
        payer,
        counter_pda,
        program_id,
        system_account,
        counter,
        &[&[bump]],
    )
}

/// Process SendInterchainTransferWithWrongSeeds instruction - identical to the regular transfer
/// but uses wrong seeds to test validation logic
pub fn process_send_interchain_transfer_with_wrong_seeds(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    token_id: [u8; 32],
    destination_chain: String,
    destination_address: Vec<u8>,
    amount: u64,
    gas_value: u128,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer = next_account_info(accounts_iter)?;
    let counter_pda = next_account_info(accounts_iter)?;
    let its_root_pda = next_account_info(accounts_iter)?;
    let token_manager_pda = next_account_info(accounts_iter)?;
    let source_ata = next_account_info(accounts_iter)?;
    let token_manager_ata = next_account_info(accounts_iter)?;
    let gateway_root_pda = next_account_info(accounts_iter)?;
    let gateway_program_account = next_account_info(accounts_iter)?;
    let gas_service_root_pda = next_account_info(accounts_iter)?;
    let gas_service_program_account = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let call_contract_signing_account = next_account_info(accounts_iter)?;
    let its_program_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let its_event_authority = next_account_info(accounts_iter)?;
    let gateway_event_authority = next_account_info(accounts_iter)?;
    let gas_service_event_authority = next_account_info(accounts_iter)?;

    let counter_pda_account = counter_pda.check_initialized_pda::<Counter>(program_id)?;
    assert_counter_pda_seeds(&counter_pda_account, counter_pda.key);
    let counter_bump = counter_pda_account.bump;

    let expected_source_ata =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            counter_pda.key,
            token_mint.key,
            token_program.key,
        );
    if source_ata.key != &expected_source_ata {
        msg!(
            "Invalid source ATA. Expected: {}, Got: {}",
            expected_source_ata,
            source_ata.key
        );
        return Err(ProgramError::InvalidAccountData);
    }

    let wrong_pda_seeds = vec![b"wrong_seed".to_vec()];
    let transfer_ix = axelar_solana_its::instruction::cpi_interchain_transfer(
        *payer.key,
        *counter_pda.key,
        *source_ata.key,
        token_id,
        destination_chain.clone(),
        destination_address,
        amount,
        *token_mint.key,
        *token_program.key,
        gas_value
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
        crate::ID,
        wrong_pda_seeds,
    )?;

    invoke_signed(
        &transfer_ix,
        &[
            payer.clone(),
            counter_pda.clone(),
            source_ata.clone(),
            token_mint.clone(),
            token_manager_pda.clone(),
            token_manager_ata.clone(),
            token_program.clone(),
            gateway_root_pda.clone(),
            gateway_program_account.clone(),
            gas_service_root_pda.clone(),
            gas_service_program_account.clone(),
            system_program.clone(),
            its_root_pda.clone(),
            call_contract_signing_account.clone(),
            its_program_account.clone(),
            its_event_authority.clone(),
            gateway_event_authority.clone(),
            gas_service_event_authority.clone(),
        ],
        &[&[&[counter_bump]]],
    )?;

    Ok(())
}

/// Process CallContractWithInterchainToken instruction - sends tokens with additional data
#[allow(clippy::too_many_arguments)]
pub fn process_call_contract_with_interchain_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    token_id: [u8; 32],
    destination_chain: String,
    destination_address: Vec<u8>,
    amount: u64,
    data: Vec<u8>,
    gas_value: u128,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer = next_account_info(accounts_iter)?;
    let counter_pda = next_account_info(accounts_iter)?;
    let its_root_pda = next_account_info(accounts_iter)?;
    let token_manager_pda = next_account_info(accounts_iter)?;
    let source_ata = next_account_info(accounts_iter)?;
    let token_manager_ata = next_account_info(accounts_iter)?;
    let gateway_root_pda = next_account_info(accounts_iter)?;
    let gateway_program_account = next_account_info(accounts_iter)?;
    let gas_service_root_pda = next_account_info(accounts_iter)?;
    let gas_service_program_account = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let call_contract_signing_account = next_account_info(accounts_iter)?;
    let its_program_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let its_event_authority = next_account_info(accounts_iter)?;
    let gateway_event_authority = next_account_info(accounts_iter)?;
    let gas_service_event_authority = next_account_info(accounts_iter)?;

    let counter_pda_account = counter_pda.check_initialized_pda::<Counter>(program_id)?;
    assert_counter_pda_seeds(&counter_pda_account, counter_pda.key);
    let counter_bump = counter_pda_account.bump;

    let expected_source_ata =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            counter_pda.key,
            token_mint.key,
            token_program.key,
        );
    if source_ata.key != &expected_source_ata {
        msg!(
            "Invalid source ATA. Expected: {}, Got: {}",
            expected_source_ata,
            source_ata.key
        );
        return Err(ProgramError::InvalidAccountData);
    }

    // Use correct seeds for the counter PDA (empty seeds)
    let pda_seeds = vec![];
    let transfer_ix = axelar_solana_its::instruction::cpi_call_contract_with_interchain_token(
        *payer.key,
        *counter_pda.key,
        *source_ata.key,
        token_id,
        destination_chain.clone(),
        destination_address.clone(),
        amount,
        *token_mint.key,
        data.clone(),
        *token_program.key,
        gas_value
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
        crate::ID,
        pda_seeds,
    )?;

    invoke_signed(
        &transfer_ix,
        &[
            payer.clone(),
            counter_pda.clone(),
            source_ata.clone(),
            token_mint.clone(),
            token_manager_pda.clone(),
            token_manager_ata.clone(),
            token_program.clone(),
            gateway_root_pda.clone(),
            gateway_program_account.clone(),
            gas_service_root_pda.clone(),
            gas_service_program_account.clone(),
            system_program.clone(),
            its_root_pda.clone(),
            call_contract_signing_account.clone(),
            its_program_account.clone(),
            its_event_authority.clone(),
            gateway_event_authority.clone(),
            gas_service_event_authority.clone(),
        ],
        &[&[&[counter_bump]]],
    )?;

    msg!("CallContractWithInterchainToken initiated from memo program PDA");
    msg!("Token ID: {:?}", token_id);
    msg!("Destination chain: {}", destination_chain);
    msg!("Amount: {}", amount);
    msg!("Data length: {}", data.len());

    Ok(())
}
