use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
};

use syn::{
    AngleBracketedGenericArguments, Expr, ExprLit, Field, FnArg, GenericArgument, ImplItem,
    ImplItemConst, ImplItemMethod, Item, ItemConst, ItemImpl, Lit, LitByteStr, PatType, Path,
    PathArguments, Type, TypeArray, TypePath,
};

use once_cell::sync::Lazy;
use std::sync::Mutex;

static VISITED_STATES: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

use crate::{
    capitalize_first_letter, find_struct, get_constraints, get_struct_fields, is_newtype_struct,
    is_option, is_slice, padding_len, pascal_to_snake, snake_to_camel,
};

use walkdir::WalkDir;

const DEFAULT_IMPORT: &str =
    "import { PublicKey, TransactionInstruction } from \"@solana/web3.js\";";

pub const ACCOUNT_KEY_INTERFACE: &str = "export interface AccountKey {
  pubkey: PublicKey;
  isSigner: boolean; 
  isWritable: boolean;
}";

const BYTEMUCK_IMPORT: &str = "import { BytemuckSerializer } from \"./bytemuck\";";

const BUFFER_IMPORT: &str = "import { Buffer } from \"buffer\";";

pub const IMPORTS: [&str; 3] = [DEFAULT_IMPORT, BYTEMUCK_IMPORT, BUFFER_IMPORT];

pub struct JSOutput {
    pub imports: HashSet<String>,
    pub content: String,
    pub custom_types: HashSet<String>,
    pub optional_types: HashSet<OptionalType>,
}

pub fn js_process_file(
    module_name: &str,
    instruction_tag: usize,
    path: &str,
    use_casting: bool,
) -> JSOutput {
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

    let mut custom_types = HashSet::new();
    let mut optional_types: HashSet<OptionalType> = HashSet::new();

    fn collect_types(
        ty: &Type,
        custom_types: &mut HashSet<String>,
        optional_types: &mut HashSet<OptionalType>,
    ) {
        match ty {
            Type::Path(tp) => {
                if let Some(seg) = tp.path.segments.first() {
                    let type_name = seg.ident.to_string();
                    if type_name == "OptionalData" {
                        if let PathArguments::AngleBracketed(args) = &seg.arguments {
                            if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                                let optional_js_name = match inner_type {
                                    Type::Path(inner) => {
                                        let rust_type =
                                            inner.path.segments.first().unwrap().ident.to_string();
                                        format!("Optional{}", capitalize_first_letter(&rust_type))
                                    }
                                    // If the inner type is a simple path, extract its name.
                                    Type::Array(_) => {
                                        let js_type = type_to_js(inner_type);
                                        format!(
                                            "Optional{}",
                                            capitalize_first_letter(&js_type.js_type)
                                        )
                                    }
                                    _ => unimplemented!(
                                        "Unsupported inner type for OptionalData: {:?}",
                                        inner_type
                                    ), // TODO
                                };
                                optional_types.insert(OptionalType {
                                    file_name: optional_js_name,
                                    inner_type: inner_type.clone(),
                                });
                            }
                        }
                    } else if !matches!(
                        type_name.as_str(),
                        "bool"
                            | "u8"
                            | "u16"
                            | "u32"
                            | "u64"
                            | "u128"
                            | "i8"
                            | "i16"
                            | "i32"
                            | "i64"
                            | "i128"
                            | "String"
                            | "Pubkey"
                            | "Vec"
                            | "Option"
                            | "str"
                    ) {
                        custom_types.insert(type_name);
                    }
                    if let PathArguments::AngleBracketed(args) = &seg.arguments {
                        for arg in &args.args {
                            if let GenericArgument::Type(inner_ty) = arg {
                                collect_types(inner_ty, custom_types, optional_types);
                            }
                        }
                    }
                }
            }
            Type::Reference(tr) => collect_types(&tr.elem, custom_types, optional_types),
            Type::Array(ta) => collect_types(&ta.elem, custom_types, optional_types),
            Type::Slice(ts) => collect_types(&ts.elem, custom_types, optional_types),
            _ => {}
        }
    }

    for Field { ty, .. } in &params_fields {
        collect_types(ty, &mut custom_types, &mut optional_types);
    }

    // Add imports at the start
    let mut imports = HashSet::new();

    for ct in custom_types.clone() {
        let snake_name = pascal_to_snake(&ct);
        imports.insert(format!(
            "import {{ {} }} from \"./raw_state/{}\";",
            ct, snake_name
        ));
    }

    for Field {
        attrs: _,
        vis: _,
        ident,
        colon_token: _,
        ty,
    } in params_fields
    {
        let camel_case_ident = snake_to_camel(&ident.as_ref().unwrap().to_string());
        let SchemaType {
            import: _, // Imports are already imported above
            schema_type,
            create_optional_type: _, // TODO?
        } = type_to_schema_js(&ty);
        schema_statements.push(format!("{}: {},", camel_case_ident, schema_type));
        if camel_case_ident == "padding" {
            declaration_statements.push("padding: Uint8Array;".to_owned());
            assign_statements.push(format!(
                "this.padding = (new Uint8Array({})).fill(0)",
                padding_len(&ty)
            ));
        } else {
            declaration_statements.push(format!(
                "{}: {};",
                camel_case_ident,
                type_to_js(&ty).js_type
            ));
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
    statements.extend(schema_statements);
    statements.push("} as const;".to_owned());
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

    statements.push("serialize(): Buffer {".to_owned());

    statements.push(format!(
        "return BytemuckSerializer.encode({}Instruction.schema, this);",
        snake_to_camel(module_name)
    ));
    statements.push("}".to_owned());
    statements.push("getInstruction(".to_owned());
    statements.extend(accounts_statements);
    statements.push("): TransactionInstruction {".to_owned());
    statements.push("const data = this.serialize();".to_owned());
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

    // Append all the previously generated statement lines.
    for stmt in statements {
        out_string.push_str(&stmt);
        out_string.push('\n');
    }

    JSOutput {
        imports,
        content: out_string,
        custom_types,
        optional_types,
    }
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
        Type::Slice(_) => format!("this.{} = obj.{};", camel_case_ident, camel_case_ident),
        Type::Reference(tr) => js_type_assignment(&tr.elem, camel_case_ident),
        x => unimplemented!("{x:?}"),
    }
}

