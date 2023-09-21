use anchor_syn::idl::Idl;
use cargo_toml::Manifest;
use clap::{crate_name, crate_version, Arg, ArgMatches, Command};
use convert_case::{Boundary, Case, Casing};
use idl_generate::{idl_process_file, idl_process_state_file};
use proc_macro2::TokenTree;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    str::FromStr,
    time::Instant,
};

use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Expr, ExprLit, Field, Fields, FieldsNamed,
    Item, ItemEnum, ItemStruct, Lit, Path, Type, TypeArray, TypePath, TypeReference, Variant,
};

use crate::js_generate::js_process_file;
use crate::py_generate::py_process_file;

pub mod idl_generate;
pub mod js_generate;
pub mod py_generate;
pub mod test;

#[derive(Debug, Clone, Copy)]
pub enum TargetLang {
    Javascript,
    Python,
    AnchorIdl,
}

pub fn command() -> Command<'static> {
    Command::new(crate_name!())
        .version(crate_version!())
        .about("Autogenerate Rust and JS instruction bindings")
        .author("Bonfida")
        .arg(
            Arg::with_name("instr-path")
                .long("instructions-path")
                .takes_value(true)
                .default_value("src/processor"),
        )
        .arg(
            Arg::with_name("toml-path")
                .long("cargo-toml-path")
                .takes_value(true)
                .default_value("Cargo.toml"),
        )
        .arg(
            Arg::with_name("instr-enum-path")
                .long("instructions-enum-path")
                .takes_value(true)
                .default_value("src/instruction.rs"),
        )
        .arg(
            Arg::with_name("account-tag-enum-path")
                .long("account-tag-enum-path")
                .takes_value(true)
                .default_value("src/state.rs"),
        )
        .arg(
            Arg::with_name("state-folder")
                .long("state-folder")
                .takes_value(true)
                .default_value("src/state"),
        )
        .arg(
            Arg::with_name("target-lang")
                .long("target-language")
                .takes_value(true)
                .default_value("js")
                .help("Enter \"py\", \"js\" or \"idl\""),
        )
        .arg(
            Arg::with_name("test")
                .long("test")
                .takes_value(true)
                .default_value("false")
                .help("Enter true or false"),
        )
        .arg(
            Arg::with_name("skip-account-tag")
                .long("skip-account-tag")
                .takes_value(false),
        )
}

pub fn process(matches: &ArgMatches) {
    let instructions_path = matches.value_of("instr-path").unwrap();
    let instructions_enum_path = matches.value_of("instr-enum-path").unwrap();
    let cargo_toml_path = matches.value_of("toml-path").unwrap();
    let target_lang_str = matches.value_of("target-lang").unwrap();
    let state_folder = matches.value_of("state-folder").unwrap();
    let skip_account_tag = matches.contains_id("skip-account-tag");
    let target_lang = match target_lang_str {
        "js" | "javascript" => TargetLang::Javascript,
        "py" | "python" => TargetLang::Python,
        "idl" | "anchor-idl" => TargetLang::AnchorIdl,
        _ => {
            println!("Target language must be javascript or python");
            panic!()
        }
    };
    let test_mode = bool::from_str(matches.value_of("test").unwrap()).unwrap();
    fs::create_dir_all("../js/src/").unwrap();
    fs::create_dir_all("../python/src/").unwrap();

    let now = Instant::now();

    match test_mode {
        true => {
            test::test(instructions_path);
        }
        false => {
            generate(
                cargo_toml_path,
                instructions_path,
                instructions_enum_path,
                state_folder,
                target_lang,
                match target_lang {
                    TargetLang::Javascript => "../js/src/raw_instructions.ts",
                    TargetLang::Python => "../python/src/raw_instructions.py",
                    TargetLang::AnchorIdl => "idl.json",
                },
                skip_account_tag,
            );
        }
    }

    let elapsed = now.elapsed();
    println!("✨  Done in {:.2?}", elapsed);
}

