//! Instruction module for the Axelar ITS CPI Caller.

use borsh::{to_vec, BorshDeserialize, BorshSerialize};
pub use solana_program;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;

/// Instructions supported by the Axelar ITS CPI Caller.
#[repr(u8)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum CpiCallerInstruction {
    /// Initialize the CPI caller by creating a counter PDA
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. [s] payer
    /// 1. [w] counter PDA
    /// 2. [] system program
    Initialize {
        /// The pda bump for the counter PDA
        counter_pda_bump: u8,
    },

    /// Send an interchain token transfer initiated by the CPI caller's PDA.
    /// The source token account (counter PDA's ATA) is automatically derived and verified.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. [w] CPI caller counter PDA (the actual token sender)
    /// 1. [] ITS root PDA
    /// 2. [] Token Manager PDA
    /// 3. [w] Source token account (counter PDA's ATA) - verified against derivation
    /// 4. [w] Token Manager's ATA
    /// 5. [] Gateway root PDA
    /// 6. [] Gateway program ID
    /// 7. [] Gas Service root PDA
    /// 8. [] Gas Service program ID
    /// 9. [] Token mint
    /// 10. [] Token program
    /// 11. [] Call contract signing PDA
    /// 12. [] ITS program ID
    /// 13. [] System program
    SendInterchainTransfer {
        /// Token ID for the transfer
        token_id: [u8; 32],
        /// Destination chain
        destination_chain: String,
        /// Destination address
        destination_address: Vec<u8>,
        /// Amount to transfer
        amount: u64,
        /// Gas value for the transfer
        gas_value: u128,
    },

    /// Send an interchain token transfer with intentionally wrong seeds (for testing)
    /// This instruction is used to test the validation logic in the ITS processor
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. [w] CPI caller counter PDA (the actual token sender)
    /// 1. [] ITS root PDA
    /// 2. [] Token Manager PDA
    /// 3. [w] Source token account (counter PDA's ATA) - verified against derivation
    /// 4. [w] Token Manager's ATA
    /// 5. [] Gateway root PDA
    /// 6. [] Gateway program ID
    /// 7. [] Gas Service root PDA
    /// 8. [] Gas Service program ID
    /// 9. [] Token mint
    /// 10. [] Token program
    /// 11. [] Call contract signing PDA
    /// 12. [] ITS program ID
    /// 13. [] System program
    SendInterchainTransferWithWrongSeeds {
        /// Token ID for the transfer
        token_id: [u8; 32],
        /// Destination chain
        destination_chain: String,
        /// Destination address
        destination_address: Vec<u8>,
        /// Amount to transfer
        amount: u64,
        /// Gas value for the transfer
        gas_value: u128,
    },

    /// Send an interchain token transfer with additional data to call a contract on the destination
    /// This uses CpiCallContractWithInterchainToken to send tokens along with arbitrary data
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. [w] CPI caller counter PDA (the actual token sender)
    /// 1. [] ITS root PDA
    /// 2. [] Token Manager PDA
    /// 3. [w] Source token account (counter PDA's ATA) - verified against derivation
    /// 4. [w] Token Manager's ATA
    /// 5. [] Gateway root PDA
    /// 6. [] Gateway program ID
    /// 7. [] Gas Service root PDA
    /// 8. [] Gas Service program ID
    /// 9. [] Token mint
    /// 10. [] Token program
    /// 11. [] Call contract signing PDA
    /// 12. [] ITS program ID
    /// 13. [] System program
    CallContractWithInterchainToken {
        /// Token ID for the transfer
        token_id: [u8; 32],
        /// Destination chain
        destination_chain: String,
        /// Destination address
        destination_address: Vec<u8>,
        /// Amount to transfer
        amount: u64,
        /// Additional data to pass to the destination contract
        data: Vec<u8>,
        /// Gas value for the transfer
        gas_value: u128,
    },
}