#[derive(Debug)]
pub struct JSType {
    pub import: Option<Import>,
    pub js_type: String,
}

#[derive(Debug)]
pub enum Import {
    Solana,
    Project,
}

fn type_to_js(ty: &Type) -> JSType {
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
            if simple_type == "OptionalData" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        let inner_js_type = type_to_js(inner_ty);
                        let js_type = format!(
                            "Optional{}",
                            capitalize_first_letter(&inner_js_type.js_type)
                        );
                        return JSType {
                            js_type,
                            import: Some(Import::Project),
                        };
                    }
                }
                unimplemented!("OptionalData type must have an inner type {segment:?}");
            }
            match simple_type.as_ref() {
                "bool" => JSType {
                    js_type: "boolean".to_owned(),
                    import: None,
                },
                "u8" | "u16" | "u32" | "i8" | "i16" | "i32" => JSType {
                    js_type: "number".to_owned(),
                    import: None,
                },
                "u64" | "u128" | "i64" | "i128" => JSType {
                    js_type: "bigint".to_owned(),
                    import: None,
                },
                "String" | "str" => JSType {
                    js_type: "string".to_owned(),
                    import: None,
                },
                "Pubkey" => JSType {
                    js_type: "PublicKey".to_owned(),
                    import: Some(Import::Solana),
                },
                "Vec" => {
                    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: _,
                        lt_token: _,
                        args,
                        gt_token: _,
                    }) = &segment.arguments
                    {
                        if let GenericArgument::Type(t) = args.first().unwrap() {
                            let JSType {
                                import,
                                js_type: inner_type,
                            } = type_to_js(t);
                            JSType {
                                import,
                                js_type: format!("{}[]", &inner_type),
                            }
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    }
                }

                _ => JSType {
                    import: Some(Import::Project),
                    js_type: simple_type,
                },
            }
        }
        Type::Array(TypeArray {
            bracket_token: _,
            elem,
            semi_token: _,
            len: _,
        }) => {
            let inner_type = type_to_schema_js(elem);
            JSType {
                import: inner_type.import,
                js_type: array_to_js(&inner_type.schema_type),
            }
        }
        Type::Reference(tr) => type_to_js(&tr.elem),
        Type::Slice(ts) => {
            // Handle slices as arrays
            let JSType {
                import,
                js_type: inner_type,
            } = type_to_js(&ts.elem);
            JSType {
                import,
                js_type: format!("{}[]", inner_type),
            }
        }
        x => unimplemented!("{x:?}"),
    }
}

