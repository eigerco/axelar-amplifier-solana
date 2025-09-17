extern crate proc_macro;

use event_cpi::gen_discriminator;
use quote::quote;
use syn::parse_macro_input;

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