pub fn generate(
    cargo_toml_path: &str,
    instructions_path: &str,
    instructions_enum_path: &str,
    state_folder_path: &str,
    target_lang: TargetLang,
    output_path: &str,
    skip_account_tag: bool,
) {
    let path = std::path::Path::new(instructions_path);
    let (instruction_tags, use_casting) = parse_instructions_enum(instructions_enum_path);
    let directory = std::fs::read_dir(path).unwrap();
    let manifest = Manifest::from_path(cargo_toml_path).unwrap();
    let state_directory = std::fs::read_dir(std::path::Path::new(state_folder_path)).unwrap();
    let mut output = get_header(target_lang);
    let mut idl = Idl {
        version: manifest.package.as_ref().unwrap().version.clone(),
        name: manifest.package.as_ref().unwrap().name.clone(),
        constants: vec![],
        instructions: vec![],
        state: None,
        accounts: vec![],
        types: vec![],
        events: None,
        errors: None,
        metadata: None,
    };
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
        match target_lang {
            TargetLang::Javascript => {
                let s = js_process_file(
                    &module_name,
                    *instruction_tag,
                    file.path().to_str().unwrap(),
                    use_casting,
                );
                output.push_str(&s)
            }
            TargetLang::Python => {
                let s = py_process_file(
                    &module_name,
                    *instruction_tag,
                    file.path().to_str().unwrap(),
                    use_casting,
                );
                output.push_str(&s)
            }
            TargetLang::AnchorIdl => {
                let i = idl_process_file(&module_name, file.path().to_str().unwrap());
                idl.instructions.push(i)
            }
        };
    }

    if matches!(target_lang, TargetLang::AnchorIdl) {
        for d in state_directory {
            let file = d.unwrap();
            let account = idl_process_state_file(&file.path(), skip_account_tag);
            idl.accounts.push(account);
        }
        output.push_str(&serde_json::to_string_pretty(&idl).unwrap())
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
    let instructions_enum = find_enum(&ast, None);
    let enum_variants = get_enum_variants(instructions_enum);
    for (i, Variant { ident, .. }) in enum_variants.into_iter().enumerate() {
        let module_name = pascal_to_snake(&ident.to_string());
        result_map.insert(module_name, i);
    }
    (result_map, use_casting)
}

pub fn parse_account_tag_enum(account_tag_enum_path: &str) -> HashMap<String, usize> {
    let mut f = File::open(account_tag_enum_path).unwrap();
    let mut result_map = HashMap::new();
    let mut raw_string = String::new();
    f.read_to_string(&mut raw_string).unwrap();
    let ast: syn::File = syn::parse_str(&raw_string).unwrap();
    let account_tag_enum = find_enum(&ast, Some("AccountTag"));
    let enum_variants = get_enum_variants(account_tag_enum);
    for (i, Variant { ident, .. }) in enum_variants.into_iter().enumerate() {
        let module_name = pascal_to_snake(&ident.to_string());
        result_map.insert(module_name, i);
    }
    result_map
}

pub fn get_header(target_lang: TargetLang) -> String {
    match target_lang {
        TargetLang::Javascript => include_str!("templates/template.ts").to_string(),
        TargetLang::Python => include_str!("templates/template.py").to_string(),
        TargetLang::AnchorIdl => String::new(),
    }
}

#[allow(dead_code)]
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
            path: Path { segments, .. },
            ..
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
            elem,
            len: Expr::Lit(ExprLit {
                lit: Lit::Int(l), ..
            }),
            ..
        }) => padding_len(elem) * l.base10_parse::<u8>().unwrap(),
        _ => unimplemented!(),
    }
}

fn snake_to_camel(s: &str) -> String {
    s.from_case(Case::Snake).
        without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit]).
        to_case(Case::Camel)
}

fn snake_to_pascal(s: &str) -> String {
    s.from_case(Case::Snake).
        without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit]).
        to_case(Case::Pascal)
}

fn pascal_to_snake(s: &str) -> String {
    s.from_case(Case::Pascal).
        without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit]).
        to_case(Case::Snake)
}

fn lower_to_upper(s: &str) -> String {
    s.from_case(Case::Lower).
        without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit]).
        to_case(Case::Upper)
}

fn find_struct(file_ast: &syn::File, ident_str: Option<&str>) -> Item {
    file_ast
        .items
        .iter()
        .find(|a| {
            if let Item::Struct(ItemStruct { ident, .. }) = a {
                ident_str.map(|s| *ident == s).unwrap_or(true)
            } else {
                false
            }
        })
        .unwrap()
        .clone()
}

fn find_enum(file_ast: &syn::File, ident_name: Option<&str>) -> Item {
    file_ast
        .items
        .iter()
        .find(|a| {
            if let Item::Enum(i) = a {
                ident_name.map(|s| i.ident == s).unwrap_or(true)
            } else {
                false
            }
        })
        .unwrap()
        .clone()
}

fn get_enum_variants(s: Item) -> Punctuated<Variant, Comma> {
    if let Item::Enum(ItemEnum { variants, .. }) = s {
        variants
    } else {
        unreachable!()
    }
}

fn get_struct_fields(s: Item) -> Punctuated<Field, Comma> {
    if let Item::Struct(ItemStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
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
    if let Type::Reference(TypeReference { elem, .. }) = ty {
        let ty = *elem.clone();
        if let Type::Slice(_) = ty {
            return true;
        }
    }
    false
}

fn is_option(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        let seg = path.segments.iter().next().unwrap();
        if seg.ident != "Option" {
            unimplemented!()
        }
        return true;
    }
    false
}