fn array_to_js(inner_type: &str) -> String {
    match inner_type as &str {
        "\"u8\"" | "\"i8\"" => "Uint8Array",
        "\"u16\"" | "\"i16\"" | "\"u32\"" | "\"i32\"" => "number[]",
        "\"u64\"" => "BigUint64Array",
        "\"i64\"" | "\"u128\"" | "\"i128\"" => "bigint[]", // TODO
        _ => unimplemented!(),
    }
    .to_owned()
}

#[derive(Debug)]
pub struct SchemaType {
    pub import: Option<Import>,
    pub schema_type: String,
    pub create_optional_type: Option<OptionalType>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct OptionalType {
    pub file_name: String,
    pub inner_type: Type,
}

fn type_to_schema_js(ty: &Type) -> SchemaType {
    match ty {
        Type::Reference(tr) => type_to_schema_js(&tr.elem),
        Type::Slice(ts) => {
            // Handle slices as arrays
            let SchemaType {
                schema_type: inner_type,
                import,
                create_optional_type: _,
            } = type_to_schema_js(&ts.elem);
            SchemaType {
                import,
                schema_type: format!("{{ array: {{ type: {} }} }}", inner_type),
                create_optional_type: None,
            }
        }
        Type::Path(TypePath {
            qself: _,
            path: Path {
                leading_colon: _,
                segments,
            },
        }) => {
            let segment = segments.iter().next().unwrap();
            let simple_type = segment.ident.to_string();
            if simple_type == "OptionalData" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                        let optional_js_name = match inner_type {
                            // If the inner type is a simple path, extract its name.
                            Type::Path(_) | Type::Array(_) => {
                                let js_type = type_to_js(inner_type);
                                format!("Optional{}", capitalize_first_letter(&js_type.js_type))
                            }
                            _ => unimplemented!(
                                "Unsupported inner type for OptionalData: {:?}",
                                inner_type
                            ), // TODO
                        };
                        return SchemaType {
                            import: None,
                            schema_type: format!("{}.schema", optional_js_name),
                            create_optional_type: Some(OptionalType {
                                file_name: optional_js_name,
                                inner_type: inner_type.clone(),
                            }),
                        };
                    }
                }
                unimplemented!("OptionalData must have an inner type {segment:?}");
            }
            let t = match simple_type.as_ref() {
                "u8" | "u16" | "u32" | "u64" | "u128" | "bool" => simple_type,
                "i8" | "i16" | "i32" | "i64" | "i128" => {
                    let mut res = "u".to_owned();
                    <String as std::fmt::Write>::write_str(&mut res, &simple_type[1..]).unwrap();
                    res
                }
                "String" => "string".to_owned(),
                "Pubkey" => {
                    return SchemaType {
                        import: Some(Import::Solana),
                        schema_type: "\"pubkey\"".to_owned(),
                        create_optional_type: None,
                    }
                }
                "Vec" => {
                    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: _,
                        lt_token: _,
                        args,
                        gt_token: _,
                    }) = &segment.arguments
                    {
                        if let GenericArgument::Type(t) = args.first().unwrap() {
                            let inner_type = type_to_schema_js(t);
                            return SchemaType {
                                import: inner_type.import,
                                schema_type: format!(
                                    "{{ array: {{ type: {} }} }}",
                                    &inner_type.schema_type
                                ),
                                create_optional_type: None,
                            };
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                // TODO delete Option?
                "Option" => {
                    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: _,
                        lt_token: _,
                        args,
                        gt_token: _,
                    }) = &segment.arguments
                    {
                        if let GenericArgument::Type(t) = args.first().unwrap() {
                            let inner_type = type_to_schema_js(t);
                            return SchemaType {
                                import: inner_type.import,
                                schema_type: format!("{{ option: {} }}", &inner_type.schema_type),
                                create_optional_type: None,
                            }; // TODO
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                "str" => {
                    return SchemaType {
                        schema_type: "\"string\"".to_owned(),
                        import: None,
                        create_optional_type: None,
                    }
                }
                _ => {
                    return SchemaType {
                        import: Some(Import::Project),
                        schema_type: format!("{}.schema", simple_type),
                        create_optional_type: None,
                    }
                } // Custom type schema
            };
            SchemaType {
                schema_type: format!("\"{}\"", t),
                import: None,
                create_optional_type: None,
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
            let inner_type = type_to_schema_js(elem);
            let mut unsigned_type = "u".to_owned();
            <String as std::fmt::Write>::write_str(
                &mut unsigned_type,
                &inner_type.schema_type[2..inner_type.schema_type.len() - 1],
            )
            .unwrap();

            match &unsigned_type as &str {
                "u16" | "u32" | "u64" | "u128" | "u8" => {
                    let schema_type = format!(
                        "{{ array: {{ type: {}, len: {} }} }}",
                        inner_type.schema_type,
                        l.base10_parse::<u8>().unwrap()
                    );
                    SchemaType {
                        schema_type,
                        import: None,
                        create_optional_type: None,
                    }
                }
                _ => {
                    unimplemented!("{inner_type:?}")
                }
            }
        }
        x => unimplemented!("{x:?}"),
    }
}

fn js_process_newtype_struct(name: &str, underlying_type: &Type) -> String {
    let ts_type = type_to_js(underlying_type);
    let schema_type = type_to_schema_js(underlying_type).schema_type;
    // If the schema_type is a quoted string (e.g. "\"u8\""), remove the surrounding quotes.
    let schema_value = if schema_type.starts_with('"') && schema_type.ends_with('"') {
        &schema_type[1..schema_type.len() - 1]
    } else {
        &schema_type
    };

    format!(
        "\nexport class {} {{
    private _value: {};

    static schema = \"{}\" as const;
    
    constructor(initialValue: {}) {{
        this._value = initialValue;
    }}

    public get value(): {} {{
        return this._value;
    }}

    
}}",
        name, ts_type.js_type, schema_value, ts_type.js_type, ts_type.js_type
    )
}

