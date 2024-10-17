use std::{fs::File, io::Read};

use syn::{
    AngleBracketedGenericArguments, Expr, ExprLit, Field, GenericArgument, Lit, Path,
    PathArguments, Type, TypeArray, TypePath,
};

use crate::{
    find_struct, get_constraints, get_struct_fields, is_option, is_slice, padding_len,
    snake_to_camel,
};

pub fn js_process_file(
    module_name: &str,
    instruction_tag: usize,
    path: &str,
    use_casting: bool,
) -> String {
    let mut f = File::open(path).unwrap();
    let mut raw_string = String::new();
    f.read_to_string(&mut raw_string).unwrap();

    let ast: syn::File = syn::parse_str(&raw_string).unwrap();
    let accounts_struct_item = find_struct(&ast, Some("Accounts"));
    let params_struct_item = find_struct(&ast, Some("Params"));

    let params_fields = get_struct_fields(params_struct_item);
    let accounts_fields = get_struct_fields(accounts_struct_item);
    let mut statements = vec![
        format!("export class {}Instruction {{", snake_to_camel(module_name)),
        if use_casting {
            "tag: bigint;"
        } else {
            "tag: number;"
        }
        .to_owned(),
    ];
    let mut declaration_statements = vec![];
    let mut schema_statements = vec![if use_casting {
        "tag: \"u64\","
    } else {
        "tag: \"u8\","
    }
    .to_owned()];
    let mut accounts_statements = vec!["programId: PublicKey,".to_owned()];
    let mut keys_statements = vec![];

    let mut assign_statements = vec![if use_casting {
        format!("this.tag = BigInt({});", instruction_tag)
    } else {
        format!("this.tag = {};", instruction_tag)
    }];
    for Field {
        attrs: _,
        vis: _,
        ident,
        colon_token: _,
        ty,
    } in params_fields
    {
        let camel_case_ident = snake_to_camel(&ident.as_ref().unwrap().to_string());
        schema_statements.push(format!("{}: {},", camel_case_ident, type_to_borsh_js(&ty)));
        if camel_case_ident == "padding" {
            declaration_statements.push("padding: Uint8Array;".to_owned());
            assign_statements.push(format!(
                "this.padding = (new Uint8Array({})).fill(0)",
                padding_len(&ty)
            ));
        } else {
            declaration_statements.push(format!("{}: {};", camel_case_ident, type_to_js(&ty)));
            assign_statements.push(js_type_assignment(&ty, &camel_case_ident));
        }
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
        if is_slice(&ty) {
            accounts_statements.push(format!("{}: PublicKey[],", camel_case_ident));
            keys_statements.push(format!("for (let k of {}) {{", camel_case_ident));
            keys_statements.push("keys.push({".to_owned());
            keys_statements.push("pubkey: k,".to_owned());
            keys_statements.push(format!("isSigner: {},", signer));
            keys_statements.push(format!("isWritable: {},", writable));
            keys_statements.push("});".to_owned());
            keys_statements.push("}".to_owned());
        } else if is_option(&ty) {
            accounts_statements.push(format!("{}?: PublicKey,", camel_case_ident));
            keys_statements.push(format!("if (!!{}) {{", camel_case_ident));
            keys_statements.push("keys.push({".to_owned());
            keys_statements.push(format!("pubkey: {},", camel_case_ident));
            keys_statements.push(format!("isSigner: {},", signer));
            keys_statements.push(format!("isWritable: {},", writable));
            keys_statements.push("});".to_owned());
            keys_statements.push("}".to_owned());
        } else {
            accounts_statements.push(format!("{}: PublicKey,", camel_case_ident));
            keys_statements.push("keys.push({".to_owned());
            keys_statements.push(format!("pubkey: {},", camel_case_ident));
            keys_statements.push(format!("isSigner: {},", signer));
            keys_statements.push(format!("isWritable: {},", writable));
            keys_statements.push("});".to_owned());
        }
    }
    statements.extend(declaration_statements.clone());
    statements.push("static schema = {".to_owned());
    statements.push("struct : {".to_owned());
    statements.extend(schema_statements);
    statements.push("},".to_owned());
    statements.push("};".to_owned());
    if declaration_statements.is_empty() {
        statements.push("constructor() {".to_owned());
    } else {
        statements.push("constructor(obj: {".to_owned());
        statements.extend({
            declaration_statements.retain(|e| !e.contains("padding"));
            declaration_statements
        });
        statements.push("}) {".to_owned());
    }
    statements.extend(assign_statements);
    statements.push("}".to_owned());

    statements.push("serialize(): Uint8Array {".to_owned());
    statements.push(format!(
        "return serialize({}Instruction.schema, this);",
        snake_to_camel(module_name)
    ));
    statements.push("}".to_owned());
    statements.push("getInstruction(".to_owned());
    statements.extend(accounts_statements);
    statements.push("): TransactionInstruction {".to_owned());
    statements.push("const data = Buffer.from(this.serialize());".to_owned());
    statements.push("let keys: AccountKey[] = [];".to_owned());
    statements.extend(keys_statements);
    statements.push("return new TransactionInstruction({".to_owned());
    statements.push("keys,".to_owned());
    statements.push("programId,".to_owned());
    statements.push("data,".to_owned());
    statements.push("});".to_owned());
    statements.push("}".to_owned());
    statements.push("}".to_owned());
    let mut out_string = String::new();
    for s in statements {
        out_string.push_str(&s);
        out_string.push('\n');
    }
    out_string
}

