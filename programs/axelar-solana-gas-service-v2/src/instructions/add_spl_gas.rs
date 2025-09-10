use crate::state::Treasury;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};
use axelar_solana_gas_service_events::events::SplGasAddedEvent;

/// Add more gas (SPL tokens) to an existing contract call.
///
/// Accounts expected:
/// 0. `[signer, writable]` The account (`sender`) paying the gas fee in SPL tokens.
/// 1. `[writable]` The sender's associated token account for the mint.
/// 2. `[writable]` The `config_pda` account.
/// 3. `[writable]` The config PDA's associated token account for the mint.
/// 4. `[]` The mint account for the SPL token.
/// 5. `[]` The SPL token program.
/// 6+. `[signer, writable]` Optional additional accounts required by the SPL token program for the transfer.
#[event_cpi]
#[derive(Accounts)]
pub struct AddSplGas<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = sender,
    )]
    pub sender_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            Treasury::SEED_PREFIX,
        ],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = treasury,
    )]
    pub treasury_ata: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn add_spl_gas<'info>(
    // explicitly specify all lifetimes to fix the `remaining_accounts` issue
    // see more: https://solana.stackexchange.com/questions/13275/cpicontext-with-remaining-accounts-is-not-working-because-of-lifetimes
    ctx: Context<'_, '_, '_, 'info, AddSplGas<'info>>,
    tx_hash: [u8; 64],
    log_index: u64,
    gas_fee_amount: u64,
    decimals: u8,
    refund_address: Pubkey,
) -> Result<()> {
    if gas_fee_amount == 0 {
        msg!("Gas fee amount cannot be zero");
        return Err(ProgramError::InvalidInstructionData.into());
    }

    let cpi_accounts = TransferChecked {
        mint: ctx.accounts.mint.to_account_info().clone(),
        from: ctx.accounts.sender_ata.to_account_info().clone(),
        to: ctx.accounts.treasury_ata.to_account_info().clone(),
        authority: ctx.accounts.sender.to_account_info().clone(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts)
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    token_interface::transfer_checked(cpi_context, gas_fee_amount, decimals)?;

    emit_cpi!(SplGasAddedEvent {
        config_pda: *ctx.accounts.treasury.to_account_info().key,
        config_pda_ata: *ctx.accounts.treasury_ata.to_account_info().key,
        mint: *ctx.accounts.mint.to_account_info().key,
        token_program_id: *ctx.accounts.token_program.to_account_info().key,
        tx_hash,
        log_index,
        refund_address,
        gas_fee_amount,
    });

    Ok(())
}
