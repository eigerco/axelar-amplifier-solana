//! Program state processor

use borsh::BorshDeserialize;
use program_utils::check_program_account;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;

use solana_program::msg;
use solana_program::pubkey::Pubkey;

use crate::instruction::AxelarEventCpiInstruction;

/// Instruction processor
pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    input: &[u8],
) -> ProgramResult {
    check_program_account(program_id, crate::check_id)?;

    let instruction = AxelarEventCpiInstruction::try_from_slice(input)?;

    match instruction {
        AxelarEventCpiInstruction::EmitEvent { memo } => {
            process_memo(program_id, accounts, memo)?;
        }
    }

    Ok(())
}

fn process_memo(_program_id: &Pubkey, accounts: &[AccountInfo<'_>], memo: String) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

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

    msg!("Memo: {}", memo);

    Ok(())
}
