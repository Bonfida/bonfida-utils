use proc_macro::TokenStream;
mod accounts;
mod borsh_size;

#[proc_macro_derive(InstructionsAccount, attributes(cons))]
pub fn derive_instructions_account(item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    accounts::process(ast)
}

#[proc_macro_derive(BorshSize, attributes(cons))]
pub fn derive_borsh_size(item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    borsh_size::process(ast).into()
}