/// Recursively searches the given root for Rust files and indexes struct definitions by name.
fn build_struct_index(root: &str) -> HashMap<String, (String, syn::File)> {
    let mut index = HashMap::new();

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
            let file_path = entry.path().display().to_string();
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if content.trim().is_empty() {
                continue;
            }
            let ast = match syn::parse_file(&content) {
                Ok(file) => file,
                Err(_) => continue,
            };
            // Add all struct definitions to the index.
            for item in &ast.items {
                if let syn::Item::Struct(s) = item {
                    index.insert(s.ident.to_string(), (file_path.clone(), ast.clone()));
                }
            }
        }
    }
    index
}

pub fn generate_optional_schema(custom_type: &OptionalType) -> JSOutput {
    let ts_inner_type = type_to_js(&custom_type.inner_type);
    let schema_inner = type_to_schema_js(&custom_type.inner_type);
    let mut statements = Vec::new();
    // Optional class declaration
    statements.push(format!("\nexport class {} {{", custom_type.file_name));
    // In our design isSome is always a u64 i.e bigint (using 1n or 0n)
    statements.push("  isSome: bigint;".to_owned());
    statements.push(format!("  value: {};", ts_inner_type.js_type));
    statements.push("".to_owned());
    statements.push("  static schema = {".to_owned());

    statements.push("    isSome: \"u64\",".to_owned());
    statements.push(format!("    value: {},", schema_inner.schema_type));
    statements.push("  } as const;".to_owned());
    statements.push("".to_owned());
    statements.push(format!(
        "  constructor(obj: {{ isSome: bigint; value: {} }}) {{",
        ts_inner_type.js_type
    ));
    statements.push("    (this.isSome = obj.isSome), (this.value = obj.value);".to_owned());
    statements.push("  }".to_owned());
    statements.push("".to_owned());
    statements.push("  exists() {".to_owned());
    statements.push("    return this.isSome === 1n;".to_owned());
    statements.push("  }".to_owned());
    statements.push("".to_owned());
    statements.push("  static decode(data: Buffer) {".to_owned());
    statements.push(
        "    const decoded = BytemuckSerializer.decode<InferField<typeof this.schema>>(".to_owned(),
    );
    statements.push("      this.schema,".to_owned());
    statements.push("      data".to_owned());
    statements.push("    );".to_owned());
    statements.push(format!(
        "    return new {}(decoded);",
        custom_type.file_name
    ));
    statements.push("  }".to_owned());
    statements.push("}".to_owned());

    // Insert necessary imports
    let mut imports = HashSet::new();
    imports
        .insert("import { BytemuckSerializer, type InferField } from \"../bytemuck\";".to_owned());
    imports.insert("import { Buffer } from \"buffer\";".to_owned());

    let js_content = statements.join("\n");
    JSOutput {
        imports,
        content: js_content,
        custom_types: HashSet::default(),
        optional_types: HashSet::default(),
    }
}

