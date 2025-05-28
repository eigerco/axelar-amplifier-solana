//! Contains

use borsh::to_vec;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{AnyBitPattern, NoUninit};
use core::any::type_name;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{msg, system_instruction, system_program};
use std::borrow::Borrow;
use std::io::Write;

/// Initialize a PDA by writing borsh serialisable data to the buffer
// TODO add constraint that the T: IsInitialized + Pack + BorshSerialize
pub fn init_pda<'a, 'b, T: solana_program::program_pack::Pack>(
    funder_info: &'a AccountInfo<'b>,
    to_create: &'a AccountInfo<'b>,
    program_id: &Pubkey,
    system_program_info: &'a AccountInfo<'b>,
    data: T,
    signer_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let rent = Rent::get()?;
    let ix = &system_instruction::create_account(
        funder_info.key,
        to_create.key,
        rent.minimum_balance(T::LEN).max(1),
        T::get_packed_len() as u64,
        program_id,
    );
    invoke_signed(
        ix,
        &[
            funder_info.clone(),
            to_create.clone(),
            system_program_info.clone(),
        ],
        &[signer_seeds],
    )?;
    let mut account_data = to_create.try_borrow_mut_data()?;
    T::pack(data, &mut account_data)?;
    Ok(())
}

/// Initialize an associated account, with raw bytes.
pub fn init_pda_raw_bytes<'a, 'b>(
    funder_info: &'a AccountInfo<'b>,
    to_create: &'a AccountInfo<'b>,
    program_id: &Pubkey,
    system_program_info: &'a AccountInfo<'b>,
    data: &[u8],
    signer_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let rent = Rent::get()?;
    let ix = &system_instruction::create_account(
        funder_info.key,
        to_create.key,
        rent.minimum_balance(data.len()).max(1),
        data.len() as u64,
        program_id,
    );
    invoke_signed(
        ix,
        &[
            funder_info.clone(),
            to_create.clone(),
            system_program_info.clone(),
        ],
        &[signer_seeds],
    )?;
    let mut account_data = to_create.try_borrow_mut_data()?;
    account_data.write_all(data).map_err(|err| {
        msg!("Cannot write data to account: {}", err);
        ProgramError::InvalidArgument
    })
}

/// Initializes a PDA without writing anything to the data storage
pub fn init_pda_raw<'a, 'b>(
    funder_info: &'a AccountInfo<'b>,
    to_create: &'a AccountInfo<'b>,
    program_id: &Pubkey,
    system_program_info: &'a AccountInfo<'b>,
    data_len: u64,
    signer_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let rent = Rent::get()?;
    let ix = &system_instruction::create_account(
        funder_info.key,
        to_create.key,
        rent.minimum_balance(data_len.try_into().expect("u64 fits into sbf word size"))
            .max(1),
        data_len,
        program_id,
    );
    invoke_signed(
        ix,
        &[
            funder_info.clone(),
            to_create.clone(),
            system_program_info.clone(),
        ],
        &[signer_seeds],
    )?;
    Ok(())
}

/// Close an associated account
pub fn close_pda(
    lamport_destination: &AccountInfo<'_>,
    pda_to_close: &AccountInfo<'_>,
) -> Result<(), solana_program::program_error::ProgramError> {
    // Transfer the lamports to the destination account
    let dest_starting_lamports = lamport_destination.lamports();
    **lamport_destination.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(pda_to_close.lamports())
        .unwrap();
    **pda_to_close.lamports.borrow_mut() = 0;

    // Downgrade the PDA's account to the system program
    pda_to_close.assign(&system_program::ID);

    // Downsize the PDA's account to 0
    pda_to_close.realloc(0, false)?;

    Ok(())
}

/// Extension trait for AccountInfo to check if the account is an initialized
/// PDA
pub trait ValidPDA {
    /// Check if the account is an initialized PDA
    // TODO add constraint that the T: IsInitialized + Pack + BorshSerialize
    fn check_initialized_pda<T: solana_program::program_pack::Pack>(
        &self,
        expected_owner_program_id: &Pubkey,
    ) -> Result<T, ProgramError>;

    /// Check if the account is an initialized PDA without deserializing the
    /// data
    fn check_initialized_pda_without_deserialization(
        &self,
        expected_owner_program_id: &Pubkey,
    ) -> Result<(), ProgramError>;

    /// Check if the account is an initialized PDA
    fn check_uninitialized_pda(&self) -> Result<(), ProgramError>;

    /// Check if the account is an initialized PDA with a data check
    fn is_initialized_pda(&self, expected_owner_program_id: &Pubkey) -> bool;
}

impl<'a> ValidPDA for &AccountInfo<'a> {
    fn check_initialized_pda<T: solana_program::program_pack::Pack>(
        &self,
        expected_owner_program_id: &Pubkey,
    ) -> Result<T, ProgramError> {
        self.check_initialized_pda_without_deserialization(expected_owner_program_id)?;

        let data = self.try_borrow_data()?;
        T::unpack_from_slice(data.borrow()).map_err(|_| ProgramError::InvalidAccountData)
    }

