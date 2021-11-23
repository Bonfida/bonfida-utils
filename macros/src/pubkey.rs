use quote::quote;
use std::str::FromStr;

use proc_macro2::{Span, TokenStream};
use solana_program::pubkey::Pubkey;
use syn::{LitByte, LitStr};

pub fn process(item: TokenStream) -> TokenStream {
    let str: LitStr = syn::parse(item.into()).unwrap();
    let key = str.value();
    let pubkey = Pubkey::from_str(&key).unwrap().to_bytes();
    let bytes = pubkey.iter().map(|b| LitByte::new(*b, Span::call_site()));
    quote!(Pubkey::new_from_array([#(#bytes),*]))
}
