use std::fmt::Write;

use proc_macro2::{TokenStream, TokenTree};
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Field, Fields, FieldsNamed, Item, ItemStruct,
};

pub type TableRow = Vec<String>;

pub fn generate_table(columns: &[String], data: &[TableRow]) -> Vec<String> {
    let mut lines = Vec::with_capacity(data.len() + 2);
    let mut column_widths = Vec::with_capacity(columns.len());
    let mut current_line = "".to_owned();
    for (c_idx, c) in columns.iter().enumerate() {
        let mut current_width = c.len();
        for d in data {
            current_width = std::cmp::max(current_width, d[c_idx].len());
        }
        column_widths.push(current_width);
        current_line
            .write_str(&format!(
                "| {value:width$} ",
                value = c,
                width = current_width
            ))
            .unwrap();
    }
    current_line.write_str("|").unwrap();
    let total_width = current_line.len();
    lines.push(current_line);

    lines.push(format!(
        "| {val:-<width$} |",
        val = "",
        width = total_width - 4
    ));
    for t in data {
        current_line = "".to_owned();
        for (value, column_width) in t.iter().zip(column_widths.iter()) {
            current_line
                .write_str(&format!("| {:1$} ", value, column_width))
                .unwrap();
        }
        current_line.write_str("|").unwrap();
        lines.push(current_line)
    }
    lines
}

pub fn find_struct(ident_str: &str, file_ast: &syn::File) -> Item {
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

pub fn get_struct_fields(s: Item) -> Punctuated<Field, Comma> {
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

pub fn get_constraints_and_doc(attrs: &[Attribute]) -> (bool, bool, Vec<String>) {
    let mut writable = false;
    let mut signer = false;
    let mut doc = vec![];
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
        } else if a.path.is_ident("doc") {
            let _t = if let TokenTree::Literal(l) = a.tokens.clone().into_iter().nth(1).unwrap() {
                l
            } else {
                panic!()
            };
            let parsed_l: syn::LitStr = syn::parse2(TokenStream::from(
                a.tokens.clone().into_iter().nth(1).unwrap(),
            ))
            .unwrap();
            doc.push(parsed_l.value().trim().to_owned());
        }
    }
    (writable, signer, doc)
}

pub fn strip_docs(attrs: &[Attribute]) -> Vec<Attribute> {
    let mut attributes = Vec::with_capacity(attrs.len());
    for a in attrs {
        if !a.path.is_ident("doc") {
            attributes.push(a.clone());
        }
    }
    attributes
}

pub fn boolean_to_emoji(b: bool) -> char {
    match b {
        true => '✅',
        false => '❌',
    }
}