    fn check_initialized_pda_without_deserialization(
        &self,
        expected_owner_program_id: &Pubkey,
    ) -> Result<(), ProgramError> {
        let has_lamports = **self.try_borrow_lamports()? > 0;
        if !has_lamports {
            msg!("account does not have enough lamports");
            return Err(ProgramError::InsufficientFunds);
        }
        let has_correct_owner = self.owner == expected_owner_program_id;
        if !has_correct_owner {
            msg!("account does not have the expected owner");
            return Err(ProgramError::IllegalOwner);
        }

        Ok(())
    }

    fn check_uninitialized_pda(&self) -> Result<(), ProgramError> {
        let data_is_empty = self.try_borrow_data()?.is_empty();
        if !data_is_empty {
            return Err(ProgramError::InvalidAccountData);
        }
        let owner_is_system = self.owner == &solana_program::system_program::id();
        if !owner_is_system {
            return Err(ProgramError::IllegalOwner);
        }
        Ok(())
    }

    fn is_initialized_pda(&self, expected_owner_program_id: &Pubkey) -> bool {
        let data_is_empty = self
            .try_borrow_data()
            .expect("to borrow the data")
            .is_empty();
        let has_correct_owner = self.owner == expected_owner_program_id;
        !data_is_empty && has_correct_owner
    }
}

/// Convenience trait to store and load rkyv serialized data to/from an account.
pub trait BorshPda
where
    Self: Sized + Clone + BorshSerialize + BorshDeserialize,
{
    /// Initializes an account with the current object serialized data.
    fn init<'a>(
        &self,
        program_id: &Pubkey,
        system_account: &AccountInfo<'a>,
        payer: &AccountInfo<'a>,
        into: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
    ) -> ProgramResult {
        let serialized_data = to_vec(self)?;

        init_pda_raw_bytes(
            payer,
            into,
            program_id,
            system_account,
            &serialized_data,
            signer_seeds,
        )?;

        Ok(())
    }

    /// Stores the current object serialized data into the destination account.
    /// The account must have been initialized beforehand.
    fn store<'a>(
        &self,
        payer: &AccountInfo<'a>,
        destination: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
    ) -> ProgramResult {
        let serialized_data = to_vec(self)?;

        if serialized_data.len() > destination.data_len() {
            let lamports_needed = Rent::get()?.minimum_balance(serialized_data.len());
            let lamports_diff = lamports_needed.saturating_sub(destination.lamports());

            invoke(
                &system_instruction::transfer(payer.key, destination.key, lamports_diff),
                &[payer.clone(), destination.clone(), system_program.clone()],
            )?;
        }

        destination.realloc(serialized_data.len(), false)?;
        let mut account_data = destination.try_borrow_mut_data()?;
        account_data.copy_from_slice(serialized_data.as_slice());

        Ok(())
    }

    /// Loads the account data and deserializes it.
    fn load(source_account: &AccountInfo<'_>) -> Result<Self, ProgramError> {
        let account_data = source_account.try_borrow_data()?;
        let deserialized = match Self::try_from_slice(&account_data[..]) {
            Ok(value) => value,
            Err(err) => {
                msg!(
                    "Warning: failed to deserialize account as {}: {}. The account might not have been initialized.",
                    type_name::<Self>(),
                    err,
                );

                return Err(ProgramError::from(err));
            }
        };

        Ok(deserialized)
    }
}

/// A trait for types that can be safely converted to and from byte slices using `bytemuck`.
pub trait BytemuckedPda: Sized + NoUninit + AnyBitPattern {
    /// Reads an immutable reference to `Self` from a byte slice.
    ///
    /// This method attempts to interpret the provided byte slice as an instance of `Self`.
    /// It checks that the length of the slice matches the size of `Self` to ensure safety.
    fn read(data: &[u8]) -> Option<&Self> {
        let result: &Self = bytemuck::try_from_bytes(data)
            .map_err(|err| {
                msg!("bytemuck error {:?}", err);
                err
            })
            .ok()?;
        Some(result)
    }

    /// Reads a mutable reference to `Self` from a mutable byte slice.
    ///
    /// Similar to [`read`], but allows for mutation of the underlying data.
    /// This is useful when you need to modify the data in place.
    fn read_mut(data: &mut [u8]) -> Option<&mut Self> {
        let result: &mut Self = bytemuck::try_from_bytes_mut(data)
            .map_err(|err| {
                msg!("bytemuck error {:?}", err);
                err
            })
            .ok()?;
        Some(result)
    }

    /// Writes the instance of `Self` into a mutable byte slice.
    ///
    /// This method serializes `self` into its byte representation and copies it into the
    /// provided mutable byte slice. It ensures that the destination slice is of the correct
    /// length to hold the data.
    fn write(&self, data: &mut [u8]) -> Option<()> {
        let self_bytes = bytemuck::bytes_of(self);
        if data.len() != self_bytes.len() {
            return None;
        }
        data.copy_from_slice(self_bytes);
        Some(())
    }
}
