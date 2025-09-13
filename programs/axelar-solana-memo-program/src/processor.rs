//! Program state processor

use axelar_solana_encoding::types::messages::Message;
use axelar_solana_gateway::executable::{
    validate_message, AxelarMessagePayload, PROGRAM_ACCOUNTS_START_INDEX,
};
use axelar_solana_gateway::state::message_payload::ImmutMessagePayload;
use axelar_solana_its::executable::{
    AxelarInterchainTokenExecutablePayload, MaybeAxelarInterchainTokenExecutablePayload,
};
use borsh::BorshDeserialize;
use mpl_token_metadata::accounts::Metadata;
use program_utils::{check_program_account, pda::ValidPDA};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::{msg, system_program};
use std::str::FromStr;
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

    if let Some(payload) = input
        .try_get_axelar_interchain_token_executable_payload(accounts)
        .transpose()?
    {
        msg!("Instruction: AxelarInterchainTokenExecute");
        return process_message_from_axelar_with_token(program_id, accounts, &payload);
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
    payload: &AxelarInterchainTokenExecutablePayload,
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
    msg!("amount: {}", payload.amount);
    msg!("symbol: {}", token_metadata.symbol);
    msg!("name: {}", token_metadata.name);
    msg!(
        "payload source address: {}",
        hex::encode(&payload.source_address)
    );

    let instruction: AxelarMemoInstruction = borsh::from_slice(&payload.data)?;

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
    let message_payload_account = &protocol_accounts[1];
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
                ],
                &[&[
                    axelar_solana_gateway::seed_prefixes::CALL_CONTRACT_SIGNING_SEED,
                    &[signing_pda.1],
                ]],
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
        AxelarMemoInstruction::SendInterchainTransfer {
            token_id,
            destination_chain,
            destination_address,
            amount,
            mint,
            token_program,
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
                mint,
                token_program,
                gas_value,
            )?
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

