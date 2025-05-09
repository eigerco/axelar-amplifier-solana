//! Instructions for the Interchain Token

use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use super::{minter, InterchainTokenServiceInstruction};

/// Instructions operating on [`TokenManager`] instances.
#[derive(Debug, Eq, PartialEq, Clone, BorshSerialize, BorshDeserialize)]
pub enum Instruction {
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
    Mint {
        /// The amount of tokens to mint.
        amount: u64,
    },

    /// `TokenManager` instructions to manage Operator role.
    ///
    /// 0. [] Interchain Token PDA.
    /// 1..N [`minter::MinterInstruction`] accounts, where the resource is
    /// the Interchain Token PDA.
    MinterInstruction(super::minter::Instruction),
}

/// Creates an [`InterchainTokenServiceInstruction::InterchainTokenInstruction`]
/// instruction with the [`Instruction::Mint`] variant.
///
/// # Errors
/// If serialization fails.
pub fn mint(
    token_id: [u8; 32],
    mint: Pubkey,
    to: Pubkey,
    minter: Pubkey,
    token_program: Pubkey,
    amount: u64,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let (its_root_pda, _) = crate::find_its_root_pda();
    let (token_manager_pda, _) = crate::find_token_manager_pda(&its_root_pda, &token_id);
    let (minter_roles_pda, _) =
        role_management::find_user_roles_pda(&crate::id(), &token_manager_pda, &minter);
    let data = to_vec(&InterchainTokenServiceInstruction::InterchainTokenMint { amount })?;

    Ok(solana_program::instruction::Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(mint, false),
            AccountMeta::new(to, false),
            AccountMeta::new_readonly(its_root_pda, false),
            AccountMeta::new_readonly(token_manager_pda, false),
            AccountMeta::new_readonly(minter, true),
            AccountMeta::new_readonly(minter_roles_pda, false),
            AccountMeta::new_readonly(token_program, false),
        ],
        data,
    })
}

/// Creates an [`Instruction::MinterInstruction`]
/// instruction with the [`minter::Instruction::TransferMintership`]
/// variant.
///
/// # Errors
///
/// If serialization fails.
pub fn transfer_mintership(
    payer: Pubkey,
    token_id: [u8; 32],
    to: Pubkey,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let (its_root_pda, _) = crate::find_its_root_pda();
    let (token_manager_pda, _) = crate::find_token_manager_pda(&its_root_pda, &token_id);
    let accounts = vec![AccountMeta::new_readonly(its_root_pda, false)];
    let (accounts, minter_instruction) =
        minter::transfer_mintership(payer, token_manager_pda, to, Some(accounts))?;

    let inputs = match minter_instruction {
        minter::Instruction::TransferMintership(val) => val,
        minter::Instruction::ProposeMintership(_) | minter::Instruction::AcceptMintership(_) => {
            return Err(ProgramError::InvalidAccountData)
        }
    };
    let data =
        to_vec(&InterchainTokenServiceInstruction::InterchainTokenTransferMintership { inputs })?;

    Ok(solana_program::instruction::Instruction {
        program_id: crate::id(),
        accounts,
        data,
    })
}

/// Creates an [`Instruction::MinterInstruction`]
/// instruction with the [`minter::Instruction::ProposeMintership`] variant.
///
/// # Errors
///
/// If serialization fails.
pub fn propose_mintership(
    payer: Pubkey,
    token_id: [u8; 32],
    to: Pubkey,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let (its_root_pda, _) = crate::find_its_root_pda();
    let (token_manager_pda, _) = crate::find_token_manager_pda(&its_root_pda, &token_id);
    let accounts = vec![AccountMeta::new_readonly(its_root_pda, false)];
    let (accounts, minter_instruction) =
        minter::propose_mintership(payer, token_manager_pda, to, Some(accounts))?;

    let inputs = match minter_instruction {
        minter::Instruction::ProposeMintership(val) => val,
        minter::Instruction::TransferMintership(_) | minter::Instruction::AcceptMintership(_) => {
            return Err(ProgramError::InvalidAccountData)
        }
    };
    let data =
        to_vec(&InterchainTokenServiceInstruction::InterchainTokenProposeMintership { inputs })?;

    Ok(solana_program::instruction::Instruction {
        program_id: crate::id(),
        accounts,
        data,
    })
}

/// Creates an [`Instruction::MinterInstruction`]
/// instruction with the [`minter::Instruction::AcceptMintership`] variant.
///
/// # Errors
///
/// If serialization fails.
pub fn accept_mintership(
    payer: Pubkey,
    token_id: [u8; 32],
    from: Pubkey,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let (its_root_pda, _) = crate::find_its_root_pda();
    let (token_manager_pda, _) = crate::find_token_manager_pda(&its_root_pda, &token_id);
    let accounts = vec![AccountMeta::new_readonly(its_root_pda, false)];
    let (accounts, minter_instruction) =
        minter::accept_mintership(payer, token_manager_pda, from, Some(accounts))?;

    let inputs = match minter_instruction {
        minter::Instruction::AcceptMintership(val) => val,
        minter::Instruction::ProposeMintership(_) | minter::Instruction::TransferMintership(_) => {
            return Err(ProgramError::InvalidAccountData)
        }
    };
    let data =
        to_vec(&InterchainTokenServiceInstruction::InterchainTokenAcceptMintership { inputs })?;

    Ok(solana_program::instruction::Instruction {
        program_id: crate::id(),
        accounts,
        data,
    })
}
