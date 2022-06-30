use std::{fs::File, io::Read, str::FromStr};

use anchor_syn::idl::{
    IdlAccount, IdlAccountItem, IdlField, IdlInstruction, IdlType, IdlTypeDefinition,
    IdlTypeDefinitionTy,
};
use syn::{
    AngleBracketedGenericArguments, Expr, ExprLit, Field, GenericArgument, Item, Lit, Path,
    PathArguments, Type, TypeArray, TypePath,
};

use crate::{find_struct, get_constraints, get_struct_fields, is_option, is_slice, snake_to_camel};

pub fn idl_process_file(module_name: &str, path: &str) -> IdlInstruction {
    let mut f = File::open(path).unwrap();
    let mut raw_string = String::new();
    f.read_to_string(&mut raw_string).unwrap();

    let ast: syn::File = syn::parse_str(&raw_string).unwrap();
    let accounts_struct_item = find_struct(&ast, Some("Accounts"));
    let params_struct_item = find_struct(&ast, Some("Params"));

    let params_fields = get_struct_fields(params_struct_item);
    let accounts_fields = get_struct_fields(accounts_struct_item);
    let mut instruction = IdlInstruction {
        name: module_name.to_owned(),
        accounts: Vec::with_capacity(accounts_fields.len()),
        args: Vec::with_capacity(params_fields.len()),
        returns: None,
    };
    for Field { ident, ty, .. } in params_fields {
        let camel_case_ident = snake_to_camel(&ident.as_ref().unwrap().to_string());
        instruction.args.push(IdlField {
            name: camel_case_ident,
            ty: type_to_idl(&ty),
        });
    }
    for Field {
        attrs, ty, ident, ..
    } in accounts_fields
    {
        let (writable, signer) = get_constraints(&attrs);
        let camel_case_ident = snake_to_camel(&ident.as_ref().unwrap().to_string());
        if is_slice(&ty) || is_option(&ty) {
            unimplemented!();
        } else {
            instruction
                .accounts
                .push(IdlAccountItem::IdlAccount(IdlAccount {
                    name: camel_case_ident,
                    is_mut: writable,
                    is_signer: signer,
                    pda: None,
                }));
        }
    }
    instruction
}

pub fn idl_process_state_file(path: &std::path::Path, skip_account_tag: bool) -> IdlTypeDefinition {
    let mut f = std::fs::File::open(path).unwrap();
    let mut raw_string = String::new();
    f.read_to_string(&mut raw_string).unwrap();

    let ast: syn::File = syn::parse_str(&raw_string).unwrap();
    let s = find_struct(&ast, None);

    let name = if let Item::Struct(s) = &s {
        s.ident.to_string()
    } else {
        unreachable!()
    };

    let struct_fields = get_struct_fields(s);

    let mut fields = Vec::with_capacity(struct_fields.len() + 1);
    if !skip_account_tag {
        fields.push(IdlField {
            name: String::from("AccountTag"),
            ty: IdlType::U64,
        });
    }

    for Field { ident, ty, .. } in struct_fields {
        fields.push(IdlField {
            name: snake_to_camel(&ident.unwrap().to_string()),
            ty: type_to_idl(&ty),
        })
    }

    IdlTypeDefinition {
        name,
        ty: IdlTypeDefinitionTy::Struct { fields },
    }
}

fn type_to_idl(ty: &Type) -> IdlType {
    match ty {
        Type::Path(TypePath {
            qself: _,
            path: Path {
                leading_colon: _,
                segments,
            },
        }) => {
            let segment = segments.iter().next().unwrap();
            let simple_type = segment.ident.to_string();
            match simple_type.as_ref() {
                "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128"
                | "String" | "Pubkey" => IdlType::from_str(&simple_type).unwrap(),
                "Vec" => {
                    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: _,
                        lt_token: _,
                        args,
                        gt_token: _,
                    }) = &segment.arguments
                    {
                        if let GenericArgument::Type(t) = args.first().unwrap() {
                            let inner_type = type_to_idl(t);
                            IdlType::Vec(Box::new(inner_type))
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                _ => IdlType::U8, // We assume this is an enum
            }
        }
        Type::Array(TypeArray {
            bracket_token: _,
            elem,
            semi_token: _,
            len:
                Expr::Lit(ExprLit {
                    attrs: _,
                    lit: Lit::Int(l),
                }),
        }) => {
            let inner_type = type_to_idl(elem);
            IdlType::Array(Box::new(inner_type), l.base10_parse::<usize>().unwrap())
        }
        _ => unimplemented!(),
    }
}
