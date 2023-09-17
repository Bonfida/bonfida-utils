use quote::quote;
use std::str::FromStr;

use proc_macro2::{Span, TokenStream};
use solana_program::pubkey::Pubkey;
use syn::{LitByte, LitStr};

pub fn process(item: TokenStream) -> TokenStream {
    let item_copy = item.clone();
    let str: LitStr = syn::parse(item.into()).unwrap();
    let key = str.value();
    let pubkey = Pubkey::from_str(&key).unwrap();
    let pubkey_bytes = pubkey.to_bytes();
    let (central_state, central_state_nonce) =
        Pubkey::find_program_address(&[&pubkey_bytes], &pubkey);
    let central_state_array = central_state.to_bytes();
    let central_state_bytes = central_state_array
        .iter()
        .map(|b| LitByte::new(*b, Span::call_site()));
    let pubkey_bytes = pubkey_bytes
        .iter()
        .map(|b| LitByte::new(*b, Span::call_site()));
    quote!(
        use solana_program::declare_id;
        pub mod central_state {
            use solana_program::pubkey::Pubkey;
            pub static KEY_BYTES: [u8;32] = [#(#central_state_bytes),*];
            pub static KEY: Pubkey = Pubkey::new_from_array(KEY_BYTES);
            pub static NONCE: u8 = #central_state_nonce;
            pub static SIGNER_SEEDS: [&'static [u8]; 2] = [&super::ID_BYTES, &[NONCE]];
        }
        declare_id!(#item_copy);
        pub static ID_BYTES: [u8;32] = [#(#pubkey_bytes),*];
    )
}
