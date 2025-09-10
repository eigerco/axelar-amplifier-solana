use crate::state::Treasury;
use anchor_lang::prelude::*;
use axelar_solana_gas_service_events::events::NativeGasRefundedEvent;
use axelar_solana_operators::OperatorAccount;
use program_utils::transfer_lamports_anchor;

/// Refund previously collected native SOL fees (operator only).
#[event_cpi]
#[derive(Accounts)]
pub struct RefundNativeFees<'info> {
    pub operator: Signer<'info>,

    #[account(
        seeds = [
            OperatorAccount::SEED_PREFIX,
            operator.key().as_ref(),
        ],
        bump = operator_pda.bump,
        seeds::program = axelar_solana_operators::ID
    )]
    pub operator_pda: Account<'info, OperatorAccount>,

    /// CHECK: Can be any account to receive funds
    #[account(mut)]
    pub receiver: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [
            Treasury::SEED_PREFIX,
        ],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,
}

pub fn refund_native_fees(
    ctx: Context<RefundNativeFees>,
    tx_hash: [u8; 64],
    log_index: u64,
    fees: u64,
) -> Result<()> {
    if fees == 0 {
        msg!("Gas fee amount cannot be zero");
        return Err(ProgramError::InvalidInstructionData.into());
    }

    transfer_lamports_anchor!(
        ctx.accounts.treasury.to_account_info(),
        ctx.accounts.receiver.to_account_info(),
        fees
    );

    emit_cpi!(NativeGasRefundedEvent {
        tx_hash,
        config_pda: *ctx.accounts.treasury.to_account_info().key,
        log_index,
        receiver: *ctx.accounts.receiver.to_account_info().key,
        fees,
    });

    Ok(())
}
