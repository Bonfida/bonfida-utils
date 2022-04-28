use std::{
    fs::{DirBuilder, File},
    process::Output,
    str::FromStr,
};

use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use solana_program::pubkey::Pubkey;

#[derive(Serialize)]
pub struct Measures {
    pub test_name: String,
    pub commit_id: String,
    pub x: Vec<u64>,
    pub y: Vec<u64>,
}

impl Measures {
    pub fn save(&self, output_file: &mut File) {
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

pub fn is_working_tree_clean() -> Result<(), &'static str> {
    let o = std::process::Command::new("git")
        .arg("diff")
        .arg("HEAD")
        .output()
        .unwrap()
        .stdout;
    if o.is_empty() {
        Ok(())
    } else {
        Err("Please commit or stash all changes before running the benchmark!")
    }
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

pub struct BenchRunner {
    re: Regex,
    test_name: &'static str,
    commit_id: String,
    output_file: File,
}

impl BenchRunner {
    pub fn new(test_name: &'static str, program_id: Pubkey) -> Self {
        is_working_tree_clean().unwrap();
        let commit_id = get_commit_id();
        let output_file = get_output_file(test_name, &commit_id)
            .expect("The benchmark has already been recorded for this commit");

        Self {
            re: Regex::new(&format!(
                "Program {} consumed (?P<val>.*) of .* compute units\n",
                program_id
            ))
            .unwrap(),
            test_name,
            commit_id,
            output_file,
        }
    }

    fn parse(&self, output: Output) -> Vec<u64> {
        let output = String::from_utf8(output.stderr).unwrap();
        let c = self.re.captures_iter(&output).collect::<Vec<_>>();
        let res = c
            .iter()
            .map(|k| k["val"].parse::<u64>().unwrap())
            .collect::<Vec<_>>();
        res
    }

    pub fn run(&self, arguments: &[String]) -> Vec<u64> {
        let mut command = std::process::Command::new("cargo");
        command.arg("test-bpf");
        command.arg("--test");
        command.arg(self.test_name);
        for (i, v) in arguments.iter().enumerate() {
            command.env(&format!("ARGUMENT_{}", i), v);
        }
        let output = command.output().unwrap();
        self.parse(output)
    }

    pub fn commit(mut self, x: Vec<u64>, y: Vec<u64>) {
        Measures {
            test_name: self.test_name.to_owned(),
            commit_id: self.commit_id.clone(),
            x,
            y,
        }
        .save(&mut self.output_file);
    }
}

pub fn get_env_arg<T: FromStr>(idx: usize) -> Option<T>
where
    <T as FromStr>::Err: std::fmt::Debug,
{
    std::env::var(&format!("ARGUMENT_{}", idx))
        .map(|s| s.parse::<T>().unwrap())
        .ok()
}

pub fn get_env_args() -> Vec<String> {
    let mut i = 0;
    let mut result = vec![];
    while let Ok(s) = std::env::var(&format!("ARGUMENT_{}", i)) {
        result.push(s);
        i += 1;
    }
    result
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
