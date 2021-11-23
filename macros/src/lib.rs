use proc_macro::{Span, TokenStream};
use proc_macro2::TokenTree;
use quote::{quote, ToTokens};
use solana_program::pubkey::Pubkey;
use syn::{
    braced,
    punctuated::Punctuated,
    token::{Comma, Pub, Semi},
    Block, ExprBlock, Field, FnArg, Generics, Ident, PatType, Stmt, Token, Type, TypeReference,
    TypeSlice, VisPublic, Visibility,
};

#[proc_macro_attribute]
pub fn accounts(_: TokenStream, mut item: TokenStream) -> TokenStream {
    let ast = syn::parse(item.clone()).unwrap();
    impl_accounts(ast)
}

fn impl_accounts(mut ast: syn::DeriveInput) -> TokenStream {
    let mut original = ast.clone();
    ast.ident = Ident::new("AccountKeys", Span::call_site().into());
    ast.vis = Visibility::Public(VisPublic {
        pub_token: Pub(proc_macro2::Span::call_site()),
    });
    let mut contains_slice = false;
    let generic = ast.generics.params[0].clone();
    ast.generics = Generics::default();
    if let (
        syn::Data::Struct(syn::DataStruct {
            struct_token: _,
            fields:
                syn::Fields::Named(syn::FieldsNamed {
                    brace_token: _,
                    named,
                }),
            semi_token: _,
        }),
        syn::Data::Struct(syn::DataStruct {
            struct_token: _,
            fields:
                syn::Fields::Named(syn::FieldsNamed {
                    brace_token: _,
                    named: named_original,
                }),
            semi_token: _,
        }),
    ) = (&mut ast.data, &mut original.data)
    {
        let mut function_arguments: Punctuated<_, Comma> = Punctuated::new();
        let mut function_body: Block = syn::parse(quote!({}).into()).unwrap();
        let key_arg: FnArg = syn::parse(quote!(foo: Pubkey).into()).unwrap();
        let slice_arg: FnArg = syn::parse(quote!(foo: &[Pubkey]).into()).unwrap();
        for (n, n_original) in named.into_iter().zip(named_original.into_iter()) {
            let mut writable = false;
            let mut signer = false;
            for i in 0..n.attrs.len() {
                if n.attrs[i].path.is_ident("cons") {
                    let t = if let TokenTree::Group(g) =
                        n.attrs[i].tokens.clone().into_iter().next().unwrap()
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
                    n.attrs.remove(i);
                    n_original.attrs.remove(i);
                    break;
                }
            }

            n.ty = if let Type::Reference(TypeReference {
                and_token,
                lifetime,
                mutability,
                elem,
            }) = n.ty.clone()
            {
                match *elem {
                    Type::Slice(TypeSlice {
                        elem: _,
                        bracket_token,
                    }) => {
                        contains_slice = true;
                        let arg = if let FnArg::Typed(mut p) = slice_arg.clone() {
                            p.pat = Box::new(syn::parse(n.ident.to_token_stream().into()).unwrap());
                            FnArg::Typed(p)
                        } else {
                            panic!()
                        };
                        function_arguments.push(arg);
                        function_body.stmts.push(account_push_expr_slice(
                            n.ident.clone().unwrap(),
                            writable,
                            signer,
                        ));
                        Type::Reference(TypeReference {
                            and_token,
                            lifetime,
                            mutability,
                            elem: Box::new(Type::Slice(TypeSlice {
                                elem: Box::new(Type::Verbatim(quote!(Pubkey))),
                                bracket_token,
                            })),
                        })
                    }
                    _ => {
                        let arg = if let FnArg::Typed(mut p) = key_arg.clone() {
                            p.pat = Box::new(syn::parse(n.ident.to_token_stream().into()).unwrap());
                            FnArg::Typed(p)
                        } else {
                            panic!()
                        };
                        function_arguments.push(arg);
                        function_body.stmts.push(account_push_expr(
                            n.ident.clone().unwrap(),
                            writable,
                            signer,
                        ));
                        Type::Verbatim(quote!(Pubkey))
                    }
                }
            } else {
                panic!()
            }
        }
        if contains_slice {
            ast.generics.params.push(generic);
        }
        let mut gen = original.into_token_stream();
        ast.to_tokens(&mut gen);
        let function = quote!(
            use solana_program::instruction::{Instruction, AccountMeta};
            pub fn get_instruction(instruction_id: u8, accounts: AccountKeys, params: Params) -> Instruction {
                let mut accounts_vec = Vec::new();
                let mut data = vec![instruction_id];
                data.extend(&params.try_to_vec().unwrap());

                #function_body
                Instruction {
                    program_id: crate::ID,
                    accounts: accounts_vec,
                    data
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
        quote!(accounts_vec.push(AccountMeta::new(accounts.#ident, #signer));).into()
    } else {
        quote!(accounts_vec.push(AccountMeta::new_readonly(accounts.#ident, #signer));).into()
    };
    syn::parse(t).unwrap()
}
fn account_push_expr_slice(ident: Ident, writable: bool, signer: bool) -> Stmt {
    let t: TokenStream = if writable {
        quote!(
            for k in accounts.#ident {
                accounts_vec.push(AccountMeta::new(*k, #signer));
            }
        )
        .into()
    } else {
        quote!(
            for k in accounts.#ident {
                accounts_vec.push(AccountMeta::new_readonly(*k, #signer));
            }
        )
        .into()
    };
    syn::parse(t).unwrap()
}
