use cargo_autobindings::{generate, TargetLang};
use clap::{App, Arg};

fn main() {
    let matches = App::new("cargo-autobindings")
        .version("0.1")
        .author("Bonfida")
        .arg(Arg::with_name("name").hidden(true).required(true))
        .arg(
            Arg::with_name("instr-path")
                .long("instructions-path")
                .takes_value(true)
                .default_value("src/processor"),
        )
        .arg(
            Arg::with_name("instr-enum-path")
                .long("instructions-enum-path")
                .takes_value(true)
                .default_value("src/instruction.rs"),
        )
        .arg(
            Arg::with_name("target-lang")
                .long("target-language")
                .takes_value(true)
                .default_value("js")
                .help("Enter \"py\" or \"js\""),
        )
        .get_matches();
    let instructions_path = matches.value_of("instr-path").unwrap();
    let instructions_enum_path = matches.value_of("instr-enum-path").unwrap();
    let target_lang_str = matches.value_of("target-lang").unwrap();
    let target_lang = match target_lang_str {
        "js" | "javascript" => TargetLang::Javascript,
        "py" | "python" => TargetLang::Python,
        _ => {
            println!("Target language must be javascript or python");
            panic!()
        }
    };
    generate(
        instructions_path,
        instructions_enum_path,
        target_lang,
        match target_lang {
            TargetLang::Javascript => "../js/src/raw_instructions.ts",
            TargetLang::Python => "../python/src/raw_instructions.py",
        },
    );
}
