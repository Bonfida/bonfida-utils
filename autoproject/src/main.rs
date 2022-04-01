use std::env;

use cargo_autoproject::generate;

fn main() {
    // let matches = Command::new("cargo-autoproject")
    //     .version("0.1")
    //     .author("Bonfida")
    //     .arg(Arg::new("name").hide(true).required(true))
    //     .arg(
    //         Arg::new("new-project-path")
    //             .long("new-project-path")
    //             .takes_value(true)
    //             .help("Enter the project name, eg: path/to/my-project-name"),
    //     )
    //     .get_matches();
    // let project_name = matches.value_of("new-project-path").unwrap();

    let args: Vec<String> = env::args().collect();
    let project_path = &args[1];
    generate(project_path);
}
