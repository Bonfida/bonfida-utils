use std::{
    fs::{DirBuilder, File},
    process::Output,
};

use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use solana_program::pubkey::Pubkey;

#[derive(Serialize)]
pub struct Measures {
    test_name: String,
    commit_id: String,
    x: Vec<u64>,
    y: Vec<u64>,
}

impl Measures {
    pub fn save(&self, output_file: File) {
        serde_json::to_writer(output_file, &self).unwrap();
    }
}

pub fn get_commit_id() -> String {
    let o = std::process::Command::new("git")
        .arg("log")
        .output()
        .unwrap()
        .stdout;
    let end = o.iter().position(|s| *s == 10 || *s == 13).unwrap();
    let output = String::from_utf8(o[0..end].to_owned()).unwrap();
    lazy_static! {
        static ref RE: Regex = Regex::new("commit (.*)").unwrap();
    }
    RE.captures(&output).unwrap()[1].to_owned()
}

pub fn is_working_tree_clean() -> bool {
    let o = std::process::Command::new("git")
        .arg("diff")
        .arg("HEAD")
        .output()
        .unwrap()
        .stdout;
    o.is_empty()
}

pub fn get_output_file(test_name: &str, commit_id: &str) -> Option<File> {
    let current_directory = std::env::current_dir().unwrap();
    let output_dir = current_directory.join("benchmark_results");
    if output_dir.is_file() {
        panic!("Delete or rename the benchmark_results file");
    }
    if !output_dir.exists() {
        let d = DirBuilder::new();
        d.create(&output_dir).unwrap();
    }
    let file_path = output_dir.join(&format!("{}_{}.json", test_name, commit_id));
    if file_path.exists() {
        return None;
    }

    Some(File::create(file_path).unwrap())
}

pub struct LogParser {
    re: Regex,
}

impl LogParser {
    pub fn new(program_id: Pubkey) -> Self {
        Self {
            re: Regex::new(&format!(
                "Program {} consumed (?P<val>.*) of .* compute units\n",
                program_id
            ))
            .unwrap(),
        }
    }

    pub fn parse(&self, output: Output) -> Vec<u64> {
        let output = String::from_utf8(output.stderr).unwrap();
        let c = self.re.captures_iter(&output).collect::<Vec<_>>();
        let res = c
            .iter()
            .map(|k| k["val"].parse::<u64>().unwrap())
            .collect::<Vec<_>>();
        res
    }
}

#[macro_export]
macro_rules! test_name {
    () => {
        std::path::Path::new(file!())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
    };
}