/// Process the SendInterchainTransfer instruction
/// 
/// This function demonstrates PDA-based interchain token transfers by:
/// 1. Validating inputs and PDA state
/// 2. Transferring tokens from the memo program's PDA to the payer
/// 3. Making a CPI call to ITS to initiate the interchain transfer
/// 
/// # Arguments
/// * `program_id` - The memo program ID
/// * `accounts` - Account context including PDAs, ATAs, and ITS accounts
/// * `token_id` - The interchain token ID to transfer
/// * `destination_chain` - Target blockchain name
/// * `destination_address` - Recipient address on target chain
/// * `amount` - Token amount to transfer (must be > 0)
/// * `mint` - Token mint address
/// * `token_program` - SPL Token program ID
/// * `gas_value` - Gas fee for cross-chain transaction
/// 
/// # Errors
/// Returns `ProgramError::InvalidInstructionData` for invalid inputs
pub fn process_send_interchain_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    token_id: [u8; 32],
    destination_chain: String,
    destination_address: Vec<u8>,
    amount: u64,
    mint: Pubkey,
    token_program: Pubkey,
    gas_value: u128,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_pda = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    
    if amount == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }
    if destination_chain.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    if destination_address.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    
    // Validate that the counter PDA is properly initialized
    let counter_account = counter_pda.check_initialized_pda::<Counter>(program_id)?;
    assert_counter_pda_seeds(&counter_account, counter_pda.key);
    
    msg!("Memo program initiating interchain transfer from PDA: {}", counter_pda.key);
    msg!("Transfer amount: {}, destination: {}", amount, destination_chain);
    msg!("Total accounts passed to memo program: {}", accounts.len());
    
    msg!("Initiating interchain transfer directly from memo PDA");
    
    // Manually construct the instruction since the helper function incorrectly marks the wallet as a signer
    use borsh::to_vec;
    use solana_program::instruction::{AccountMeta, Instruction};
    
    let (gateway_root_pda, _) = axelar_solana_gateway::get_gateway_root_config_pda();
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let (token_manager_pda, _) = axelar_solana_its::find_token_manager_pda(&its_root_pda, &token_id);
    let token_manager_ata = spl_associated_token_account::get_associated_token_address_with_program_id(&token_manager_pda, &mint, &token_program);
    let source_ata = spl_associated_token_account::get_associated_token_address_with_program_id(counter_pda.key, &mint, &token_program);
    let (call_contract_signing_pda, signing_pda_bump) = axelar_solana_gateway::get_call_contract_signing_pda(axelar_solana_its::ID);
    // Gas service config PDA - we derive it manually since we don't have the gas service crate
    // This matches what the gas service uses: find_program_address(&[b"config"], &gas_service_id)
    let gas_service_id = Pubkey::from_str("GasPTBV8jYURiUWYnL8a2wNqkBB7fBHqBUr1JcpEDKHC").unwrap();
    let (gas_config_pda, _) = Pubkey::find_program_address(&[b"config"], &gas_service_id);
    
    // The accounts array passed to this function has:
    // [0] = counter_pda
    // [1] = payer  
    // [2] = payer (again for ITS)
    // [3] = wallet placeholder (but we want to use counter_pda)
    // [4] = payer_ata (not what we want)
    // [5] = memo_ata (THIS is what we want as source!)
    // [6] = mint
    // [7] = token_manager_pda
    // [8] = token_manager_ata
    // [9] = token_program
    // [10] = gateway_root_pda
    // [11] = gateway_program
    // [12] = gas_config_pda
    // [13] = gas_service_program
    // [14] = system_program
    // [15] = its_root_pda
    // [16] = call_contract_signing_pda
    // [17] = its_program
    //
    // For PDA transfer, we need:
    // - payer (signer)
    // - wallet = counter_pda (NOT signer, but this is the source of funds)
    // - source_ata = memo_ata (the PDA's ATA)
    let transfer_accounts = vec![
        AccountMeta::new(*accounts[1].key, true), // payer is signer
        AccountMeta::new(*accounts[0].key, false), // wallet = counter_pda (NOT a signer, but writable since it owns tokens)
        AccountMeta::new(*accounts[5].key, false), // source_ata = memo_ata (PDA's ATA at index 5)
        AccountMeta::new(*accounts[6].key, false), // mint
        AccountMeta::new(*accounts[7].key, false), // token_manager_pda
        AccountMeta::new(*accounts[8].key, false), // token_manager_ata
        AccountMeta::new_readonly(*accounts[9].key, false), // token_program
        AccountMeta::new_readonly(*accounts[10].key, false), // gateway_root_pda
        AccountMeta::new_readonly(*accounts[11].key, false), // gateway_program
        AccountMeta::new(*accounts[12].key, false), // gas_config_pda
        AccountMeta::new_readonly(*accounts[13].key, false), // gas_service_program
        AccountMeta::new_readonly(*accounts[14].key, false), // system_program
        AccountMeta::new_readonly(*accounts[15].key, false), // its_root_pda
        AccountMeta::new_readonly(*accounts[16].key, false), // call_contract_signing_pda
        AccountMeta::new_readonly(*accounts[17].key, false), // its_program
    ];
    
    let data = to_vec(&axelar_solana_its::instruction::InterchainTokenServiceInstruction::InterchainTransfer {
        token_id,
        destination_chain: destination_chain.clone(),
        destination_address,
        amount,
        gas_value: gas_value.try_into().map_err(|_| ProgramError::InvalidInstructionData)?,
        signing_pda_bump,
        pda_program_id: Some(crate::ID), // Memo program ID
        pda_seeds: Some(vec![]), // No seeds since counter PDA is derived with find_program_address(&[], program_id)
    })?;
    
    let transfer_ix = Instruction {
        program_id: axelar_solana_its::ID,
        accounts: transfer_accounts,
        data,
    };
    
    // Verify PDA derivation and get the correct bump
    let (expected_counter_pda, expected_bump) = Pubkey::find_program_address(&[], &crate::ID);
    msg!("Expected counter PDA: {}, bump: {}", expected_counter_pda, expected_bump);
    msg!("Actual counter PDA: {}, match: {}", accounts[0].key, expected_counter_pda == *accounts[0].key);
    msg!("Counter account bump: {}, expected bump: {}", counter_account.bump, expected_bump);
    
    if expected_counter_pda != *accounts[0].key {
        msg!("ERROR: Counter PDA mismatch!");
        return Err(ProgramError::InvalidArgument);
    }
    
    // TODO This is awful.
    // Make the CPI call to ITS
    // We need to pass accounts in the same order as transfer_accounts expects them
    let invoke_accounts = [
        accounts[1].clone(),    // payer
        accounts[0].clone(),    // counter_pda (wallet)
        accounts[5].clone(),    // memo_ata (source_ata)
        accounts[6].clone(),    // mint
        accounts[7].clone(),    // token_manager_pda
        accounts[8].clone(),    // token_manager_ata
        accounts[9].clone(),    // token_program
        accounts[10].clone(),   // gateway_root_pda
        accounts[11].clone(),   // gateway_program
        accounts[12].clone(),   // gas_config_pda
        accounts[13].clone(),   // gas_service_program
        accounts[14].clone(),   // system_program
        accounts[15].clone(),   // its_root_pda
        accounts[16].clone(),   // call_contract_signing_pda
        accounts[17].clone(),   // its_program
    ];
    
    // Use invoke_signed since we're using a PDA (counter_pda) that we own
    // The counter PDA is derived with empty seeds and the memo program ID
    invoke_signed(
        &transfer_ix,
        &invoke_accounts,
        &[&[&[expected_bump]]], // Signer seeds: [bump] for the counter PDA (derived with empty seeds + bump)
    )?;
    
    msg!("Interchain transfer initiated successfully");
    Ok(())
}

