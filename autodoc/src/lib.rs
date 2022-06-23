use std::{collections::HashMap, fmt::Write, time::Instant};

use clap::{crate_name, Arg, ArgMatches, Command};
use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated, token::Comma, Field, Item, ItemEnum, Token, Type, TypePath,
    TypeReference, Variant,
};
use utils::{
    boolean_to_emoji, find_struct, get_constraints_and_doc, get_struct_fields, strip_docs,
};

use crate::utils::generate_table;

pub mod utils;

const LITS: &[&str] = &["N", "M", "P", "Q", "R"];

pub fn command() -> Command<'static> {
    Command::new(crate_name!())
        .version("0.1")
        .author("Bonfida")
        .about("Autogenerate Rust documentation for instructions")
        .arg(
            Arg::with_name("instructions_path")
                .takes_value(true)
                .default_value("src/processor"),
        )
        .arg(
            Arg::with_name("instructions_enum_path")
                .takes_value(true)
                .default_value("src/instruction.rs"),
        )
}

pub fn process(matches: &ArgMatches) {
    let instructions_path = matches.value_of("instructions_path").unwrap();
    let instructions_enum_path = matches.value_of("instructions_enum_path").unwrap();
    generate(
        instructions_path,
        instructions_enum_path,
        "src/instruction_auto.rs",
    );
}

pub fn generate(instructions_path: &str, instructions_enum_path: &str, output_path: &str) {
    let now = Instant::now();
    let path = std::path::Path::new(instructions_path);
    let directory = std::fs::read_dir(path).unwrap();
    let accounts_table_columns = [
        "Index".to_owned(),
        "Writable".to_owned(),
        "Signer".to_owned(),
        "Description".to_owned(),
    ];
    let mut instruction_docs = HashMap::new();
    for d in directory {
        let file = d.unwrap();
        let module_name = std::path::Path::new(&file.file_name())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let (module_doc, accounts_descriptors) = parse_instruction(file.path().to_str().unwrap());
        let table = generate_table(&accounts_table_columns, &accounts_descriptors);
        let instruction_documentation =
            [&module_doc as &[String], &["".to_owned()], &table].concat();
        instruction_docs.insert(snake_to_pascal(&module_name), instruction_documentation);
    }

    process_instructions(instructions_enum_path, output_path, &instruction_docs);

    // let mut out_file = File::create(output_path).unwrap();
    // out_file.write_all(output.as_bytes()).unwrap();

    let elapsed = now.elapsed();
    println!("✨  Done in {:.2?}", elapsed);
}

fn process_instructions(
    instructions_path: &str,
    out_path: &str,
    instruction_docs: &HashMap<String, Vec<String>>,
) {
    let path = std::path::Path::new(instructions_path);
    let raw_file = std::fs::read_to_string(path).unwrap();
    let mut file_ast: syn::File = syn::parse_str(&raw_file).unwrap();
    let instructions_enum = find_enum(&mut file_ast);
    let enum_variants = get_enum_variants(instructions_enum);

    for Variant {
        attrs,
        ident,
        fields: _,
        discriminant: _,
    } in enum_variants
    {
        let instruction_doc = instruction_docs.get(&ident.to_string()).unwrap();
        *attrs = strip_docs(attrs);
        for d in instruction_doc {
            attrs.push(syn::Attribute {
                pound_token: <Token![#]>::default(),
                style: syn::AttrStyle::Outer,
                bracket_token: syn::token::Bracket {
                    span: Span::call_site(),
                },
                path: syn::Path::from(syn::PathSegment::from(syn::Ident::new(
                    "doc",
                    Span::call_site(),
                ))),
                tokens: quote!(= #d),
            });
        }
    }
    let mut t = file_ast.to_token_stream().to_string();
    // std::fs::write(out_path, t).unwrap();
    t = rustfmt_wrapper::rustfmt(&t).unwrap();
    let mut processed = "".to_owned();
    for l in t.lines() {
        if l.matches("#[doc = ").next().is_some() {
            let mut o = l.replace("#[doc = \"", "/// ");
            o.truncate(o.len() - 2);
            processed.write_str(&o).unwrap();
        } else {
            processed.write_str(l).unwrap()
        }
        processed.write_char('\n').unwrap();
    }
    std::fs::write(out_path, processed).unwrap();
}

fn parse_instruction(instruction_path: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let path = std::path::Path::new(instruction_path);
    let raw_file = std::fs::read_to_string(path).unwrap();
    let file_ast: syn::File = syn::parse_str(&raw_file).unwrap();
    let (_, _, file_doc) = get_constraints_and_doc(&file_ast.attrs);
    let accounts_struct = find_struct("Accounts", &file_ast);
    let accounts_fields = get_struct_fields(accounts_struct);
    let mut accounts_descriptors = Vec::with_capacity(accounts_fields.len());
    let mut current_lit_offset = 0;
    for (
        f_idx,
        Field {
            attrs,
            vis: _,
            ident: _,
            colon_token: _,
            ty,
        },
    ) in accounts_fields.iter().enumerate()
    {
        let (writable, signer, doc) = get_constraints_and_doc(attrs);
        let mut index = if current_lit_offset == 0 {
            f_idx.to_string()
        } else {
            format!("{} + {}", f_idx, LITS[..current_lit_offset].join(" + "))
        };
        if is_slice(ty) {
            current_lit_offset += 1;
            index
                .write_str(&format!(
                    "..{} + {}",
                    f_idx,
                    LITS[..current_lit_offset].join(" + "),
                ))
                .unwrap();
        }
        accounts_descriptors.push(vec![
            index,
            boolean_to_emoji(writable).to_string(),
            boolean_to_emoji(signer).to_string(),
            doc.into_iter().next().unwrap_or_else(|| "".to_owned()), // TODO: multi-line comments?
        ]);
    }
    (file_doc, accounts_descriptors)
}

fn snake_to_pascal(s: &str) -> String {
    s.from_case(Case::Snake).to_case(Case::Pascal)
}

fn find_enum(file_ast: &mut syn::File) -> &mut Item {
    file_ast
        .items
        .iter_mut()
        .find(|a| matches!(a, Item::Enum(_)))
        .unwrap()
}

fn get_enum_variants(s: &mut Item) -> &mut Punctuated<Variant, Comma> {
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
