use std::{
    collections::{hash_map::Entry, HashMap},
    fs::{read_dir, DirBuilder, File},
};

use gnuplot::{Figure, PlotOption};
use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Measures {
    commit_id: String,
    x: Vec<u64>,
    y: Vec<u64>,
}
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
    let mut figures = HashMap::new();
    let file_name_re = Regex::new("(?P<test_name>.*)_(?P<commit_id>[^_]*)").unwrap();
    for f in files {
        let path = f.unwrap().path();
        if !path.is_file() {
            continue;
        }
        let file = File::open(&path).unwrap();
        let c = file_name_re
            .captures(path.file_stem().unwrap().to_str().unwrap())
            .unwrap();
        let test_name = &c["test_name"];
        println!("{test_name}");
        let figure_entry = figures.entry(test_name.to_owned());
        let figure = match figure_entry {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(o) => o
                .insert(Figure::new())
                .set_title(&test_name.replace('_', "\\_")),
        };
        let measures: Measures = serde_json::from_reader(file).unwrap();
        figure.axes2d().lines_points(
            &measures.x,
            &measures.y,
            &[PlotOption::Caption(
                &String::from_utf8(measures.commit_id.into_bytes()[0..7].to_owned()).unwrap(),
            )],
        );
    }
    if !graph_output_path.is_dir() {
        let dir_builder = DirBuilder::new();
        dir_builder.create(&graph_output_path).unwrap();
    }
    for (name, mut fig) in figures {
        let path = graph_output_path.join(&format!("{name}.png"));
        fig.save_to_png(&path, 1920, 1080).unwrap();
    }
}
