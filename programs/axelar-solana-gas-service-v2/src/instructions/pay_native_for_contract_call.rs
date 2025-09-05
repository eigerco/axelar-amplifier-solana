use crate::state::Config;
use anchor_lang::{prelude::*, system_program};
use axelar_solana_gas_service_events::events::NativeGasPaidForContractCallEvent;

/// Pay gas fees for a contract call using native SOL.
///
/// Accounts expected:
/// 0. `[signer, writable]` The account (`payer`) paying the gas fee in lamports.
/// 1. `[writable]` The `config_pda` account that receives the lamports.
/// 2. `[]` The `system_program` account.
#[event_cpi]
#[derive(Accounts)]
pub struct PayNativeForContractCall<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
    	mut,
        seeds = [
            Config::SEED_PREFIX,
        ],
        bump = config_pda.load()?.bump,
    )]
    pub config_pda: AccountLoader<'info, Config>,

    pub system_program: Program<'info, System>,
}

pub fn pay_native_for_contract_call(
    ctx: Context<PayNativeForContractCall>,
    destination_chain: String,
    destination_address: String,
    payload_hash: [u8; 32],
    refund_address: Pubkey,
    params: &[u8],
    gas_fee_amount: u64,
) -> Result<()> {
    if gas_fee_amount == 0 {
        msg!("Gas fee amount cannot be zero");
        return Err(ProgramError::InvalidInstructionData.into());
    }

    let config_pda_account_info = &ctx.accounts.config_pda.to_account_info();

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: config_pda_account_info.clone(),
            },
        ),
        gas_fee_amount,
    )?;

    emit_cpi!(NativeGasPaidForContractCallEvent {
        config_pda: *config_pda_account_info.key,
        destination_chain: destination_chain.clone(),
        destination_address: destination_address.clone(),
        payload_hash,
        refund_address,
        params: params.to_vec(),
        gas_fee_amount,
    });

    Ok(())
}
