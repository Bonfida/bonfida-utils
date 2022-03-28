use std::{fs::File, io::Read};

use syn::{
    AngleBracketedGenericArguments, Expr, ExprLit, Field, GenericArgument, Lit, Path,
    PathArguments, Type, TypeArray, TypePath,
};

use crate::{
    find_struct, get_constraints, get_struct_fields, is_option, is_slice, lower_to_upper,
    padding_len, snake_to_pascal,
};

pub fn py_process_file(
    module_name: &str,
    instruction_tag: usize,
    path: &str,
    use_casting: bool,
) -> String {
    let mut f = File::open(path).unwrap();
    let mut raw_string = String::new();
    f.read_to_string(&mut raw_string).unwrap();

    let ast: syn::File = syn::parse_str(&raw_string).unwrap();
    let accounts_struct_item = find_struct("Accounts", &ast);
    let params_struct_item = find_struct("Params", &ast);

    let params_fields = get_struct_fields(params_struct_item);
    let accounts_fields = get_struct_fields(accounts_struct_item);
    let mut statements = vec![
        format!("class {}Instruction:", snake_to_pascal(module_name)),
        "\tschema = CStruct(".to_owned(),
    ];
    let mut ser_input_statements = vec![];
    let mut schema_statements = vec![if use_casting {
        "\t\t\"tag\" / U64,"
    } else {
        "\t\t\"tag\" / U8,"
    }
    .to_owned()];
    let mut get_instr_input_statements = vec!["programId: PublicKey,".to_owned()];
    let mut keys_statements = vec![];

    let mut ser_build_statements = vec![format!("\t\t\t\"tag\": {},", instruction_tag)];
    for Field {
        attrs: _,
        vis: _,
        ident,
        colon_token: _,
        ty,
    } in params_fields
    {
        let snake_case_ident = ident.unwrap().to_string();
        schema_statements.push(format!(
            "\t\t\"{}\" / {},",
            snake_case_ident,
            type_to_borsh_py(&ty)
        ));
        if snake_case_ident == "padding" {
            ser_build_statements.push(format!("\t\t\t\"padding\": [0]*{}", padding_len(&ty)));
        } else {
            ser_input_statements.push(format!("\t\t{}: {},", snake_case_ident, type_to_py(&ty)));
            ser_build_statements.push(format!(
                "\t\t\t\"{}\": {},",
                snake_case_ident, snake_case_ident
            ));
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
        let snake_case_ident = ident.as_ref().unwrap().to_string();
        if is_slice(&ty) {
            get_instr_input_statements.push(format!("{}: List[PublicKey],", snake_case_ident));
            keys_statements.push(format!("for k in {}:", snake_case_ident));
            keys_statements.push("\t\tkeys.append(AccountMeta(k,".to_owned());
        } else if is_option(&ty) {
            get_instr_input_statements.push(format!("{}: PublicKey = None", snake_case_ident));
            keys_statements.push(format!("if ({} is not None):", snake_case_ident));
            keys_statements.push(format!("\t\tkeys.append(AccountMeta({},", snake_case_ident,));
        } else {
            get_instr_input_statements.push(format!("{}: PublicKey,", snake_case_ident));
            keys_statements.push(format!("\t\tkeys.append(AccountMeta({},", snake_case_ident,));
        }
        keys_statements.push(format!(
            "\t\t\t{}, {}))",
            snake_to_pascal(&signer.to_string()),
            snake_to_pascal(&writable.to_string())
        ));
    }

    let min_ser_input_statements = ser_input_statements
        .iter()
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();

    statements.extend(schema_statements.into_iter());
    statements.push("\t)".to_owned());

    statements.push("\tdef serialize(self,".to_owned());
    statements.extend({
        ser_input_statements.retain(|e| !e.contains("padding"));
        ser_input_statements.to_owned()
    });
    statements.push("\t) -> str:".to_owned());
    statements.push("\t\treturn self.schema.build({".to_owned());
    statements.extend(ser_build_statements);
    statements.push("\t\t})".to_owned());

    statements.push("\tdef getInstruction(self,".to_owned());
    statements.extend(get_instr_input_statements);
    statements.extend(ser_input_statements);
    statements.push(") -> TransactionInstruction:".to_owned());
    statements.push("\t\tdata = self.serialize(".to_owned());
    statements.extend(
        min_ser_input_statements
            .iter()
            .map(|s| s.split(':').next().unwrap().to_string() + ",")
            .collect::<Vec<String>>(),
    );
    statements.push(")".to_owned());
    statements.push("\t\tkeys: List[AccountMeta] = []".to_owned());
    statements.extend(keys_statements);
    statements.push("\t\treturn TransactionInstruction(keys, programId, data)".to_owned());
    let mut out_string = String::new();
    for s in statements {
        out_string.push_str(&s);
        out_string.push('\n');
    }
    out_string
}

fn type_to_py(ty: &Type) -> String {
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
                "u8" | "u16" | "u32" | "i8" | "i16" | "i32" | "u64" | "u128" | "i64" | "i128" => {
                    "int".to_owned()
                }
                "String" => "str".to_owned(),
                "Pubkey" => "PublicKey".to_owned(),
                "Vec" => {
                    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: _,
                        lt_token: _,
                        args,
                        gt_token: _,
                    }) = &segment.arguments
                    {
                        if let GenericArgument::Type(t) = args.first().unwrap() {
                            let inner_type = type_to_py(t);
                            return format!("List[{}]", &inner_type);
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    };
                }
                _ => "int".to_owned(), // We assume this is an enum
            }
        }
        Type::Array(TypeArray {
            bracket_token: _,
            elem,
            semi_token: _,
            len: _,
        }) => {
            let inner_type = type_to_borsh_py(elem);
            format!("List[{}]", inner_type)
        }
        _ => unimplemented!(),
    }
}

fn type_to_borsh_py(ty: &Type) -> String {
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
                "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" => {
                    lower_to_upper(&simple_type)
                }
                "String" => simple_type,
                "Pubkey" => "U8[32]".to_owned(),
                "Vec" => {
                    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: _,
                        lt_token: _,
                        args,
                        gt_token: _,
                    }) = &segment.arguments
                    {
                        if let GenericArgument::Type(t) = args.first().unwrap() {
                            let inner_type = type_to_borsh_py(t);
                            return format!("Vec({})", &inner_type);
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    };
                }
                _ => "U8".to_owned(), // We assume this is an enum
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
            let inner_type = type_to_borsh_py(elem);
            let mut unsigned_type = "u".to_owned();
            <String as std::fmt::Write>::write_str(
                &mut unsigned_type,
                &inner_type[2..inner_type.len() - 1],
            )
            .unwrap();

            //TODO
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
