//! Program state processor

use borsh::{self, BorshDeserialize};
use program_utils::{check_program_account, pda::ValidPDA};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::{msg, system_program};

use crate::assert_counter_pda_seeds;
use crate::instruction::CpiCallerInstruction;
use crate::state::Counter;

/// Instruction processor
pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    input: &[u8],
) -> ProgramResult {
    check_program_account(program_id, crate::check_id)?;

    msg!("Instruction: Native");
    let instruction = CpiCallerInstruction::try_from_slice(input)?;
    process_native_ix(program_id, accounts, instruction)
}

/// Process a native instruction submitted by another program or user ON the
/// Solana network
pub fn process_native_ix(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    payload: CpiCallerInstruction,
) -> ProgramResult {
    let _account_info_iter = &mut accounts.iter();

    match payload {
        CpiCallerInstruction::Initialize { counter_pda_bump } => {
            msg!("Instruction: Initialize");
            process_initialize_cpi_caller_counter(program_id, accounts, counter_pda_bump)?;
        }
        CpiCallerInstruction::SendInterchainTransfer {
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
        CpiCallerInstruction::SendInterchainTransferWithWrongSeeds {
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
        CpiCallerInstruction::CallContractWithInterchainToken {
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
    }

    Ok(())
}

/// This function is used to initialize the program.
pub fn process_initialize_cpi_caller_counter(
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
    // Check: CPI caller counter PDA Account is not initialized
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

/// Process SendInterchainTransfer instruction - initiates an interchain token transfer
/// from the CPI caller's PDA
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

    // Pass the CPI caller program ID as the source so events show the CPI caller is the source address
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
        ],
        &[&[&[counter_bump]]],
    )?;

    msg!("Interchain transfer initiated from CPI caller PDA");
    msg!("Token ID: {:?}", token_id);
    msg!("Destination chain: {}", destination_chain);
    msg!("Amount: {}", amount);

    Ok(())
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

    // Intentionally use wrong seeds to test validation
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
        ],
        &[&[&[counter_bump]]],
    )?;

    msg!("SendInterchainTransferWithWrongSeeds attempted from CPI caller");

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
        ],
        &[&[&[counter_bump]]],
    )?;

    msg!("CallContractWithInterchainToken initiated from CPI caller PDA");
    msg!("Token ID: {:?}", token_id);
    msg!("Destination chain: {}", destination_chain);
    msg!("Amount: {}", amount);
    msg!("Data length: {}", data.len());

    Ok(())
}
