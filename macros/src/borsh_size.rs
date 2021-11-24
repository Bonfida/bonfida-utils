use proc_macro2::TokenStream;
use quote::quote;
use syn::{Type, TypeArray};

pub fn process(mut ast: syn::DeriveInput) -> TokenStream {
    let struct_ident = ast.ident;
    if let syn::Data::Struct(syn::DataStruct {
        struct_token: _,
        fields:
            syn::Fields::Named(syn::FieldsNamed {
                brace_token: _,
                named,
            }),
        semi_token: _,
    }) = &mut ast.data
    {
        let vars = named.into_iter().map(|n| {
            let field_ident = n.ident.clone().unwrap();
            match n.ty.clone() {
                Type::Array(TypeArray {
                    bracket_token: _,
                    elem: _,
                    semi_token: _,
                    len,
                }) => quote!((#len) * self.#field_ident[0].borsh_len()),
                Type::Path(_) => quote!(self.#field_ident.borsh_len()),
                Type::Tuple(_) => todo!(),
                _ => panic!(),
            }
        });
        let formula = vars.chain(Some(quote!(0)));
        let t = quote!(
            impl BorshSize for #struct_ident {
                fn borsh_len(&self) -> usize {
                    #(#formula)+*
                }
            }
        );
        return t;
    }
    panic!()
}