/// Creates a [`CpiCallerInstruction::Initialize`] instruction.
pub fn initialize(payer: &Pubkey, counter_pda: &(Pubkey, u8)) -> Result<Instruction, ProgramError> {
    let data = to_vec(&CpiCallerInstruction::Initialize {
        counter_pda_bump: counter_pda.1,
    })?;

    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(counter_pda.0, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    Ok(Instruction {
        program_id: crate::ID,
        accounts,
        data,
    })
}

/// Creates a [`CpiCallerInstruction::SendInterchainTransfer`] instruction.
/// The source token account (counter PDA's ATA) is automatically derived and verified inside the processor.
#[allow(clippy::too_many_arguments)]
pub fn send_interchain_transfer(
    payer: &Pubkey,
    cpi_caller_counter_pda: &Pubkey,
    its_root_pda: &Pubkey,
    token_manager_pda: &Pubkey,
    token_manager_ata: &Pubkey,
    gateway_root_pda: &Pubkey,
    gas_service_root_pda: &Pubkey,
    token_mint: &Pubkey,
    token_program: &Pubkey,
    token_id: [u8; 32],
    destination_chain: String,
    destination_address: Vec<u8>,
    amount: u64,
    gas_value: u128,
) -> Result<Instruction, ProgramError> {
    let data = to_vec(&CpiCallerInstruction::SendInterchainTransfer {
        token_id,
        destination_chain,
        destination_address,
        amount,
        gas_value,
    })?;

    // Derive the source ATA (counter PDA's token account)
    let source_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        cpi_caller_counter_pda,
        token_mint,
        token_program,
    );

    // Additional required accounts for proper ITS instruction
    let gateway_program = axelar_solana_gateway::id();
    let gas_service_program = axelar_solana_gas_service::id();
    let (call_contract_signing_pda, _) =
        axelar_solana_gateway::get_call_contract_signing_pda(axelar_solana_its::id());
    let its_program = axelar_solana_its::id();

    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(*cpi_caller_counter_pda, false),
        AccountMeta::new_readonly(*its_root_pda, false),
        AccountMeta::new(*token_manager_pda, false),
        AccountMeta::new(source_ata, false),
        AccountMeta::new(*token_manager_ata, false),
        AccountMeta::new_readonly(*gateway_root_pda, false),
        AccountMeta::new_readonly(gateway_program, false),
        AccountMeta::new(*gas_service_root_pda, false),
        AccountMeta::new_readonly(gas_service_program, false),
        AccountMeta::new(*token_mint, false),
        AccountMeta::new_readonly(*token_program, false),
        AccountMeta::new_readonly(call_contract_signing_pda, false),
        AccountMeta::new_readonly(its_program, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    Ok(Instruction {
        program_id: crate::ID,
        accounts,
        data,
    })
}

/// Creates a [`CpiCallerInstruction::SendInterchainTransferWithWrongSeeds`] instruction.
/// This is used to test validation logic by intentionally providing wrong seeds
#[allow(clippy::too_many_arguments)]
pub fn send_interchain_transfer_with_wrong_seeds(
    payer: &Pubkey,
    cpi_caller_counter_pda: &Pubkey,
    its_root_pda: &Pubkey,
    token_manager_pda: &Pubkey,
    token_manager_ata: &Pubkey,
    gateway_root_pda: &Pubkey,
    gas_service_root_pda: &Pubkey,
    token_mint: &Pubkey,
    token_program: &Pubkey,
    token_id: [u8; 32],
    destination_chain: String,
    destination_address: Vec<u8>,
    amount: u64,
    gas_value: u128,
) -> Result<Instruction, ProgramError> {
    let data = to_vec(
        &CpiCallerInstruction::SendInterchainTransferWithWrongSeeds {
            token_id,
            destination_chain,
            destination_address,
            amount,
            gas_value,
        },
    )?;

    // Derive the source ATA (counter PDA's token account)
    let source_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        cpi_caller_counter_pda,
        token_mint,
        token_program,
    );

    // Additional required accounts for proper ITS instruction
    let gateway_program = axelar_solana_gateway::id();
    let gas_service_program = axelar_solana_gas_service::id();
    let (call_contract_signing_pda, _) =
        axelar_solana_gateway::get_call_contract_signing_pda(axelar_solana_its::id());
    let its_program = axelar_solana_its::id();

    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(*cpi_caller_counter_pda, false),
        AccountMeta::new_readonly(*its_root_pda, false),
        AccountMeta::new(*token_manager_pda, false),
        AccountMeta::new(source_ata, false),
        AccountMeta::new(*token_manager_ata, false),
        AccountMeta::new_readonly(*gateway_root_pda, false),
        AccountMeta::new_readonly(gateway_program, false),
        AccountMeta::new(*gas_service_root_pda, false),
        AccountMeta::new_readonly(gas_service_program, false),
        AccountMeta::new(*token_mint, false),
        AccountMeta::new_readonly(*token_program, false),
        AccountMeta::new_readonly(call_contract_signing_pda, false),
        AccountMeta::new_readonly(its_program, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    Ok(Instruction {
        program_id: crate::ID,
        accounts,
        data,
    })
}

/// Creates a [`CpiCallerInstruction::CallContractWithInterchainToken`] instruction.
/// This sends tokens along with additional data to call a contract on the destination
#[allow(clippy::too_many_arguments)]
pub fn call_contract_with_interchain_token(
    payer: &Pubkey,
    cpi_caller_counter_pda: &Pubkey,
    its_root_pda: &Pubkey,
    token_manager_pda: &Pubkey,
    token_manager_ata: &Pubkey,
    gateway_root_pda: &Pubkey,
    gas_service_root_pda: &Pubkey,
    token_mint: &Pubkey,
    token_program: &Pubkey,
    token_id: [u8; 32],
    destination_chain: String,
    destination_address: Vec<u8>,
    amount: u64,
    data: Vec<u8>,
    gas_value: u128,
) -> Result<Instruction, ProgramError> {
    let instruction_data = to_vec(&CpiCallerInstruction::CallContractWithInterchainToken {
        token_id,
        destination_chain,
        destination_address,
        amount,
        data,
        gas_value,
    })?;

    // Derive the source ATA (counter PDA's token account)
    let source_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        cpi_caller_counter_pda,
        token_mint,
        token_program,
    );

    // Additional required accounts for proper ITS instruction
    let gateway_program = axelar_solana_gateway::id();
    let gas_service_program = axelar_solana_gas_service::id();
    let (call_contract_signing_pda, _) =
        axelar_solana_gateway::get_call_contract_signing_pda(axelar_solana_its::id());
    let its_program = axelar_solana_its::id();

    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(*cpi_caller_counter_pda, false),
        AccountMeta::new_readonly(*its_root_pda, false),
        AccountMeta::new(*token_manager_pda, false),
        AccountMeta::new(source_ata, false),
        AccountMeta::new(*token_manager_ata, false),
        AccountMeta::new_readonly(*gateway_root_pda, false),
        AccountMeta::new_readonly(gateway_program, false),
        AccountMeta::new(*gas_service_root_pda, false),
        AccountMeta::new_readonly(gas_service_program, false),
        AccountMeta::new(*token_mint, false),
        AccountMeta::new_readonly(*token_program, false),
        AccountMeta::new_readonly(call_contract_signing_pda, false),
        AccountMeta::new_readonly(its_program, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    Ok(Instruction {
        program_id: crate::ID,
        accounts,
        data: instruction_data,
    })
}
