use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

use convert_case::{Case, Casing};
use proc_macro2::TokenTree;
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Expr, ExprLit, Field, Fields, FieldsNamed,
    Item, ItemEnum, ItemStruct, Lit, Path, Type, TypeArray, TypePath, TypeReference, Variant,
};

const HEADER: &str = include_str!("templates/template.ts");

pub fn generate(instructions_path: &str, instructions_enum_path: &str, output_path: &str) {
    let path = std::path::Path::new(instructions_path);
    let instruction_tags = parse_instructions_enum(instructions_enum_path);
    let directory = std::fs::read_dir(path).unwrap();
    let mut output = get_header();
    for d in directory {
        let file = d.unwrap();
        let module_name = std::path::Path::new(&file.file_name())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let instruction_tag = instruction_tags
            .get(&module_name)
            .unwrap_or_else(|| panic!("Instruction not found for {}", module_name));
        let s = process_file(
            &module_name,
            *instruction_tag,
            file.path().to_str().unwrap(),
        );
        output.push_str(&s);
    }

    let mut out_file = File::create(output_path).unwrap();
    out_file.write_all(output.as_bytes()).unwrap();
}

pub fn parse_instructions_enum(instructions_enum_path: &str) -> HashMap<String, usize> {
    let mut f = File::open(instructions_enum_path).unwrap();
    let mut result_map = HashMap::new();
    let mut raw_string = String::new();
    f.read_to_string(&mut raw_string).unwrap();
    let ast: syn::File = syn::parse_str(&raw_string).unwrap();
    let instructions_enum = find_enum(&ast);
    let enum_variants = get_enum_variants(instructions_enum);
    for (
        i,
        Variant {
            attrs: _,
            ident,
            fields: _,
            discriminant: _,
        },
    ) in enum_variants.into_iter().enumerate()
    {
        let module_name = pascal_to_snake(&ident.to_string());
        result_map.insert(module_name, i);
    }
    result_map
}

pub fn get_header() -> String {
    HEADER.to_owned()
}

