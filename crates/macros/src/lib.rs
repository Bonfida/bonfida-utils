use proc_macro::TokenStream;
mod accounts;
mod borsh_size;
mod compute_record_hash;
mod compute_record_hash_v2;
mod declare_id_with_central_state;
mod wrapped_pod;

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

#[proc_macro_derive(WrappedPod)]
pub fn derive_wrapped_pod(item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    wrapped_pod::process(ast, false).into()
}

#[proc_macro_derive(WrappedPodMut)]
pub fn derive_wrapped_pod_mut(item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    wrapped_pod::process(ast, true).into()
}

#[proc_macro]
pub fn declare_id_with_central_state(item: TokenStream) -> TokenStream {
    declare_id_with_central_state::process(item.into()).into()
}

#[proc_macro]
pub fn compute_hashv(item: TokenStream) -> TokenStream {
    compute_record_hash::process(item.into()).into()
}

#[proc_macro]
pub fn compute_record_hash_v2(item: TokenStream) -> TokenStream {
    compute_record_hash_v2::process(item.into()).into()
}
