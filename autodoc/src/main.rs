use cargo_autodoc::{generate, utils::generate_table};
use clap::{App, Arg};

fn main() {
    let matches = App::new("cargo-autodoc")
        .version("0.1")
        .author("Bonfida")
        .arg(Arg::with_name("name").hidden(true).required(true))
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
        .get_matches();
    let instructions_path = matches.value_of("instructions_path").unwrap();
    let instructions_enum_path = matches.value_of("instructions_enum_path").unwrap();
    generate(
        instructions_path,
        instructions_enum_path,
        "src/instructions_auto.rs",
    );
}
