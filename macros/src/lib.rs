use proc_macro::TokenStream;
mod accounts;
mod pubkey;

#[proc_macro_attribute]
pub fn accounts(_: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    accounts::process(ast)
}

#[proc_macro]
pub fn pubkey(item: TokenStream) -> TokenStream {
    pubkey::process(item.into()).into()
}
