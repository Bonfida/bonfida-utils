use convert_case::{Case, Casing};
use fs_extra::dir::get_dir_content;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

const CASE_STR_ID: [&str; 4] = [
    "TOBEREPLACEDBY_UPPERSNAKE",
    "TOBEREPLACEDBY_LOWERSNAKE",
    "TOBEREPLACEDBY_KEBAB",
    "TOBEREPLACEDBY_PASCAL",
];

pub fn generate(project_path: &str) {
    let now = Instant::now();

    let project_dir = std::path::Path::new(&project_path);
    let template_path = std::path::Path::new("./src/template/");

    copy_dir_all(template_path, project_dir).unwrap();

    let directory = get_dir_content(project_dir).unwrap().files;

    let project_name = project_dir.file_name().unwrap().to_str().unwrap();
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