pub fn process_file(module_name: &str, instruction_tag: usize, path: &str) -> String {
    let mut f = File::open(path).unwrap();
    let mut raw_string = String::new();
    f.read_to_string(&mut raw_string).unwrap();

    let ast: syn::File = syn::parse_str(&raw_string).unwrap();
    let accounts_struct_item = find_struct("Accounts", &ast);
    let params_struct_item = find_struct("Params", &ast);

    let params_fields = get_struct_fields(params_struct_item);
    let accounts_fields = get_struct_fields(accounts_struct_item);
    let mut statements = vec![
        format!("export class {}Instruction {{", snake_to_camel(module_name)),
        "tag: number;".to_owned(),
    ];
    let mut declaration_statements = vec![];
    let mut assign_statements = vec![format!("this.tag = {}", instruction_tag)];
    let mut schema_statements = vec!["[\"tag\", \"u8\"],".to_owned()];
    let mut accounts_statements = vec!["programId: PublicKey,".to_owned()];
    let mut keys_statements = vec![];
    for Field {
        attrs: _,
        vis: _,
        ident,
        colon_token: _,
        ty,
    } in params_fields
    {
        let camel_case_ident = snake_to_camel(&ident.as_ref().unwrap().to_string());
        declaration_statements.push(format!("{}: {};", camel_case_ident, type_to_js(&ty)));
        schema_statements.push(format!(
            "[\"{}\", {}],",
            camel_case_ident,
            type_to_borsh(&ty)
        ));
        assign_statements.push(format!(
            "this.{} = obj.{};",
            camel_case_ident, camel_case_ident
        ));
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
    statements.push("static schema: Schema = new Map([".to_owned());
    statements.push("[".to_owned());
    statements.push(format!("{}Instruction,", snake_to_camel(module_name)));
    statements.push("{".to_owned());
    statements.push("kind: \"struct\",".to_owned());
    statements.push("fields: [".to_owned());
    statements.extend(schema_statements.into_iter());
    statements.push("],".to_owned());
    statements.push("},".to_owned());
    statements.push("],".to_owned());
    statements.push("]);".to_owned());
    statements.push("constructor(obj: {".to_owned());
    statements.extend(declaration_statements);
    statements.push("}) {".to_owned());
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

fn type_to_js(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath {
            qself: _,
            path: Path {
                leading_colon: _,
                segments,
            },
        }) => {
            let simple_type = segments.iter().next().unwrap().ident.to_string();
            match simple_type.as_ref() {
                "u8" | "u16" | "u32" | "i8" | "i16" | "i32" => "number".to_owned(),
                "u64" | "u128" | "i64" | "i128" => "BN".to_owned(),
                "String" => "string".to_owned(),
                _ => "number".to_owned(), // We assume this is an enum
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
            let inner_type = type_to_borsh(elem);
            match &inner_type as &str {
                "\"u8\"" | "\"i8\"" => "Uint8Array",
                "\"u16\"" | "\"i16\"" | "\"u32\"" | "\"i32\"" => "number[]",
                "\"u64\"" | "\"i64\"" | "\"u128\"" | "\"i128\"" => "BN[]",
                _ => unimplemented!(),
            }
            .to_owned()
        }
        _ => unimplemented!(),
    }
}

fn type_to_borsh(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath {
            qself: _,
            path: Path {
                leading_colon: _,
                segments,
            },
        }) => {
            let simple_type = segments.iter().next().unwrap().ident.to_string();
            let t = match simple_type.as_ref() {
                "u8" | "u16" | "u32" | "u64" | "u128" => &simple_type,
                "i8" | "i16" | "i32" | "i64" | "i128" => unimplemented!(),
                "String" => "string",
                _ => "u8", // We assume this is an enum
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
            let inner_type = type_to_borsh(elem);

            match &inner_type as &str {
                "\"u8\"" => format!("[{}]", l.base10_parse::<u8>().unwrap()),
                "\"u16\"" | "\"u32\"" | "\"u64\"" | "\"u128\"" => {
                    format!("[{}, {}]", inner_type, l.base10_parse::<u8>().unwrap())
                }
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    }
}

fn snake_to_camel(s: &str) -> String {
    s.from_case(Case::Snake).to_case(Case::Camel)
}
fn pascal_to_snake(s: &str) -> String {
    s.from_case(Case::Pascal).to_case(Case::Snake)
}

fn find_struct(ident_str: &str, file_ast: &syn::File) -> Item {
    file_ast
        .items
        .iter()
        .find(|a| {
            if let Item::Struct(ItemStruct {
                ident,
                attrs: _,
                vis: _,
                struct_token: _,
                generics: _,
                fields: _,
                semi_token: _,
            }) = a
            {
                *ident == ident_str
            } else {
                false
            }
        })
        .unwrap()
        .clone()
}

fn find_enum(file_ast: &syn::File) -> Item {
    file_ast
        .items
        .iter()
        .find(|a| matches!(a, Item::Enum(_)))
        .unwrap()
        .clone()
}

fn get_enum_variants(s: Item) -> Punctuated<Variant, Comma> {
    if let Item::Enum(ItemEnum {
        attrs: _,
        vis: _,
        enum_token: _,
        ident: _,
        generics: _,
        brace_token: _,
        variants,
    }) = s
    {
        variants
    } else {
        unreachable!()
    }
}

fn get_struct_fields(s: Item) -> Punctuated<Field, Comma> {
    if let Item::Struct(ItemStruct {
        ident: _,
        attrs: _,
        vis: _,
        struct_token: _,
        generics: _,
        fields:
            Fields::Named(FieldsNamed {
                named,
                brace_token: _,
            }),
        semi_token: _,
    }) = s
    {
        named
    } else {
        unreachable!()
    }
}

fn get_constraints(attrs: &[Attribute]) -> (bool, bool) {
    let mut writable = false;
    let mut signer = false;
    for a in attrs {
        if a.path.is_ident("cons") {
            let t = if let TokenTree::Group(g) = a.tokens.clone().into_iter().next().unwrap() {
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
    (writable, signer)
}

fn is_slice(ty: &Type) -> bool {
    if let Type::Reference(TypeReference {
        and_token: _,
        lifetime: _,
        mutability: _,
        elem,
    }) = ty
    {
        let ty = *elem.clone();
        if let Type::Slice(_) = ty {
            return true;
        }
    }
    false
}

fn is_option(ty: &Type) -> bool {
    if let Type::Path(TypePath { qself: _, path }) = ty {
        let seg = path.segments.iter().next().unwrap();
        if seg.ident != "Option" {
            unimplemented!()
        }
        return true;
    }
    false
}