pub fn js_generate_state_files(
    project_root: &str,
    custom_types: &HashSet<String>,
    optional_types: &HashSet<OptionalType>,
) -> HashMap<String, JSOutput> {
    // Build an index from struct names to (source file path, parsed AST)
    let struct_index = build_struct_index(project_root);

    let mut state_outputs = HashMap::new();

    for optional_ty in optional_types {
        state_outputs.insert(
            optional_ty.file_name.clone(),
            generate_optional_schema(optional_ty),
        );
    }

    for custom_type in custom_types {
        if let Some((_, ast)) = struct_index.get(custom_type) {
            // Find the struct in the AST.
            let struct_item = find_struct(ast, Some(custom_type));
            let fields = get_struct_fields(struct_item);

            let mut class_imports = HashSet::new();
            let mut statements = Vec::new();
            statements.push(format!("\nexport class {} {{", custom_type));

            for Field { ty, ident, .. } in fields.iter() {
                let field_name = snake_to_camel(&ident.as_ref().unwrap().to_string());
                let type_info = type_to_js(ty);

                if let Some(imp) = type_info.import {
                    match imp {
                        Import::Solana => {
                            class_imports.insert(
                                "import { PublicKey } from \"@solana/web3.js\";".to_owned(),
                            );
                        }
                        Import::Project => {
                            let snake_name = pascal_to_snake(&type_info.js_type);
                            class_imports.insert(format!(
                                "import {{ {} }} from \"./{}\";",
                                type_info.js_type, snake_name
                            ));
                        }
                    }
                }
                statements.push(format!("  {}: {};", field_name, type_info.js_type));
            }

            // Generate the static 'schema'
            statements.push("\n  static schema = {".to_owned());
            for Field { ty, ident, .. } in fields.iter() {
                let field_name = snake_to_camel(&ident.as_ref().unwrap().to_string());
                let SchemaType {
                    schema_type,
                    import,
                    create_optional_type,
                } = type_to_schema_js(ty);

                if let Some(optional_type) = create_optional_type {
                    state_outputs.insert(
                        optional_type.file_name.clone(),
                        generate_optional_schema(&optional_type),
                    );
                }
                // TODO use import?
                {
                    // If no import is needed and the schema string is not a primitive (i.e. not wrapped in quotes),
                    // this indicates that nested state might be missing and must be processed recursively.
                    if import.is_some() && !schema_type.starts_with("\"") {
                        let mut visited = VISITED_STATES.lock().unwrap();
                        if !visited.contains(&schema_type) {
                            visited.insert(schema_type.clone());
                            generate_nested_state(ty);
                            eprintln!(
                                "Recursively processed missing state for type: {}",
                                schema_type
                            );
                        }
                    }
                }
                statements.push(format!("    {}: {},", field_name, schema_type));
            }
            statements.push("  } as const;\n".to_owned());

            // Generate a constructor that takes an object literal.
            statements.push("  constructor(obj: {".to_owned());
            for Field { ty, ident, .. } in fields.iter() {
                let field_name = snake_to_camel(&ident.as_ref().unwrap().to_string());
                let type_info = type_to_js(ty);
                if let Some(Import::Project) = type_info.import {
                    statements.push(format!(
                        "    {}: InferField<typeof {}.schema>,",
                        field_name, type_info.js_type
                    ));
                } else {
                    statements.push(format!("    {}: {},", field_name, type_info.js_type));
                }
            }
            statements.push("  }) {".to_owned());
            for Field { ident, ty, .. } in fields.iter() {
                let field_name = snake_to_camel(&ident.as_ref().unwrap().to_string());
                let type_info = type_to_js(ty);

                if let Some(Import::Project) = type_info.import {
                    statements.push(format!(
                        "    this.{} = new {}(obj.{});",
                        field_name, type_info.js_type, field_name
                    ));
                } else {
                    statements.push(format!("    this.{} = obj.{};", field_name, field_name));
                }
            }
            statements.push("  }\n".to_owned());

            // Encode methods
            statements.push("   encode() {".to_owned());
            statements.push(format!(
                "       return BytemuckSerializer.encode({custom_type}.schema, this)"
            ));
            statements.push("   }\n".to_owned());

            // Decode
            statements.push("   static decode(data: Buffer) {".to_owned());
            statements.push(
                               "       const decoded = BytemuckSerializer.decode<InferField<typeof this.schema>>(this.schema, data);"
                               .to_owned(),
                           );
            statements.push(format!("       return new {custom_type}(decoded);"));
            statements.push("   }\n".to_owned());
            class_imports.insert(
                "import { BytemuckSerializer, type InferField } from \"../bytemuck\";".to_owned(),
            );
            class_imports.insert("import { Buffer } from \"buffer\";".to_owned());

            // Search for SEED and get_key/find_key
            for item in ast.items.iter() {
                if let Item::Impl(ItemImpl { self_ty, items, .. }) = item {
                    // Make sure it's an impl block for our struct
                    match *self_ty.to_owned() {
                        Type::Path(TypePath {
                            qself: _,
                            path:
                                Path {
                                    leading_colon: _,
                                    segments,
                                },
                        }) => {
                            let segment = segments.iter().next().unwrap();
                            let ty_str = segment.ident.to_string();
                            if ty_str != *custom_type {
                                continue;
                            }

                            for impl_item in items {
                                match impl_item {
                                    ImplItem::Const(ImplItemConst { ident, expr, .. }) => {
                                        if *ident == "SEED" {
                                            if let Expr::Lit(ExprLit {
                                                attrs: _,
                                                lit: Lit::ByteStr(repr),
                                            }) = expr
                                            {
                                                let seed = repr.token().to_string();
                                                let seed = seed.strip_prefix("b").unwrap();
                                                statements.push(format!(
                                                    "   static SEED = Buffer.from({})",
                                                    seed
                                                ));
                                            }
                                        }
                                    }

                                    // The exact implementation of the PDA derivation is left to the user to handle complex cases
                                    ImplItem::Method(ImplItemMethod { sig, block: _, .. }) => {
                                        let fn_name = sig.ident.to_string();
                                        if fn_name == "get_key" || fn_name == "find_key" {
                                            struct FnParam {
                                                pub name: String,
                                                pub js_type: String,
                                            }
                                            let mut params: Vec<FnParam> = vec![];

                                            for input in &sig.inputs {
                                                if let FnArg::Typed(PatType { pat, ty, .. }) = input
                                                {
                                                    let param_name = match *pat.clone() {
                                                        syn::Pat::Ident(pat_ident) => {
                                                            pat_ident.ident.to_string()
                                                        }
                                                        _ => unimplemented!("{pat:?}"),
                                                    };
                                                    let param_type = type_to_js(&ty.clone());
                                                    params.push(FnParam {
                                                        name: param_name.clone(),
                                                        js_type: param_type.js_type.clone(),
                                                    });

                                                    match param_type.import {
                                                        Some(Import::Solana) => {
                                                            class_imports.insert( "import { PublicKey } from \"@solana/web3.js\";".to_owned());
                                                        }
                                                        _ => unimplemented!(
                                                            "{param_name} {param_type:?}"
                                                        ),
                                                    }
                                                }
                                            }
                                            let mut fn_statement = String::new();
                                            if !params.is_empty() {
                                                fn_statement
                                                    .push_str(&format!("    static {fn_name}("));
                                                for param in params {
                                                    fn_statement.push_str(&format!(
                                                        "{}: {}, ",
                                                        param.name, param.js_type
                                                    ));
                                                }
                                                fn_statement.push_str("programId?: PublicKey");
                                                fn_statement.push(')');
                                                fn_statement.push_str("{ \n         // TODO \n }");
                                            }
                                            statements.push(fn_statement)
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }

                        _ => todo!("{self_ty:?}"),
                    }
                }
            }

            // Close class
            statements.push("}".to_owned());

            // Join all parts into one JS content string.
            let js_content = statements.join("\n");
            let output = JSOutput {
                imports: class_imports,
                content: js_content,
                custom_types: HashSet::default(),
                optional_types: HashSet::default(),
            };

            state_outputs.insert(custom_type.clone(), output);
        } else {
            // If a struct isn't found, write an error.
            eprint!("Struct {} not found in your project", custom_type);
        }
    }

    state_outputs
}

fn generate_nested_state(ty: &Type) -> Option<JSOutput> {
    // If the type is wrapped (e.g. in a reference, slice, or array),
    // then simply process its inner element.

    match ty {
        Type::Reference(tr) => return generate_nested_state(&tr.elem),
        Type::Slice(ts) => return generate_nested_state(&ts.elem),
        Type::Array(ta) => return generate_nested_state(&ta.elem),
        _ => (),
    }

    // If it's a simple path type, extract the first segment.
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.first() {
            let type_name = seg.ident.to_string();

            // For primitives and standard types, avoid generating nested state.
            if matches!(
                type_name.as_str(),
                "u8" | "u16"
                    | "u32"
                    | "u64"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "bool"
                    | "String"
                    | "str"
            ) {
                return None;
            }

            // For Option and Vec, get the inner type.
            if type_name == "Option" || type_name == "Vec" {
                if let PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return generate_nested_state(inner_ty);
                    }
                }
                return None;
            }

            // For OptionalData wrapper, get its inner type too.
            if type_name == "OptionalData" {
                if let PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return generate_nested_state(inner_ty);
                    }
                }
                return None;
            }

            // At this point we assume this is a custom type that needs generating.
            // Use our build_struct_index to locate the source.
            let project_root = ".";
            let index = build_struct_index(project_root);
            if let Some((file_path, ast)) = index.get(&type_name) {
                let struct_item = find_struct(ast, Some(&type_name));

                if let Some(underlying_type) = is_newtype_struct(&struct_item) {
                    let alias = js_process_newtype_struct(&type_name, underlying_type);
                    std::fs::write(
                        format!("../js/src/raw_state/{}.ts", pascal_to_snake(&type_name)),
                        &alias,
                    )
                    .unwrap();
                    return Some(JSOutput {
                        imports: HashSet::new(),
                        content: alias,
                        custom_types: HashSet::new(),
                        optional_types: HashSet::default(),
                    });
                }
                let fields = get_struct_fields(struct_item);

                let mut class_imports = std::collections::HashSet::new();
                let mut statements = Vec::new();
                statements.push(format!("export class {} {{", type_name));

                // Generate class property declarations from struct fields
                for Field { ty, ident, .. } in fields.iter() {
                    let field_name = snake_to_camel(&ident.as_ref().unwrap().to_string());
                    let type_info = type_to_js(ty);
                    // Record any needed imports based on the field type.
                    if let Some(imp) = type_info.import {
                        match imp {
                            Import::Solana => {
                                class_imports.insert(
                                    "import { PublicKey } from \"@solana/web3.js\";".to_owned(),
                                );
                            }
                            Import::Project => {
                                let snake_name = pascal_to_snake(&type_info.js_type);
                                class_imports.insert(format!(
                                    "import {{ {} }} from \"./{}\";",
                                    type_info.js_type, snake_name
                                ));
                            }
                        }
                    }
                    statements.push(format!("  {}: {};", field_name, type_info.js_type));
                }

                // Generate the static schema property using type_to_schema_js for each field.
                statements.push("  static schema = {".to_owned());
                for Field { ty, ident, .. } in fields.iter() {
                    let field_name = snake_to_camel(&ident.as_ref().unwrap().to_string());
                    let SchemaType {
                        schema_type,
                        import: _,
                        create_optional_type: _, // TODO
                    } = type_to_schema_js(ty);
                    statements.push(format!("    {}: {},", field_name, schema_type));
                }
                statements.push("  } as const;".to_owned());

                // Generate the constructor that assigns object properties.
                statements.push("  constructor(obj: {".to_owned());
                for Field { ty, ident, .. } in fields.iter() {
                    let field_name = snake_to_camel(&ident.as_ref().unwrap().to_string());
                    let type_info = type_to_js(ty);
                    statements.push(format!("    {}: {},", field_name, type_info.js_type));
                }
                statements.push("  }) {".to_owned());
                for Field { ident, .. } in fields.iter() {
                    let field_name = snake_to_camel(&ident.as_ref().unwrap().to_string());
                    statements.push(format!("    this.{} = obj.{};", field_name, field_name));
                }
                statements.push("  }".to_owned());

                // Encode methods
                statements.push("   encode() {".to_owned());
                statements.push(format!(
                    "       return BytemuckSerializer.encode({type_name}.schema, this)"
                ));
                statements.push("   }".to_owned());

                // Decode
                statements.push("   static decode(data: Buffer) {".to_owned());
                statements.push(
                    "       const decoded = BytemuckSerializer.decode<InferField<typeof this.schema>>(this.schema, data);"
                    .to_owned(),
                );
                statements.push(format!("       return new {type_name}(decoded);"));
                statements.push("   }".to_owned());
                class_imports.insert(
                    "import { BytemuckSerializer, type InferField } from \"../bytemuck\";"
                        .to_owned(),
                );
                class_imports.insert("import { Buffer } from \"buffer\";".to_owned());

                // Close class
                statements.push("}".to_owned());

                let imports_str = class_imports.iter().cloned().collect::<Vec<_>>().join("\n");
                let content = statements.join("\n");
                let full_content = format!("{}\n\n{}", imports_str, content);

                let js_output = JSOutput {
                    imports: class_imports,
                    content: content.clone(),
                    custom_types: std::collections::HashSet::new(),
                    optional_types: HashSet::default(),
                };

                // Write the generated file to disk:
                std::fs::write(
                    format!("../js/src/raw_state/{}.ts", pascal_to_snake(&type_name)),
                    &full_content,
                )
                .unwrap();

                // Recursively check each field to generate any further missing nested state.
                for Field { ty, .. } in fields.iter() {
                    let _ = generate_nested_state(ty);
                }

                eprintln!(
                    "Recursively processed missing state for type: {} (from {})",
                    type_name, file_path
                );
                return Some(js_output);
            } else {
                eprintln!(
                    "Struct {} not found in project for nested state generation",
                    type_name
                );
            }
        }
    }
    None
}
