use cargo_autoproject::generate;
use clap::{App, Arg};

fn main() {
    let matches = App::new("cargo-autoproject")
        .version("0.1")
        .author("Bonfida")
        .arg(Arg::with_name("name").hidden(true).required(true))
        .arg(
            Arg::with_name("project-name")
                .long("project-name")
                .takes_value(true)
                .default_value("src/instruction.rs")
                .help("Enter the project name, eg: my-project-name"),
        )
        .get_matches();
    let project_name = matches.value_of("project-name").unwrap();

    generate(project_name);
}
