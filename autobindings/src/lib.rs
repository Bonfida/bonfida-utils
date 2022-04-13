use convert_case::{Case, Casing};
use proc_macro2::TokenTree;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Expr, ExprLit, Field, Fields, FieldsNamed,
    Item, ItemEnum, ItemStruct, Lit, Path, Type, TypeArray, TypePath, TypeReference, Variant,
};

use crate::js_generate::js_process_file;
use crate::py_generate::py_process_file;

pub mod js_generate;
pub mod py_generate;
pub mod test;

#[derive(Debug, Clone, Copy)]
pub enum TargetLang {
    Javascript,
    Python,
}

pub fn generate(
    instructions_path: &str,
    instructions_enum_path: &str,
    target_lang: TargetLang,
    output_path: &str,
) {
    let path = std::path::Path::new(instructions_path);
    let (instruction_tags, use_casting) = parse_instructions_enum(instructions_enum_path);
    let directory = std::fs::read_dir(path).unwrap();
    let mut output = get_header(target_lang);
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
        let s = match target_lang {
            TargetLang::Javascript => js_process_file(
                &module_name,
                *instruction_tag,
                file.path().to_str().unwrap(),
                use_casting,
            ),
            TargetLang::Python => py_process_file(
                &module_name,
                *instruction_tag,
                file.path().to_str().unwrap(),
                use_casting,
            ),
        };
        output.push_str(&s);
    }

    let mut out_file = File::create(output_path).unwrap();
    out_file.write_all(output.as_bytes()).unwrap();
}

pub fn parse_instructions_enum(instructions_enum_path: &str) -> (HashMap<String, usize>, bool) {
    let mut f = File::open(instructions_enum_path).unwrap();
    let mut result_map = HashMap::new();
    let mut raw_string = String::new();
    f.read_to_string(&mut raw_string).unwrap();
    let use_casting = raw_string.contains("get_instruction_cast");
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
    (result_map, use_casting)
}

pub fn get_header(target_lang: TargetLang) -> String {
    match target_lang {
        TargetLang::Javascript => include_str!("templates/template.ts").to_string(),
        TargetLang::Python => include_str!("templates/template.py").to_string(),
    }
}

fn get_simple_type(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath {
            qself: _,
            path: Path {
                leading_colon: _,
                segments,
            },
        }) => segments.iter().next().unwrap().ident.to_string(),
        _ => unimplemented!(),
    }
}

fn padding_len(ty: &Type) -> u8 {
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
                "u8" => 1,
                "u16" => 2,
                "u32" => 4,
                "u64" => 8,
                "u128" => 16,
                _ => unimplemented!(), // padding should be of types given above
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
        }) => padding_len(elem) * l.base10_parse::<u8>().unwrap(),
        _ => unimplemented!(),
    }
}

fn snake_to_camel(s: &str) -> String {
    s.from_case(Case::Snake).to_case(Case::Camel)
}
fn snake_to_pascal(s: &str) -> String {
    s.from_case(Case::Snake).to_case(Case::Pascal)
}
fn pascal_to_snake(s: &str) -> String {
    s.from_case(Case::Pascal).to_case(Case::Snake)
}
fn lower_to_upper(s: &str) -> String {
    s.from_case(Case::Lower).to_case(Case::Upper)
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
