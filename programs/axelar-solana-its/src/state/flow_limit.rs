//! Module with data structure definition for handling flow limits on interchain
//! tokens.

use core::any::type_name;
use core::mem::size_of;
use core::time::Duration;

use borsh::{BorshDeserialize, BorshSerialize};
use program_utils::pda::BorshPda;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{Pack, Sealed};
use solana_program::sysvar::Sysvar;

const EPOCH_TIME: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Debug, Eq, PartialEq, Clone, BorshSerialize, BorshDeserialize)]
/// Struct containing flow information for a specific epoch.
pub struct FlowSlot {
    pub flow_in: u64,
    pub flow_out: u64,
    pub bump: u8,
}

/// Module for handling flow limits on interchain tokens.
impl FlowSlot {
    pub(crate) fn new(bump: u8) -> Self {
        Self {
            flow_in: 0,
            flow_out: 0,
            bump,
        }
    }

    pub(crate) fn add_flow(
        &mut self,
        flow_limit: u64,
        amount: u64,
        direction: FlowDirection,
    ) -> ProgramResult {
        let (to_add, to_compare) = match direction {
            FlowDirection::In => (&mut self.flow_in, self.flow_out),
            FlowDirection::Out => (&mut self.flow_out, self.flow_in),
        };

        Self::update_flow(flow_limit, to_add, to_compare, amount)
    }

    fn update_flow(
        flow_limit: u64,
        to_add: &mut u64,
        to_compare: u64,
        amount: u64,
    ) -> ProgramResult {
        // As the flow limit can be updated and set to 0, we need to handle the
        // case.
        if flow_limit == 0 {
            return Ok(());
        }

        // The flow limit can be interpreted as a limit over the net amount of tokens
        // transferred from one chain to another within a six hours time window. Thus,
        // if the limit is 100 and 30 tokens have been transferred in one
        // direction, one could still transfer 130 tokens in the other direction
        // within the same epoch, for instance.
        let new_total = to_add
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        let max_allowed_flow = to_compare
            .checked_add(flow_limit)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        if new_total > max_allowed_flow || amount > flow_limit {
            msg!("Flow limit exceeded");
            return Err(ProgramError::InvalidArgument);
        }

        *to_add = new_total;

        Ok(())
    }
}

impl Pack for FlowSlot {
    const LEN: usize = 2 * size_of::<u64>() + size_of::<u8>();

