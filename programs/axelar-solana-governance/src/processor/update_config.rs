//! Update Governance Config Account with new Governance Config data.

use borsh::BorshSerialize;
use program_utils::{account_array_structs, pda::ValidPDA};
use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::program_error::ProgramError;

use crate::{
    processor::ensure_valid_governance_root_pda,
    state::{validate_config, GovernanceConfig},
};

account_array_structs! {
    GovernanceConfigUpdateInfo,
    GovernanceConfigUpdateMeta,
    payer,
    root_pda
}

/// Updates the Governance Config Account with the provided Governance Config.
///
/// # Errors
///
/// This function will return a [`ProgramError`] if any of the subcmds fail.
pub(crate) fn process(
    accounts: &[AccountInfo<'_>],
    mut new_governance_config: GovernanceConfig,
) -> Result<(), ProgramError> {
    let GovernanceConfigUpdateInfo { payer, root_pda } =
        GovernanceConfigUpdateInfo::from_account_iter(&mut accounts.iter())?;

    // Check: The operator is the payer and has signed
    let current_config = root_pda.check_initialized_pda::<GovernanceConfig>(&crate::id())?;

    ensure_valid_governance_root_pda(current_config.bump, root_pda.key)?;

    if !payer.is_signer {
        msg!("The operator account must sign the transaction");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if current_config.operator != payer.key.to_bytes() {
        msg!("Only the current operator can update the governance config");
        return Err(ProgramError::InvalidAccountData);
    }

    // Check: Ensure the config data is valid
    validate_config(&new_governance_config)?;

    // We overwrite/preserve from initial config the fields that should not be changed
    new_governance_config.bump = current_config.bump;
    new_governance_config.operator = current_config.operator;

    // Overwrite the config data in the PDA
    let mut data = root_pda.try_borrow_mut_data()?;

    new_governance_config
        .serialize(&mut &mut data[..])
        .map_err(|err| {
            msg!("Failed to serialize new governance config: {}", err);
            ProgramError::InvalidAccountData
        })?;

    Ok(())
}
