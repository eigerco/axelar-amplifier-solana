use crate::state::Treasury;
use anchor_lang::{prelude::*, system_program};
use axelar_solana_gas_service_events::events::NativeGasAddedEvent;

/// Add more native SOL gas to an existing transaction.
#[event_cpi]
#[derive(Accounts)]
pub struct AddNativeGas<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(
        mut,
        seeds = [
            Treasury::SEED_PREFIX,
        ],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    pub system_program: Program<'info, System>,
}

pub fn add_native_gas(
    ctx: Context<AddNativeGas>,
    tx_hash: [u8; 64],
    log_index: u64,
    gas_fee_amount: u64,
    refund_address: Pubkey,
) -> Result<()> {
    if gas_fee_amount == 0 {
        msg!("Gas fee amount cannot be zero");
        return Err(ProgramError::InvalidInstructionData.into());
    }

    let treasury_account_info = &ctx.accounts.treasury.to_account_info();

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.sender.to_account_info(),
                to: treasury_account_info.clone(),
            },
        ),
        gas_fee_amount,
    )?;

    emit_cpi!(NativeGasAddedEvent {
        config_pda: *treasury_account_info.key,
        tx_hash,
        log_index,
        refund_address,
        gas_fee_amount,
    });

    Ok(())
}
