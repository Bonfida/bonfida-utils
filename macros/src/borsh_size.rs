use proc_macro2::TokenStream;
use quote::quote;
use syn::{Type, TypeArray, Variant};

pub fn process(mut ast: syn::DeriveInput) -> TokenStream {
    let struct_ident = ast.ident;
    match &mut ast.data {
        syn::Data::Struct(syn::DataStruct {
            struct_token: t,
            fields:
                syn::Fields::Named(syn::FieldsNamed {
                    brace_token: _,
                    named,
                }),
            semi_token: _,
        }) => {
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
            t
        }
        syn::Data::Enum(syn::DataEnum {
            enum_token: _,
            brace_token: _,
            variants,
        }) => {
            for Variant {
                attrs: _,
                ident: _,
                fields,
                discriminant: _,
            } in variants
            {
                if !fields.is_empty() {
                    unimplemented!("This derive macro only supports field-less enums. The BorshSize trait should be manually implemented.")
                }
            }
            let t = quote!(
                impl BorshSize for #struct_ident {
                    fn borsh_len(&self) -> usize {
                        1
                    }
                }
            );
            t
        }
        _ => unimplemented!(),
    }
}
