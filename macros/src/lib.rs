use proc_macro::TokenStream;
mod accounts;
mod borsh_size;
mod declare_id_with_central_state;
mod instruction_params;

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

#[proc_macro_derive(InstructionParams)]
pub fn derive_instruction_params(item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    instruction_params::process(ast).into()
}

#[proc_macro]
pub fn declare_id_with_central_state(item: TokenStream) -> TokenStream {
    declare_id_with_central_state::process(item.into()).into()
}
