extern crate proc_macro;

use event_cpi::sighash;

use quote::quote;
use syn::parse_macro_input;

fn gen_discriminator(namespace: &str, name: impl ToString) -> proc_macro2::TokenStream {
    let discriminator = sighash(namespace, name.to_string().as_str());
    format!("&{discriminator:?}").parse().unwrap()
}

#[proc_macro_attribute]
pub fn event(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let event_strct = parse_macro_input!(input as syn::ItemStruct);
    let event_name = &event_strct.ident;

    let discriminator = gen_discriminator("event", event_name);

    let ret = quote! {
        #[derive(BorshSerialize, BorshDeserialize)]
        #event_strct

        impl event_cpi::CpiEvent for #event_name {
            fn data(&self) -> Vec<u8> {
                let mut data = Vec::with_capacity(256);
                data.extend_from_slice(#event_name::DISCRIMINATOR);
                self.serialize(&mut data).unwrap();
                data
            }
        }

        impl event_cpi::Discriminator for #event_name {
            const DISCRIMINATOR: &'static [u8] = #discriminator;
        }
    };

    #[allow(unreachable_code)]
    proc_macro::TokenStream::from(ret)
}

#[proc_macro]
pub fn event_cpi_accounts(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input to get the accounts list name (optional)
    let accounts_list_name = if input.is_empty() {
        // Default to "accounts" if no argument provided
        quote::format_ident!("accounts")
    } else {
        // Parse the provided identifier
        let accounts_ident = parse_macro_input!(input as syn::Ident);
        accounts_ident
    };

    proc_macro::TokenStream::from(quote! {
        let __event_cpi_authority_info = solana_program::account_info::next_account_info(#accounts_list_name)?;
        let __event_cpi_program_account = solana_program::account_info::next_account_info(#accounts_list_name)?;

        let (__event_cpi_derived_authority_info, __event_cpi_authority_bump) =
            solana_program::pubkey::Pubkey::find_program_address(&[event_cpi::EVENT_AUTHORITY_SEED], &crate::ID);

        // Check that the event authority public key matches
        if *__event_cpi_authority_info.key != __event_cpi_derived_authority_info {
            return Err(solana_program::program_error::ProgramError::InvalidAccountData);
        }

        if *__event_cpi_program_account.key != crate::ID {
            return Err(solana_program::program_error::ProgramError::IncorrectProgramId);
        }
    })
}

#[proc_macro]
pub fn emit_cpi(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let event_struct = parse_macro_input!(input as syn::Expr);

    proc_macro::TokenStream::from(quote! {
        // 1. Assumes these two values are in scope from event_cpi_accounts! macro
        // __event_cpi_authority_info
        // __event_cpi_authority_bump

        let disc = event_cpi::EVENT_IX_TAG_LE;
        let inner_data = event_cpi::CpiEvent::data(&#event_struct);
        let ix_data: Vec<u8> = event_cpi::EVENT_IX_TAG_LE
            .into_iter()
            .map(|b| *b)
            .chain(inner_data.into_iter())
            .collect();

        // 2. construct the instruction (non-anchor style)
        let ix = solana_program::instruction::Instruction::new_with_bytes(
            crate::ID,
            &ix_data,
            vec![
                solana_program::instruction::AccountMeta::new_readonly(
                    *__event_cpi_authority_info.key,
                    true,
                ),
            ],
        );
        // 3. invoke_signed the instruction
        solana_program::program::invoke_signed(
            &ix,
            // TODO check if this needs to be cloned
            &[__event_cpi_authority_info.clone()],
            &[&[event_cpi::EVENT_AUTHORITY_SEED, &[__event_cpi_authority_bump]]],
        )?;
    })
}

#[proc_macro]
pub fn event_cpi_handler(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input to get the accounts list name (optional)
    let accounts_list_name = if input.is_empty() {
        // Default to "accounts" if no argument provided
        quote::format_ident!("accounts")
    } else {
        // Parse the provided identifier
        let accounts_ident = parse_macro_input!(input as syn::Ident);
        accounts_ident
    };

    proc_macro::TokenStream::from(quote! {
        {
            let __event_cpi_authority_info = next_account_info(#accounts_list_name)?;
            let __event_cpi_program_account = next_account_info(#accounts_list_name)?;

            let (__event_cpi_derived_authority_info, __event_cpi_authority_bump) =
                solana_program::pubkey::Pubkey::find_program_address(&[event_cpi::EVENT_AUTHORITY_SEED], &crate::ID);

            // Check that the event authority public key matches
            if *__event_cpi_authority_info.key != __event_cpi_derived_authority_info {
                return Err(solana_program::program_error::ProgramError::InvalidAccountData);
            }

            if *__event_cpi_program_account.key != crate::ID {
                return Err(solana_program::program_error::ProgramError::InvalidAccountData);
            }
        }
    })
}
