use crate::state::Config;
use anchor_lang::prelude::*;

/// Transfer operatorship of the gas service to a new operator.
///
/// Accounts expected:
/// 0. `[signer, writable]` The current `operator` account
/// 1. `[]` The new `operator` account to transfer operatorship to
/// 2. `[writable]` The `config_pda` account
#[derive(Accounts)]
pub struct TransferOperatorship<'info> {
    #[account(mut, address = config_pda.load()?.operator)]
    pub current_operator: Signer<'info>,

    /// CHECK: The new operator can be any valid Solana account.
    /// No additional validation is required as we're simply storing the public key.
    pub new_operator: AccountInfo<'info>,

    #[account(
    	mut,
        seeds = [
            Config::SEED_PREFIX,
        ],
        bump = config_pda.load()?.bump,
    )]
    pub config_pda: AccountLoader<'info, Config>,
}

pub fn transfer_operatorship(ctx: Context<TransferOperatorship>) -> Result<()> {
    let config_pda = &mut ctx.accounts.config_pda.load_mut()?;

    config_pda.operator = ctx.accounts.new_operator.key();

    Ok(())
}
