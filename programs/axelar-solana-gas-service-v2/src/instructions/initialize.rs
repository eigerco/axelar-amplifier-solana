use crate::state::Config;
use anchor_lang::prelude::*;

/// Initialize the configuration PDA.
///
/// Accounts expected:
/// 0. `[signer, writable]` The account (`payer`) paying for PDA creation
/// 1. `[]` The `operator` account of this PDA.
/// 2. `[writable]` The `config_pda` account to be created.
/// 3. `[]` The `system_program` account.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub operator: Signer<'info>,

    #[account(
        init,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
        payer = payer,
        seeds = [
            Config::SEED_PREFIX,
        ],
        bump,
    )]
    pub config_pda: AccountLoader<'info, Config>,

    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let config_pda = &mut ctx.accounts.config_pda.load_init()?;

    config_pda.operator = ctx.accounts.operator.key();
    config_pda.bump = ctx.bumps.config_pda;

    Ok(())
}
