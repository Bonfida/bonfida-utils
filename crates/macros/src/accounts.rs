use std::convert::TryInto;

use proc_macro::{Span, TokenStream};
use proc_macro2::TokenTree;
use quote::{quote, ToTokens};
use syn::{
    token::Pub, Block, Generics, Ident, Stmt, Type, TypePath, TypeReference, TypeSlice, VisPublic,
    Visibility,
};

pub fn process(mut ast: syn::DeriveInput) -> TokenStream {
    ast.ident = Ident::new("AccountKeys", Span::call_site().into());
    ast.vis = Visibility::Public(VisPublic {
        pub_token: Pub(proc_macro2::Span::call_site()),
    });
    ast.generics = Generics::default();
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
        let mut function_body: Block =
            syn::parse(quote!({}).into()).unwrap_or_else(|_| panic!("{}", line!().to_string()));
        for n in named.into_iter() {
            let mut writable = false;
            let mut signer = false;
            for i in 0..n.attrs.len() {
                if n.attrs[i].path.is_ident("cons") {
                    let t = if let TokenTree::Group(g) = n.attrs[i]
                        .tokens
                        .clone()
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| panic!("{}", line!().to_string()))
                    {
                        g.stream()
                    } else {
                        panic!()
                    };

                    for constraint in t.into_iter() {
                        match constraint {
                            TokenTree::Ident(i) => {
                                if &i.to_string() == "writable" {
                                    writable = true;
                                }
                                if &i.to_string() == "signer" {
                                    signer = true;
                                }
                            }
                            TokenTree::Punct(p) if p.as_char() == ',' => {}
                            _ => {}
                        }
                    }
                    break;
                }
            }

            match n.ty.clone() {
                Type::Reference(TypeReference {
                    and_token: _,
                    lifetime: _,
                    mutability: _,
                    elem,
                }) => match *elem {
                    Type::Slice(TypeSlice {
                        elem: _,
                        bracket_token: _,
                    }) => {
                        function_body.stmts.push(account_push_expr_slice(
                            n.ident
                                .clone()
                                .unwrap_or_else(|| panic!("{}", line!().to_string())),
                            writable,
                            signer,
                        ));
                    }
                    _ => {
                        function_body.stmts.push(account_push_expr(
                            n.ident
                                .clone()
                                .unwrap_or_else(|| panic!("{}", line!().to_string())),
                            writable,
                            signer,
                        ));
                    }
                },
                Type::Path(TypePath { qself: _, path }) => {
                    let seg = path
                        .segments
                        .iter()
                        .next()
                        .unwrap_or_else(|| panic!("{}", line!().to_string()));
                    match seg.ident.to_string().as_str() {
                        "Option" => function_body.stmts.push(account_push_option(
                            n.ident
                                .clone()
                                .unwrap_or_else(|| panic!("{}", line!().to_string())),
                            writable,
                            signer,
                        )),
                        "Vec" => function_body.stmts.push(account_push_vec(
                            n.ident
                                .clone()
                                .unwrap_or_else(|| panic!("{}", line!().to_string())),
                            writable,
                            signer,
                        )),
                        _ => unimplemented!(),
                    }
                }
                _ => {
                    panic!()
                }
            }
        }
        let mut gen = proc_macro2::TokenStream::new();
        let function = quote!(
            use solana_program::instruction::{AccountMeta, Instruction};
            impl<'a> InstructionsAccount for Accounts<'a, Pubkey> {
                fn get_accounts_vec(&self) -> Vec<AccountMeta> {
                    let mut accounts_vec = Vec::new();
                    #function_body
                    accounts_vec
                }
            }
        );
        function.to_tokens(&mut gen);
        gen.into()
    } else {
        panic!()
    }
}

fn account_push_expr(ident: Ident, writable: bool, signer: bool) -> Stmt {
    let t: TokenStream = if writable {
        quote!(accounts_vec.push(AccountMeta::new(*self.#ident, #signer));).into()
    } else {
        quote!(accounts_vec.push(AccountMeta::new_readonly(*self.#ident, #signer));).into()
    };
    syn::parse(t).unwrap_or_else(|_| panic!("{}", line!().to_string()))
}

fn account_push_option(ident: Ident, writable: bool, signer: bool) -> Stmt {
    let t: TokenStream = if writable {
        quote!(
            if let Some(k) = self.#ident {
                accounts_vec.push(AccountMeta::new(*k, #signer));
            };
        )
        .try_into()
        .unwrap_or_else(|_| panic!("{}", line!().to_string()))
    } else {
        quote!(
            if let Some(k) = self.#ident {
                accounts_vec.push(AccountMeta::new_readonly(*k, #signer));
            };
        )
        .try_into()
        .unwrap_or_else(|_| panic!("{}", line!().to_string()))
    };
    syn::parse(t).unwrap_or_else(|_| panic!("{}", line!().to_string()))
}

fn account_push_vec(ident: Ident, writable: bool, signer: bool) -> Stmt {
    let t: TokenStream = if writable {
        quote!(
            accounts_vec.extend(#ident.iter().map(|k| AccountMeta::new(*k, #signer)));
        )
        .try_into()
        .unwrap_or_else(|_| panic!("{}", line!().to_string()))
    } else {
        quote!(
            accounts_vec.extend(self.#ident.iter().map(|k| AccountMeta::new_readonly(**k, #signer)));

        )
        .try_into()
        .unwrap_or_else(|_| panic!("{}", line!().to_string()))
    };
    syn::parse(t).unwrap_or_else(|_| panic!("{}", line!().to_string()))
}
fn account_push_expr_slice(ident: Ident, writable: bool, signer: bool) -> Stmt {
    let t: TokenStream = if writable {
        quote!(
            for k in self.#ident {
                accounts_vec.push(AccountMeta::new(*k, #signer));
            }
        )
        .try_into()
        .unwrap_or_else(|_| panic!("{}", line!().to_string()))
    } else {
        quote!(
            for k in self.#ident {
                accounts_vec.push(AccountMeta::new_readonly(*k, #signer));
            }
        )
        .try_into()
        .unwrap_or_else(|_| panic!("{}", line!().to_string()))
    };
    syn::parse(t).unwrap_or_else(|_| panic!("{}", line!().to_string()))
}
