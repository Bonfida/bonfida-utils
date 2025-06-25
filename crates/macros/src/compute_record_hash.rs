use quote::quote;

use proc_macro2::{Span, TokenStream};
use solana_program::hash::hashv;
use syn::{LitByte, LitStr};

use crate::SPL_NAME_SERVICE_HASH_PREFIX;

pub fn process(item: TokenStream) -> TokenStream {
    let str: LitStr = syn::parse(item.into()).unwrap();
    let key = str.value();
    let hashed_array =
        hashv(&[format!("{}\x01{}", SPL_NAME_SERVICE_HASH_PREFIX, key).as_bytes()]).to_bytes();
    let hashed_bytes = hashed_array
        .iter()
        .map(|b| LitByte::new(*b, Span::call_site()));
    quote!(
        [#(#hashed_bytes),*]
    )
}
