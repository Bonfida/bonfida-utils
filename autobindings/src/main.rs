use cargo_autobindings::generate;
use clap::{App, Arg};

fn main() {
    let matches = App::new("cargo-autobindings")
        .version("0.1")
        .author("Bonfida")
        .arg(Arg::with_name("name").hidden(true).required(true))
        .arg(
            Arg::with_name("instructions_path")
                .takes_value(true)
                .default_value("src/processor"),
        )
        .get_matches();
    let instructions_path = matches.value_of("instructions_path").unwrap();
    generate(instructions_path, "../js/src/raw_instructions.ts");
}
