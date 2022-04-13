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
        "\tschema = borsh.CStruct(".to_owned(),
    ];
    let mut ser_input_statements = vec![];
    let mut schema_statements = vec![if use_casting {
        "\t\t\"tag\" / borsh.U64,"
    } else {
        "\t\t\"tag\" / borsh.U8,"
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

        if snake_case_ident == "_padding" {
            schema_statements.push(format!("\t\t\"padding\" / borsh.U8[{}],", padding_len(&ty)));
            ser_build_statements.push(format!("\t\t\t\"padding\": [0]*{}", padding_len(&ty)));
        } else {
            schema_statements.push(format!(
                "\t\t\"{}\" / {},",
                snake_case_ident.trim_start_matches('_'),
                type_to_borsh_py(&ty)
            ));
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
            keys_statements.push(format!("\t\tfor k in {}:", snake_case_ident));
            keys_statements.push("\t\t\tkeys.append(AccountMeta(k,".to_owned());
        } else if is_option(&ty) {
            get_instr_input_statements.push(format!("\t\t{}: PublicKey = None,", snake_case_ident));
            keys_statements.push(format!("\t\tif ({} is not None):", snake_case_ident));
            keys_statements.push(format!(
                "\t\t\tkeys.append(AccountMeta({},",
                snake_case_ident,
            ));
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
    statements.extend(ser_input_statements);
    statements.extend(get_instr_input_statements);
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

pub(crate) fn type_to_py(ty: &Type) -> String {
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
                "Pubkey" => "List[int]".to_owned(),
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
            let inner_type = type_to_py(elem);
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
                    "borsh.".to_owned() + lower_to_upper(&simple_type).as_str()
                }
                "String" => "borsh.String".to_owned(),
                "Pubkey" => "borsh.U8[32]".to_owned(),
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
                            return format!("borsh.Vec({})", &inner_type);
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    };
                }
                _ => "borsh.U8".to_owned(), // We assume this is an enum
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
            format!("{}[{}]", inner_type, l.base10_parse::<u8>().unwrap())
        }
        _ => unimplemented!(),
    }
}
