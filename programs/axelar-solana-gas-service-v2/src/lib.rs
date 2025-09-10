//! Axelar Gas Service program for the Solana blockchain
#![allow(clippy::little_endian_bytes)]
pub mod instructions;
pub mod state;

use instructions::*;

use anchor_lang::prelude::*;

// Export current sdk types for downstream users building with a different sdk
// version.
use program_utils::ensure_single_feature;

ensure_single_feature!("devnet-amplifier", "stagenet", "testnet", "mainnet");

#[cfg(feature = "devnet-amplifier")]
declare_id!("gasd4em72NAm7faq5dvjN5GkXE59dUkTThWmYDX95bK");

#[cfg(feature = "stagenet")]
declare_id!("gaspfz1SLfPr1zmackMVMgShjkuCGPZ5taN8wAfwreW");

#[cfg(feature = "testnet")]
declare_id!("gaspFGXoWNNMMaYGhJoNRMNAp8R3srFeBmKAoeLgSYy");

#[cfg(feature = "mainnet")]
declare_id!("gas1111111111111111111111111111111111111111");

/// Discriminators for the top-level instructions supported by the Axelar Solana Gas Service program.
/// These discriminators are inherited from the v1 Gas Service program to maintain backwards compatibility.
pub struct GasServiceDiscriminators;

impl GasServiceDiscriminators {
    // Single-byte discriminators
    pub const INITIALIZE: &'static [u8] = &[0];
    pub const TRANSFER_OPERATORSHIP: &'static [u8] = &[1];

    // Two-byte discriminators for SPL Token operations
    pub const SPL_PAY_FOR_CONTRACT_CALL: &'static [u8] = &[2, 0];
    pub const SPL_ADD_GAS: &'static [u8] = &[2, 1];
    pub const SPL_COLLECT_FEES: &'static [u8] = &[2, 2];
    pub const SPL_REFUND: &'static [u8] = &[2, 3];

    // Two-byte discriminators for Native Token operations
    pub const NATIVE_PAY_FOR_CONTRACT_CALL: &'static [u8] = &[3, 0];
    pub const NATIVE_ADD_GAS: &'static [u8] = &[3, 1];
    pub const NATIVE_COLLECT_FEES: &'static [u8] = &[3, 2];
    pub const NATIVE_REFUND: &'static [u8] = &[3, 3];
}

#[program]
pub mod axelar_solana_gas_service_v2 {
    use super::*;

    #[instruction(discriminator = GasServiceDiscriminators::INITIALIZE)]
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::initialize(ctx)
    }

    //
    // Gas-related operations with SPL tokens
    //

    #[instruction(discriminator = GasServiceDiscriminators::SPL_PAY_FOR_CONTRACT_CALL)]
    pub fn pay_spl_for_contract_call<'info>(
        ctx: Context<'_, '_, '_, 'info, PaySplForContractCall<'info>>,
        destination_chain: String,
        destination_address: String,
        payload_hash: [u8; 32],
        gas_fee_amount: u64,
        params: Vec<u8>,
        decimals: u8,
        refund_address: Pubkey,
    ) -> Result<()> {
        instructions::pay_spl_for_contract_call::pay_spl_for_contract_call(
            ctx,
            destination_chain,
            destination_address,
            payload_hash,
            &params,
            gas_fee_amount,
            decimals,
            refund_address,
        )
    }

    #[instruction(discriminator = GasServiceDiscriminators::SPL_ADD_GAS)]
    pub fn add_spl_gas<'info>(
        ctx: Context<'_, '_, '_, 'info, AddSplGas<'info>>,
        tx_hash: [u8; 64],
        log_index: u64,
        gas_fee_amount: u64,
        decimals: u8,
        refund_address: Pubkey,
    ) -> Result<()> {
        instructions::add_spl_gas::add_spl_gas(
            ctx,
            tx_hash,
            log_index,
            gas_fee_amount,
            decimals,
            refund_address,
        )
    }

    #[instruction(discriminator = GasServiceDiscriminators::SPL_COLLECT_FEES)]
    pub fn collect_spl_fees(ctx: Context<CollectSplFees>, amount: u64, decimals: u8) -> Result<()> {
        instructions::collect_spl_fees::collect_spl_fees(ctx, amount, decimals)
    }

    #[instruction(discriminator = GasServiceDiscriminators::SPL_REFUND)]
    pub fn refund_spl_fees(
        ctx: Context<RefundSplFees>,
        tx_hash: [u8; 64],
        log_index: u64,
        fees: u64,
        decimals: u8,
    ) -> Result<()> {
        instructions::refund_spl_fees::refund_spl_fees(ctx, tx_hash, log_index, fees, decimals)
    }

    //
    // Gas-related operations with native token SOL
    //

    #[instruction(discriminator = GasServiceDiscriminators::NATIVE_PAY_FOR_CONTRACT_CALL)]
    pub fn pay_native_for_contract_call(
        ctx: Context<PayNativeForContractCall>,
        destination_chain: String,
        destination_address: String,
        payload_hash: [u8; 32],
        refund_address: Pubkey,
        params: Vec<u8>,
        gas_fee_amount: u64,
    ) -> Result<()> {
        instructions::pay_native_for_contract_call::pay_native_for_contract_call(
            ctx,
            destination_chain,
            destination_address,
            payload_hash,
            refund_address,
            &params,
            gas_fee_amount,
        )
    }

    #[instruction(discriminator = GasServiceDiscriminators::NATIVE_ADD_GAS)]
    pub fn add_native_gas(
        ctx: Context<AddNativeGas>,
        tx_hash: [u8; 64],
        log_index: u64,
        gas_fee_amount: u64,
        refund_address: Pubkey,
    ) -> Result<()> {
        instructions::add_native_gas::add_native_gas(
            ctx,
            tx_hash,
            log_index,
            gas_fee_amount,
            refund_address,
        )
    }

    #[instruction(discriminator = GasServiceDiscriminators::NATIVE_COLLECT_FEES)]
    pub fn collect_native_fees(ctx: Context<CollectNativeFees>, amount: u64) -> Result<()> {
        instructions::collect_native_fees::collect_native_fees(ctx, amount)
    }

    #[instruction(discriminator = GasServiceDiscriminators::NATIVE_REFUND)]
    pub fn refund_native_fees(
        ctx: Context<RefundNativeFees>,
        tx_hash: [u8; 64],
        log_index: u64,
        fees: u64,
    ) -> Result<()> {
        instructions::refund_native_fees::refund_native_fees(ctx, tx_hash, log_index, fees)
    }
}
