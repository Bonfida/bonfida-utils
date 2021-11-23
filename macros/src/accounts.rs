use proc_macro::{Span, TokenStream};
use proc_macro2::TokenTree;
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated,
    token::{Comma, Pub},
    Block, FnArg, Generics, Ident, Stmt, Type, TypeReference, TypeSlice, VisPublic, Visibility,
};

pub fn process(mut ast: syn::DeriveInput) -> TokenStream {
    ast.ident = Ident::new("AccountKeys", Span::call_site().into());
    ast.vis = Visibility::Public(VisPublic {
        pub_token: Pub(proc_macro2::Span::call_site()),
    });
    let mut contains_slice = false;
    let generic = ast.generics.params[0].clone();
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
        let mut function_arguments: Punctuated<_, Comma> = Punctuated::new();
        let mut function_body: Block = syn::parse(quote!({}).into()).unwrap();
        let key_arg: FnArg = syn::parse(quote!(foo: Pubkey).into()).unwrap();
        let slice_arg: FnArg = syn::parse(quote!(foo: &[Pubkey]).into()).unwrap();
        for n in named.into_iter() {
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
        let mut gen = proc_macro2::TokenStream::new();
        let function = quote!(
            use solana_program::instruction::{AccountMeta, Instruction};
            impl<'a> InstructionsAccount for Accounts<'a, Pubkey> {
                fn get_instruction<P: BorshDeserialize + BorshSerialize + BorshSize>(&self, instruction_id: u8, params: P) -> Instruction {
                    let mut accounts_vec = Vec::new();
                    let cap = 1 + params.borsh_len();
                    let mut data = Vec::with_capacity(cap);
                    unsafe {
                        data.set_len(cap);
                    }
                    data[0] = instruction_id;
                    params.serialize(&mut (&mut data[1..])).unwrap();

                    #function_body
                    Instruction {
                        program_id: crate::ID,
                        accounts: accounts_vec,
                        data
                    }
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
    syn::parse(t).unwrap()
}
fn account_push_expr_slice(ident: Ident, writable: bool, signer: bool) -> Stmt {
    let t: TokenStream = if writable {
        quote!(
            for k in self.#ident {
                accounts_vec.push(AccountMeta::new(*k, #signer));
            }
        )
        .into()
    } else {
        quote!(
            for k in self.#ident {
                accounts_vec.push(AccountMeta::new_readonly(*k, #signer));
            }
        )
        .into()
    };
    syn::parse(t).unwrap()
}
