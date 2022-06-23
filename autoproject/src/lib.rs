use clap::{crate_name, crate_version, Arg, ArgMatches, Command};
use convert_case::{Case, Casing};
use fs_extra::dir::get_dir_content;
use include_dir::{include_dir, Dir};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;

const CASE_STR_ID: [&str; 4] = [
    "TOBEREPLACEDBY_UPPERSNAKE",
    "TOBEREPLACEDBY_LOWERSNAKE",
    "TOBEREPLACEDBY_KEBAB",
    "TOBEREPLACEDBY_PASCAL",
];

const TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/template");

pub fn command() -> Command<'static> {
    Command::new(crate_name!())
        .version(crate_version!())
        .author("Bonfida")
        .about("Initialize a new project")
        .allow_external_subcommands(true)
        .arg(
            Arg::new("name")
                .required(true)
                .takes_value(true)
                .help("The new project's name"),
        )
        .arg(
            Arg::new("new-project-path")
                .long("path")
                .short('p')
                .takes_value(true)
                .required(false)
                .help("Path in which to create the new project. Default to current directory."),
        )
}

pub fn process(matches: &ArgMatches) {
    let project_name = matches.value_of("name").unwrap();
    let project_path = matches.value_of("new-project-path").unwrap();
    generate(project_name, project_path);
}

pub fn generate(project_name: &str, project_path: &str) {
    let now = Instant::now();

    let mut project_dir = std::path::PathBuf::from_str(&project_path).unwrap();
    project_dir.push(project_name);

    println!("{:?}", project_dir);

    // return;

    TEMPLATE.extract(&project_dir).unwrap();

    let directory = get_dir_content(project_dir).unwrap().files;

    for file_path_str in directory {
        let file_path = Path::new(&file_path_str);
        let mut raw_file = std::fs::read_to_string(&file_path).unwrap();

        for case_id_str in CASE_STR_ID.iter() {
            raw_file = raw_file.replace(
                case_id_str,
                &project_name
                    .from_case(Case::Kebab)
                    .to_case(get_case_from_id(case_id_str)),
            );
        }

        let mut out_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&file_path)
            .unwrap();
        out_file.write_all(raw_file.as_bytes()).unwrap();
    }

    let elapsed = now.elapsed();
    println!("✨  Done in {:.2?}", elapsed);
}

fn get_case_from_id(id_str: &str) -> Case {
    match id_str {
        "TOBEREPLACEDBY_UPPERSNAKE" => Case::UpperSnake,
        "TOBEREPLACEDBY_LOWERSNAKE" => Case::Snake,
        "TOBEREPLACEDBY_KEBAB" => Case::Kebab,
        "TOBEREPLACEDBY_PASCAL" => Case::Pascal,
        _ => panic!(),
    }
}

use std::{fs, io};

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
