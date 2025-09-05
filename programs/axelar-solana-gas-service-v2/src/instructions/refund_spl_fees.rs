use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};
use axelar_solana_gas_service_events::events::SplGasRefundedEvent;

/// Refund previously collected SPL token fees (operator only).
///
/// Accounts expected:
/// 0. `[signer, read-only]` The `operator` account authorized to collect fees.
/// 1. `[writable]` The `receiver` account where the tokens will be sent.
/// 2. `[writable]` The `config_pda` account.
/// 3. `[writable]` The config PDA's associated token account for the mint.
/// 4. `[]` The mint account for the SPL token.
/// 5. `[]` The SPL token program.
#[event_cpi]
#[derive(Accounts)]
pub struct RefundSplFees<'info> {
    #[account(address = config_pda.load()?.operator)]
    pub operator: Signer<'info>,

    #[account(
        mut,
        token::mint = mint,
    )]
    pub receiver_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            Config::SEED_PREFIX,
        ],
        bump = config_pda.load()?.bump,
    )]
    pub config_pda: AccountLoader<'info, Config>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = config_pda,
    )]
    pub config_pda_ata: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn refund_spl_fees(
    ctx: Context<RefundSplFees>,
    tx_hash: [u8; 64],
    log_index: u64,
    fees: u64,
    decimals: u8,
) -> Result<()> {
    if fees == 0 {
        msg!("Gas fee amount cannot be zero");
        return Err(ProgramError::InvalidInstructionData.into());
    }

    let config = ctx.accounts.config_pda.load()?;

    let signer_seeds: &[&[&[u8]]] = &[&[Config::SEED_PREFIX, &[config.bump]]];

    let cpi_accounts = TransferChecked {
        mint: ctx.accounts.mint.to_account_info().clone(),
        from: ctx.accounts.config_pda_ata.to_account_info().clone(),
        to: ctx.accounts.receiver_account.to_account_info().clone(),
        authority: ctx.accounts.config_pda.to_account_info().clone(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    token_interface::transfer_checked(cpi_context, fees, decimals)?;

    emit_cpi!(SplGasRefundedEvent {
        config_pda_ata: *ctx.accounts.config_pda_ata.to_account_info().key,
        mint: *ctx.accounts.mint.to_account_info().key,
        token_program_id: *ctx.accounts.token_program.to_account_info().key,
        tx_hash,
        config_pda: *ctx.accounts.config_pda.to_account_info().key,
        log_index,
        receiver: *ctx.accounts.receiver_account.to_account_info().key,
        fees,
    });

    Ok(())
}
