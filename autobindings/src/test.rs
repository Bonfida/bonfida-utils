use std::io::Write;
use std::process::Command;
use std::{fs::File, io::Read};

use convert_case::{Case, Casing};

use rand::distributions::Alphanumeric;
use rand::rngs::ThreadRng;
use rand::{Rng, RngCore};
use std::str::from_utf8;
use std::str::FromStr;
use std::vec;
use syn::{AngleBracketedGenericArguments, GenericArgument, PathArguments};
use syn::{Field, Path, Type, TypeArray, TypePath};

use crate::{find_struct, get_struct_fields, is_option, is_slice, snake_to_camel, snake_to_pascal};

pub fn test(instructions_path: &str) {
    let path = std::path::Path::new(instructions_path);
    let directory = std::fs::read_dir(path).unwrap();
    for d in directory {
        let file = d.unwrap();
        let module_name = std::path::Path::new(&file.file_name())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        let mut f = File::open(file.path().to_str().unwrap()).unwrap();
        let mut raw_string = String::new();
        f.read_to_string(&mut raw_string).unwrap();

        let ast: syn::File = syn::parse_str(&raw_string).unwrap();
        let accounts_struct_item = find_struct("Accounts", &ast);
        let params_struct_item = find_struct("Params", &ast);

        let params_fields = get_struct_fields(params_struct_item);
        let accounts_fields = get_struct_fields(accounts_struct_item);

        let mut js_params_inputs = vec![];
        let mut js_acc_inputs = vec![];
        let mut py_test_inputs = vec![];

        let mut rng = rand::thread_rng();

        println!("Testing instr {:?}", module_name);

        for Field {
            attrs: _,
            vis: _,
            ident,
            colon_token: _,
            ty,
        } in params_fields
        {
            let test_input;
            let snake_case_ident = ident.unwrap().to_string();
            if snake_case_ident != "_padding" {
                test_input = type_to_test_input(&ty, &mut rng, false);
                py_test_inputs = [
                    py_test_inputs,
                    test_input.iter().map(|e| e.py.clone()).collect(),
                ]
                .concat();
                js_params_inputs = [
                    js_params_inputs,
                    vec![snake_to_camel(&snake_case_ident) + ":"],
                    test_input.iter().map(|e| e.js.clone()).collect(),
                ]
                .concat();
            }
        }
        // Add pgr id
        let mut rand_key_bytes = [0; 32];
        rng.fill_bytes(&mut rand_key_bytes);
        js_acc_inputs.push(format!("new PublicKey({:?}),", rand_key_bytes));
        py_test_inputs.push(format!("PublicKey({:?}),", rand_key_bytes));

        for Field {
            attrs: _,
            vis: _,
            ident: _,
            colon_token: _,
            ty,
        } in accounts_fields
        {
            rng.fill_bytes(&mut rand_key_bytes);
            if is_slice(&ty) {
                js_acc_inputs.push(format!("[new PublicKey({:?})],", rand_key_bytes));
                py_test_inputs.push(format!("[PublicKey({:?})],", rand_key_bytes));
            } else if is_option(&ty) {
                js_acc_inputs.push("undefined,".to_owned());
                py_test_inputs.push("None,".to_owned());
            } else {
                js_acc_inputs.push(format!("new PublicKey({:?}),", rand_key_bytes));
                py_test_inputs.push(format!("PublicKey({:?}),", rand_key_bytes));
            }
        }

        let mut js_params_inputs_string = String::new();
        for s in js_params_inputs {
            js_params_inputs_string.push_str(&s);
            js_params_inputs_string.push('\n');
        }
        let mut js_acc_inputs_string = String::new();
        for s in js_acc_inputs {
            js_acc_inputs_string.push_str(&s);
            js_acc_inputs_string.push('\n');
        }
        let mut py_inputs_string = String::new();
        for s in py_test_inputs {
            py_inputs_string.push_str(&s);
            py_inputs_string.push('\n');
        }

        let mut py_test_file = File::create("../python/tmp_test.py").unwrap();
        let mut py_test_str = format!(
            "from src.raw_instructions import {}Instruction",
            snake_to_pascal(&module_name)
        );
        py_test_str.push_str("\nfrom solana.publickey import PublicKey");

        py_test_str.push_str(&format!(
            "\ninstr = {}Instruction().getInstruction(",
            snake_to_pascal(&module_name)
        ));
        py_test_str.push_str(&py_inputs_string);
        py_test_str.push(')');
        py_test_str.push_str("\nprint(instr.program_id)");
        py_test_str.push_str("\nfor k in instr.keys:");
        py_test_str.push_str("\n\tprint(\"Account \" + str(k.pubkey))");
        py_test_str.push_str("\n\tprint(k.is_signer)");
        py_test_str.push_str("\n\tprint(k.is_writable)");
        py_test_str.push_str("\nprint(instr.data.hex())");
        py_test_file.write_all(py_test_str.as_bytes()).unwrap();

        let py_output = Command::new("python")
            .arg("../python/tmp_test.py")
            .output()
            .expect("failed to execute process");
        let py_output_str = from_utf8(&py_output.stdout).unwrap();
        // println!("{:?}", py_output_str);
        // println!("ERR {:?}", from_utf8(&py_output.stderr).unwrap());
        let py_results = parse_python_output(py_output_str.to_owned());

        let mut js_test_file = File::create("../js/tmp_test.ts").unwrap();
        let mut js_test_str = format!(
            "import {{ PublicKey }} from \"@solana/web3.js\";
            import BN from \"bn.js\";
            import {{ {}Instruction }} from \"./src/raw_instructions\";",
            snake_to_camel(&module_name)
        );
        if !js_params_inputs_string.is_empty() {
            js_params_inputs_string = "{".to_owned() + &js_params_inputs_string + "}";
        }
        js_test_str.push_str(&format!(
            "const ix = new {}Instruction({}).getInstruction(",
            snake_to_camel(&module_name),
            js_params_inputs_string
        ));
        js_test_str.push_str(&js_acc_inputs_string);
        js_test_str.push_str(");");
        js_test_str.push_str(
            "console.log(ix.programId.toString());
        for (let a of ix.keys) {
          console.log(\"Account \" + a.pubkey.toString());
          console.log(a.isSigner);
          console.log(a.isWritable);
        }
        console.log(ix.data.toString(\"hex\"));",
        );
        js_test_file.write_all(js_test_str.as_bytes()).unwrap();

        let js_output = Command::new("ts-node")
            .arg("../js/tmp_test.ts")
            .output()
            .expect("failed to execute ts test process (please install ts-node)");
        let js_output_str = from_utf8(&js_output.stdout).unwrap();
        let js_results = parse_js_output(js_output_str.to_owned());

        // println!("{:?}", py_results);
        // println!("{:?}", js_results);

        if py_results != js_results {
            println!(
                "ERROR: Py results {:?} | Js results: {:?}",
                py_results, js_results
            );
            panic!()
        }
    }
    println!("Success: All tests passing")
}

#[derive(Clone, Debug)]
struct TestInput {
    js: String,
    py: String,
}

fn type_to_test_input(ty: &Type, rng: &mut ThreadRng, array: bool) -> Vec<TestInput> {
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
                    let range_max = &simple_type[1..].parse::<i32>().unwrap();
                    let input = rng.gen_range(0..*range_max).to_string();
                    vec![TestInput {
                        py: input.clone() + ",",
                        js: if array
                            || ["u8", "u16", "u32", "i8", "i16", "i32"]
                                .contains(&simple_type.as_str())
                        {
                            input + ","
                        } else {
                            "new BN(".to_owned() + &input + "),"
                        },
                    }]
                }
                "String" => {
                    let str_length: usize = rng.gen_range(0..100);
                    let input = "\"".to_owned()
                        + &rng
                            .sample_iter(&Alphanumeric)
                            .take(str_length)
                            .map(char::from)
                            .collect::<String>()
                        + "\",";

                    vec![TestInput {
                        py: input.clone(),
                        js: input,
                    }]
                }
                "Pubkey" => {
                    let mut rand_key_bytes = [0; 32];
                    rng.fill_bytes(&mut rand_key_bytes);

                    vec![TestInput {
                        py: format!("{:?},", rand_key_bytes),
                        js: format!("new Uint8Array({:?}),", rand_key_bytes),
                    }]
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
                            let mut inputs = vec![TestInput {
                                js: "[".to_owned(),
                                py: "[".to_owned(),
                            }];
                            let input_len = rng.gen_range(0..255);
                            for _ in 0..input_len {
                                let inner_type = type_to_test_input(t, rng, true);
                                inputs.push(inner_type[0].clone());
                            }
                            inputs.push(TestInput {
                                js: "],".to_owned(),
                                py: "],".to_owned(),
                            });
                            return inputs;
                        } else {
                            unimplemented!()
                        }
                    } else {
                        unreachable!()
                    };
                }
                _ => {
                    let input = rng.gen_range(0..255).to_string() + ",";
                    vec![TestInput {
                        js: input.clone(),
                        py: input,
                    }]
                } // We assume this is an enum
            }
        }
        Type::Array(TypeArray {
            bracket_token: _,
            elem,
            semi_token: _,
            len: _,
        }) => {
            let mut is_uint8arr = false;
            let mut inputs = vec![TestInput {
                js: if let Type::Path(TypePath {
                    qself: _,
                    path:
                        Path {
                            leading_colon: _,
                            segments,
                        },
                }) = *(elem.to_owned())
                {
                    let segment = segments.iter().next().unwrap();
                    let simple_type = segment.ident.to_string();
                    if simple_type == "u8" {
                        is_uint8arr = true;
                        "new Uint8Array([".to_owned()
                    } else {
                        "[".to_owned()
                    }
                } else {
                    unimplemented!()
                },
                py: "[".to_owned(),
            }];
            let input_len = 32; //rng.gen_range(0..255);
            for _ in 0..input_len {
                let inner_type = type_to_test_input(elem, rng, true)[0].clone();
                inputs.push(inner_type);
            }
            inputs.push(TestInput {
                js: if is_uint8arr {
                    "]),".to_owned()
                } else {
                    "],".to_owned()
                },
                py: "],".to_owned(),
            });
            inputs
        }
        _ => unimplemented!(),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TestResult {
    program_id: String,
    accounts: Vec<TestResultAccount>,
    data: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
struct TestResultAccount {
    key: String,
    is_signer: bool,
    is_writable: bool,
}

fn parse_python_output(py_output: String) -> TestResult {
    let mut lines = py_output.lines();
    let mut result_accounts = vec![];
    let pgr_id = lines.next().unwrap();

    let mut curr_line = lines.next().unwrap();

    while curr_line.contains("Account ") {
        result_accounts.push(TestResultAccount {
            key: curr_line.to_owned(),
            is_signer: bool::from_str(
                &lines
                    .next()
                    .unwrap()
                    .from_case(Case::Pascal)
                    .to_case(Case::Lower),
            )
            .unwrap(),
            is_writable: bool::from_str(
                &lines
                    .next()
                    .unwrap()
                    .from_case(Case::Pascal)
                    .to_case(Case::Lower),
            )
            .unwrap(),
        });
        curr_line = lines.next().unwrap();
    }

    let data_bytes = hex::decode(curr_line).unwrap();

    TestResult {
        program_id: pgr_id.to_owned(),
        accounts: result_accounts,
        data: data_bytes,
    }
}

fn parse_js_output(js_output: String) -> TestResult {
    let mut lines = js_output.lines();
    let mut result_accounts = vec![];
    let pgr_id = lines.next().unwrap();

    let mut curr_line = lines.next().unwrap();

    while curr_line.contains("Account ") {
        result_accounts.push(TestResultAccount {
            key: curr_line.to_owned(),
            is_signer: bool::from_str(
                &lines
                    .next()
                    .unwrap()
                    .from_case(Case::Pascal)
                    .to_case(Case::Lower),
            )
            .unwrap(),
            is_writable: bool::from_str(
                &lines
                    .next()
                    .unwrap()
                    .from_case(Case::Pascal)
                    .to_case(Case::Lower),
            )
            .unwrap(),
        });
        curr_line = lines.next().unwrap();
    }

    let data_bytes = hex::decode(curr_line).unwrap();

    TestResult {
        program_id: pgr_id.to_owned(),
        accounts: result_accounts,
        data: data_bytes,
    }
}
