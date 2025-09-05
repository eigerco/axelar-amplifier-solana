use crate::state::Config;
use anchor_lang::prelude::*;

/// Collect accrued native SOL fees (operator only).
///
/// Accounts expected:
/// 1. `[signer, read-only]` The `operator` account authorized to collect fees.
/// 2. `[writable]` The `config_pda` account holding the accrued lamports to collect.
/// 3. `[writable]` The `receiver` account where the collected lamports will be sent.
#[derive(Accounts)]
pub struct CollectNativeFees<'info> {
    #[account(address = config_pda.load()?.operator)]
    pub operator: Signer<'info>,

    #[account(
    	mut,
        seeds = [
            Config::SEED_PREFIX,
        ],
        bump = config_pda.load()?.bump,
    )]
    pub config_pda: AccountLoader<'info, Config>,

    /// CHECK: Can be any account to receive funds
    #[account(mut)]
    pub receiver: UncheckedAccount<'info>,
}

pub fn collect_native_fees(ctx: Context<CollectNativeFees>, amount: u64) -> Result<()> {
    if amount == 0 {
        msg!("Gas fee amount cannot be zero");
        return Err(ProgramError::InvalidInstructionData.into());
    }

    // TODO(v2) consider making this a utility function in program-utils
    // similar to transfer_lamports
    if ctx.accounts.config_pda.get_lamports() < amount {
        return Err(ProgramError::InsufficientFunds.into());
    }
    ctx.accounts.config_pda.sub_lamports(amount)?;
    ctx.accounts.receiver.add_lamports(amount)?;

    Ok(())
}
