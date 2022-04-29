use std::{
    collections::{hash_map::Entry, HashMap},
    fs::{read_dir, DirBuilder, File},
};

use bonfida_utils::bench::Measures;
use gnuplot::{Figure, PlotOption};
use regex::Regex;
fn main() {
    let current_directory = std::env::current_dir().unwrap();
    let results_directory = current_directory.join("benchmark_results");
    if !results_directory.is_dir() {
        eprintln!("Couldn't find the benchmark results directory");
        return;
    }

    let graph_output_path = results_directory.join("graphs");
    if graph_output_path.is_file() {
        eprintln!("Delete the graphs benchmark_results/graphs file");
        return;
    }

    let files = read_dir(&results_directory).unwrap();
    let mut test_results = HashMap::new();
    let file_name_re = Regex::new("(?P<test_name>.*)_(?P<commit_id>[^_]*)").unwrap();

    if !graph_output_path.is_dir() {
        let dir_builder = DirBuilder::new();
        dir_builder.create(&graph_output_path).unwrap();
    }
    for f in files {
        let path = f.unwrap().path();
        if !path.is_file() {
            continue;
        }
        let c = file_name_re
            .captures(path.file_stem().unwrap().to_str().unwrap())
            .unwrap();
        let test_name = c["test_name"].to_owned();
        let test_entry = test_results.entry(test_name);
        let results = match test_entry {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(o) => o.insert(vec![]),
        };
        results.push(path);
    }
    let commit_names = get_commit_names();
    for (name, results) in test_results {
        let mut figure = Figure::new();
        figure.set_title(&name.replace('_', "\\_"));
        let plot = figure.axes2d();
        for path in results {
            let file = File::open(&path).unwrap();
            let measures: Measures = serde_json::from_reader(file).unwrap();
            let caption = commit_names
                .get(&measures.commit_id)
                .cloned()
                .unwrap_or_else(|| {
                    String::from_utf8(measures.commit_id.as_bytes()[0..7].to_owned()).unwrap()
                });
            plot.lines_points(&measures.x, &measures.y, &[PlotOption::Caption(&caption)]);
        }

        let path = graph_output_path.join(&format!("{name}.png"));
        figure.save_to_png(&path, 1920, 1080).unwrap();
    }
}

fn get_commit_names() -> HashMap<String, String> {
    let output = std::process::Command::new("git")
        .arg("branch")
        .arg("--format")
        .arg("%(refname) %(objectname)")
        .output()
        .unwrap();
    let re = Regex::new("refs/heads/(?P<branch_name>.*) (?P<commit_id>[^_\n\r]*)").unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    let mut result = HashMap::new();
    for l in output.lines() {
        let captures = re.captures(l).unwrap();
        result.insert(
            captures["commit_id"].to_owned(),
            captures["branch_name"].to_owned(),
        );
    }
    result
}