fn js_type_assignment(ty: &Type, camel_case_ident: &str) -> String {
    match ty {
        Type::Path(_) => format!("this.{} = obj.{};", camel_case_ident, camel_case_ident),
        Type::Array(TypeArray {
            bracket_token: _,
            elem: _,
            semi_token: _,
            len:
                Expr::Lit(ExprLit {
                    attrs: _,
                    lit: Lit::Int(_),
                }),
        }) => format!("this.{} = obj.{};", camel_case_ident, camel_case_ident),
        _ => unimplemented!(),
    }
}

fn type_to_js(ty: &Type) -> String {
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
                "bool" => "boolean".to_owned(),
                "u8" | "u16" | "u32" | "i8" | "i16" | "i32" => "number".to_owned(),
                "u64" | "u128" | "i64" | "i128" => "bigint".to_owned(),
                "String" => "string".to_owned(),
                "Pubkey" => "Uint8Array".to_owned(),
                "Vec" => {
                    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: _,
                        lt_token: _,
                        args,
                        gt_token: _,
                    }) = &segment.arguments
                    {
                        if let GenericArgument::Type(t) = args.first().unwrap() {
                            let inner_type = type_to_js(t);
                            format!("{}[]", &inner_type)
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                "Option" => {
                    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: _,
                        lt_token: _,
                        args,
                        gt_token: _,
                    }) = &segment.arguments
                    {
                        if let GenericArgument::Type(t) = args.first().unwrap() {
                            let inner_type = type_to_js(t);
                            format!("{} | null", &inner_type)
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                _ => "number".to_owned(), // We assume this is an enum
            }
        }
        Type::Array(TypeArray {
            bracket_token: _,
            elem,
            semi_token: _,
            len: _,
        }) => {
            let inner_type = type_to_borsh_js(elem);
            array_to_js(&inner_type)
        }
        _ => unimplemented!(),
    }
}

fn array_to_js(inner_type: &str) -> String {
    match inner_type as &str {
        "\"u8\"" | "\"i8\"" => "Uint8Array",
        "\"u16\"" | "\"i16\"" | "\"u32\"" | "\"i32\"" => "number[]",
        "\"u64\"" | "\"i64\"" | "\"u128\"" | "\"i128\"" => "bigint[]",
        _ => unimplemented!(),
    }
    .to_owned()
}

fn type_to_borsh_js(ty: &Type) -> String {
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
            let t = match simple_type.as_ref() {
                "u8" | "u16" | "u32" | "u64" | "u128" | "bool" => simple_type,
                "i8" | "i16" | "i32" | "i64" | "i128" => {
                    let mut res = "u".to_owned();
                    <String as std::fmt::Write>::write_str(&mut res, &simple_type[1..]).unwrap();
                    res
                }
                "String" => "string".to_owned(),
                "Pubkey" => return "{ array: { type: \"u8\", len: 32 } }".to_owned(),
                "Vec" => {
                    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: _,
                        lt_token: _,
                        args,
                        gt_token: _,
                    }) = &segment.arguments
                    {
                        if let GenericArgument::Type(t) = args.first().unwrap() {
                            let inner_type = type_to_borsh_js(t);
                            return format!("{{ array: {{ type: {} }} }}", &inner_type);
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    };
                }
                "Option" => {
                    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: _,
                        lt_token: _,
                        args,
                        gt_token: _,
                    }) = &segment.arguments
                    {
                        if let GenericArgument::Type(t) = args.first().unwrap() {
                            let inner_type = type_to_borsh_js(t);
                            return format!("{{ option: {}  }}", &inner_type);
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    };
                }
                _ => "u8".to_owned(), // We assume this is an enum
            };
            format!("\"{}\"", t)
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
            let inner_type = type_to_borsh_js(elem);
            let mut unsigned_type = "u".to_owned();
            <String as std::fmt::Write>::write_str(
                &mut unsigned_type,
                &inner_type[2..inner_type.len() - 1],
            )
            .unwrap();

            match &unsigned_type as &str {
                "u8" => format!("[{}]", l.base10_parse::<u8>().unwrap()),
                "u16" | "u32" | "u64" | "u128" => {
                    format!("[{}, {}]", inner_type, l.base10_parse::<u8>().unwrap())
                }
                _ => {
                    println!("{:?}", inner_type);
                    unimplemented!()
                }
            }
        }
        _ => unimplemented!(),
    }
}