    #[allow(clippy::unwrap_used)]
    fn pack_into_slice(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap();
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, solana_program::program_error::ProgramError> {
        let mut mut_src: &[u8] = src;
        Self::deserialize(&mut mut_src).map_err(|err| {
            msg!(
                "Error: failed to deserialize account as {}: {}",
                type_name::<Self>(),
                err
            );
            ProgramError::InvalidAccountData
        })
    }
}

impl Sealed for FlowSlot {}
impl BorshPda for FlowSlot {}

#[derive(Debug, Clone, Copy)]
pub(crate) enum FlowDirection {
    In,
    Out,
}

pub(crate) fn current_flow_epoch() -> Result<u64, ProgramError> {
    flow_epoch_with_timestamp(Clock::get()?.unix_timestamp)
}

/// Returns the current flow epoch based on the provided clock.
///
/// # Errors
///
/// Returns an error if conversion from clock to internal flow epoch fails.
pub fn flow_epoch_with_timestamp(timestamp: i64) -> Result<u64, ProgramError> {
    let unix_timestamp: u64 = timestamp
        .try_into()
        .map_err(|_err| ProgramError::ArithmeticOverflow)?;

    unix_timestamp
        .checked_div(EPOCH_TIME.as_secs())
        .ok_or(ProgramError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_slot_new_valid() {
        // Test valid creation of FlowSlot
        let expected_flow_in = 0;
        let expected_flow_out = 0;

        let slot = FlowSlot::new(0);
        assert_eq!(slot.flow_in, expected_flow_in);
        assert_eq!(slot.flow_out, expected_flow_out);
    }

    #[test]
    fn test_add_flow_in_valid() {
        // Test adding flow_in within limits
        let flow_limit = 100;
        let mut slot = FlowSlot::new(0);
        slot.add_flow(flow_limit, 20, FlowDirection::In).unwrap();
        slot.add_flow(flow_limit, 30, FlowDirection::Out).unwrap();
        let amount = 40;

        let result = slot.add_flow(flow_limit, amount, FlowDirection::In);
        assert!(result.is_ok());
        assert_eq!(slot.flow_in, 60);
    }

    #[test]
    fn test_add_flow_in_exceeds_limit() {
        // Test adding flow_in that exceeds limit should fail
        let flow_limit = 100;
        let mut slot = FlowSlot::new(0);
        slot.add_flow(flow_limit, 80, FlowDirection::In).unwrap();
        let amount = 30; // This would make flow_in 110, exceeding the limit

        let result = slot.add_flow(flow_limit, amount, FlowDirection::In);
        assert_eq!(result, Err(ProgramError::InvalidArgument));
        assert_eq!(slot.flow_in, 80); // Ensure flow_in hasn't changed
    }

    #[test]
    fn test_add_flow_out_valid() {
        // Test adding flow_out within limits
        let flow_limit = 100;
        let mut slot = FlowSlot::new(0);
        slot.add_flow(flow_limit, 30, FlowDirection::In).unwrap();
        slot.add_flow(flow_limit, 20, FlowDirection::Out).unwrap();
        let amount = 50;

        let result = slot.add_flow(flow_limit, amount, FlowDirection::Out);
        assert!(result.is_ok());
        assert_eq!(slot.flow_out, 70);
    }

    #[test]
    fn test_add_flow_out_exceeds_limit() {
        // Test adding flow_out that exceeds limit should fail
        let flow_limit = 100;
        let mut slot = FlowSlot::new(0);
        slot.add_flow(flow_limit, 90, FlowDirection::Out).unwrap();
        let amount = 20; // This would make flow_out 110, exceeding the limit

        let result = slot.add_flow(flow_limit, amount, FlowDirection::Out);
        assert_eq!(result, Err(ProgramError::InvalidArgument));
        assert_eq!(slot.flow_out, 90); // Ensure flow_out hasn't changed
    }

    #[test]
    fn test_add_flow_in_overflow() {
        // Test arithmetic overflow in add_flow_in
        let flow_limit = u64::MAX;
        let mut slot = FlowSlot::new(0);
        slot.add_flow(flow_limit, u64::MAX - 10, FlowDirection::In)
            .unwrap();
        let amount = 20;

        let result = slot.add_flow(flow_limit, amount, FlowDirection::In);
        assert_eq!(result, Err(ProgramError::ArithmeticOverflow));
        assert_eq!(slot.flow_in, u64::MAX - 10); // Ensure flow_in hasn't
                                                 // changed
    }

    #[test]
    fn test_add_flow_out_overflow() {
        // Test arithmetic overflow in add_flow_out
        let flow_limit = u64::MAX;
        let mut slot = FlowSlot::new(0);
        slot.add_flow(flow_limit, u64::MAX - 10, FlowDirection::Out)
            .unwrap();
        let amount = 20;

        let result = slot.add_flow(flow_limit, amount, FlowDirection::Out);
        assert_eq!(result, Err(ProgramError::ArithmeticOverflow));
        assert_eq!(slot.flow_out, u64::MAX - 10); // Ensure flow_out hasn't
                                                  // changed
    }

    #[test]
    fn test_add_flow_zero_flow_limit() {
        // Test behavior when flow_limit is zero in add_flow methods
        let flow_limit = 0;
        let mut slot = FlowSlot::new(0);

        let result_in = slot.add_flow(flow_limit, 10, FlowDirection::In);
        let result_out = slot.add_flow(flow_limit, 10, FlowDirection::Out);

        // Since flow_limit is zero, the methods should return Ok without modifying
        // flow_in or flow_out
        assert!(result_in.is_ok());
        assert!(result_out.is_ok());
        assert_eq!(slot.flow_in, 0);
        assert_eq!(slot.flow_out, 0);
    }

    #[test]
    fn test_add_flow_amount_exceeds_flow_limit() {
        // Test when amount exceeds flow_limit in add_flow methods
        let flow_limit = 50;
        let mut slot = FlowSlot::new(0);
        let amount = 60; // Exceeds flow_limit

        let result_in = slot.add_flow(flow_limit, amount, FlowDirection::In);
        let result_out = slot.add_flow(flow_limit, amount, FlowDirection::Out);

        assert_eq!(result_in, Err(ProgramError::InvalidArgument));
        assert_eq!(result_out, Err(ProgramError::InvalidArgument));
        assert_eq!(slot.flow_in, 0);
        assert_eq!(slot.flow_out, 0);
    }

    #[test]
    fn test_add_flow_new_total_exceeds_max_allowed_flow() {
        // Test when new_total exceeds max_allowed_flow in add_flow methods
        let flow_limit = 100;
        let mut slot = FlowSlot::new(0);
        slot.add_flow(flow_limit, 80, FlowDirection::In).unwrap();
        slot.add_flow(flow_limit, 50, FlowDirection::Out).unwrap();
        let amount = 80; // This would make flow_in 160, which exceeds max_allowed_flow (flow_out +
                         // flow_limit = 150)

        let result = slot.add_flow(flow_limit, amount, FlowDirection::In);

        assert_eq!(result, Err(ProgramError::InvalidArgument));
        assert_eq!(slot.flow_in, 80); // Ensure flow_in hasn't changed
    }

    #[test]
    fn test_add_flow_new_total_exceeds_max_allowed_flow_over_multiple_updates() {
        // Test when new_total exceeds max_allowed_flow in add_flow methods
        let flow_limit = 100;
        let mut slot = FlowSlot::new(0);
        slot.add_flow(flow_limit, 80, FlowDirection::In).unwrap();
        slot.add_flow(flow_limit, 50, FlowDirection::Out).unwrap();
        let amount = 20; // This would make flow_in 100, which does not exceed max_allowed_flow (flow_out
                         // + flow_limit = 150)

        let result = slot.add_flow(flow_limit, amount, FlowDirection::In);

        assert!(result.is_ok());

        let amount = 60; // This would make flow_in 160, which exceeds max_allowed_flow (flow_out +
                         // flow_limit = 150)

        let result = slot.add_flow(flow_limit, amount, FlowDirection::In);

        assert_eq!(result, Err(ProgramError::InvalidArgument));
        assert_eq!(slot.flow_in, 100); // Ensure flow_in hasn't changed
    }

    #[test]
    fn test_flow_slot_initialization_with_direction() {
        // Test that FlowSlot initializes correctly based on transfer direction
        let flow_limit = 100;
        let amount = 50;

        // Test incoming transfer initialization
        let mut slot_in = FlowSlot::new(0);
        slot_in
            .add_flow(flow_limit, amount, FlowDirection::In)
            .unwrap();
        assert_eq!(slot_in.flow_in, amount);
        assert_eq!(slot_in.flow_out, 0);

        // Test outgoing transfer initialization
        let mut slot_out = FlowSlot::new(0);
        slot_out
            .add_flow(flow_limit, amount, FlowDirection::Out)
            .unwrap();
        assert_eq!(slot_out.flow_in, 0);
        assert_eq!(slot_out.flow_out, amount);
    }
}
