use std::path::PathBuf;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use vrptw_code::{
    optimizers::{
        aco::AcoParams,
        simulated_annealing::{self, SAParams},
    },
    problem::{self, Solution},
};

fn get_vrp_files() -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let path = std::path::Path::new("./data/");
    let entries = std::fs::read_dir(path)?;

    let mut files = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("vrp") {
            files.push(path);
        }
    }
    Ok(files)
}

fn default_sa(files: &Vec<PathBuf>) -> Vec<(&PathBuf, f64)> {
    files
        .par_iter()
        .map(|file| {
            let data = std::fs::read_to_string(file).expect("Couldn't open file");
            let parsed_input = vrptw_code::parser::InputData::parse_input(data.as_str());
            let pb = problem::Problem::new(parsed_input);
            let solution = Solution::simplest(&pb);
            let sa_params = SAParams::default();
            let mut algorithm =
                simulated_annealing::build_algorithm(&pb, &solution, &sa_params, true);
            algorithm.step(&pb, usize::MAX);
            let best_dist = algorithm.current_solution().total_distance(&pb);
            (file, best_dist)
        })
        .collect::<Vec<(&PathBuf, f64)>>()
}

fn default_aco(files: &Vec<PathBuf>) -> Vec<(&PathBuf, f64)> {
    files
        .par_iter()
        .map(|file| {
            let data = std::fs::read_to_string(file).expect("Couldn't open file");
            let parsed_input = vrptw_code::parser::InputData::parse_input(data.as_str());
            let pb = problem::Problem::new(parsed_input);
            let solution = Solution::simplest(&pb);
            let aco_params = AcoParams::default();
            let mut algorithm =
                vrptw_code::optimizers::aco::build_algorithm(&pb, &solution, &aco_params, true);
            algorithm.step(&pb, usize::MAX);
            let best_dist = algorithm.current_solution().total_distance(&pb);
            (file, best_dist)
        })
        .collect::<Vec<(&PathBuf, f64)>>()
}

#[test]
fn test_comparaison() -> Result<(), Box<dyn std::error::Error>> {
    let files = get_vrp_files()?;

    let sa_results = default_sa(&files);
    let aco_results = default_aco(&files);

    println!("Comparaison done for all files.");
    println!("SA Results: {:?}", sa_results);
    println!("ACO Results: {:?}", aco_results);
    Ok(())
}
