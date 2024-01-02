use quote::quote;
use spl_name_service::state::HASH_PREFIX;

use proc_macro2::{Span, TokenStream};
use solana_program::hash::hashv;
use syn::{LitByte, LitStr};

pub fn process(item: TokenStream) -> TokenStream {
    let str: LitStr = syn::parse(item.into()).unwrap();
    let key = str.value();
    let hashed_array = hashv(&[format!("{}\x02{}", HASH_PREFIX, key).as_bytes()]).to_bytes();
    let hashed_bytes = hashed_array
        .iter()
        .map(|b| LitByte::new(*b, Span::call_site()));
    quote!(
        [#(#hashed_bytes),*]
    )
}
