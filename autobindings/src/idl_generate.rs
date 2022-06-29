use std::{fs::File, io::Read, str::FromStr};

use anchor_syn::idl::{IdlAccount, IdlAccountItem, IdlField, IdlInstruction, IdlType};
use syn::{
    AngleBracketedGenericArguments, Expr, ExprLit, Field, GenericArgument, Lit, Path,
    PathArguments, Type, TypeArray, TypePath,
};

use crate::{
    find_struct, get_constraints, get_simple_type, get_struct_fields, is_option, is_slice,
    snake_to_camel,
};

pub fn idl_process_file(
    module_name: &str,
    instruction_tag: usize,
    path: &str,
    use_casting: bool,
) -> IdlInstruction {
    let mut f = File::open(path).unwrap();
    let mut raw_string = String::new();
    f.read_to_string(&mut raw_string).unwrap();

    let ast: syn::File = syn::parse_str(&raw_string).unwrap();
    let accounts_struct_item = find_struct("Accounts", &ast);
    let params_struct_item = find_struct("Params", &ast);

    let params_fields = get_struct_fields(params_struct_item);
    let accounts_fields = get_struct_fields(accounts_struct_item);
    let mut instruction = IdlInstruction {
        name: module_name.to_owned(),
        accounts: Vec::with_capacity(accounts_fields.len()),
        args: Vec::with_capacity(params_fields.len()),
        returns: None,
    };
    for Field {
        attrs: _,
        vis: _,
        ident,
        colon_token: _,
        ty,
    } in params_fields
    {
        let camel_case_ident = snake_to_camel(&ident.as_ref().unwrap().to_string());
        instruction.args.push(IdlField {
            name: camel_case_ident,
            ty: type_to_idl(&ty),
        });
    }
    for Field {
        attrs,
        vis: _,
        ident,
        colon_token: _,
        ty,
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

fn js_type_assignment(ty: &Type, camel_case_ident: &str) -> String {
    match ty {
        Type::Path(_) => {
            let simple_type = get_simple_type(ty);
            match simple_type.as_ref() {
                "i8" | "i16" | "i32" => {
                    let bit_width = simple_type[1..].parse::<u8>().unwrap();
                    format!(
                        "this.{} = new BN(obj.{}).fromTwos({}).toNumber();",
                        camel_case_ident, camel_case_ident, bit_width
                    )
                }
                "i64" | "i128" => {
                    let bit_width = simple_type[1..].parse::<u8>().unwrap();
                    format!(
                        "this.{} = obj.{}.fromTwos({});",
                        camel_case_ident, camel_case_ident, bit_width
                    )
                }
                _ => format!("this.{} = obj.{};", camel_case_ident, camel_case_ident),
            }
        }
        Type::Array(TypeArray {
            bracket_token: _,
            elem,
            semi_token: _,
            len:
                Expr::Lit(ExprLit {
                    attrs: _,
                    lit: Lit::Int(_),
                }),
        }) => {
            let simple_type = get_simple_type(elem);
            match &simple_type as &str {
                "i8" | "i16" | "i32" => {
                    let bit_width = simple_type[1..].parse::<u8>().unwrap();
                    format!(
                        "this.{} = obj.{}.map(o => new BN(o).fromTwos({}).toNumber());",
                        camel_case_ident, camel_case_ident, bit_width
                    )
                }
                "i64" | "i128" => {
                    let bit_width = simple_type[1..].parse::<u8>().unwrap();
                    format!(
                        "this.{} = obj.{}.map(o => o.fromTwos({}));",
                        camel_case_ident, camel_case_ident, bit_width
                    )
                }
                _ => format!("this.{} = obj.{};", camel_case_ident, camel_case_ident),
            }
        }
        _ => unimplemented!(),
    }
}

fn array_to_js(inner_type: &str) -> String {
    match inner_type as &str {
        "\"u8\"" | "\"i8\"" => "Uint8Array",
        "\"u16\"" | "\"i16\"" | "\"u32\"" | "\"i32\"" => "number[]",
        "\"u64\"" | "\"i64\"" | "\"u128\"" | "\"i128\"" => "BN[]",
        _ => unimplemented!(),
    }
    .to_owned()
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
                            return IdlType::Vec(Box::new(inner_type));
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    };
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
