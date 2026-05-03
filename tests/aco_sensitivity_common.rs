#![allow(dead_code)]

use std::{fs, io::Write, path::PathBuf, time::Instant};

use vrptw_code::{
    graph_utils::plot_multi_series_line,
    optimizers::aco,
    parser::InputData,
    problem::{Problem, Solution},
};

pub const RUNS_PER_CONFIG: usize = 6;

pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

pub fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let avg = mean(values);
    let var = values
        .iter()
        .map(|v| {
            let d = v - avg;
            d * d
        })
        .sum::<f64>()
        / values.len() as f64;
    var.sqrt()
}

pub fn get_vrp_files() -> Result<Vec<PathBuf>, std::io::Error> {
    let path = PathBuf::from("./data/");
    let mut files: Vec<PathBuf> = fs::read_dir(&path)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("vrp"))
        .collect();
    files.sort();
    Ok(files)
}

pub fn load_problems() -> Result<Vec<(String, Problem)>, Box<dyn std::error::Error>> {
    let mut problems = Vec::new();
    for file in get_vrp_files()? {
        let raw = fs::read_to_string(&file)?;
        let parsed = InputData::parse_input(&raw);
        let problem = Problem::new(parsed);
        let name = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("instance")
            .to_string();
        problems.push((name, problem));
    }
    Ok(problems)
}

pub fn run_aco_sweep<FParams, FValue, FX>(
    param_name: &str,
    csv_name: &str,
    plot_title: &str,
    x_label: &str,
    values: &[FValue],
    make_params: FParams,
    to_x: FX,
) -> Result<(), Box<dyn std::error::Error>>
where
    FValue: Copy,
    FParams: Fn(FValue) -> aco::AcoParams,
    FX: Fn(FValue) -> f64,
{
    println!("Loading VRP instances...");
    let problems = load_problems()?;
    fs::create_dir_all("plots")?;

    for (instance_name, problem) in problems {
        println!("Processing {}...", instance_name);

        let mut csv = fs::File::create(format!("plots/{}_{}.csv", csv_name, instance_name))?;
        writeln!(
            csv,
            "{},mean_distance,std_distance,feasible_rate,mean_elapsed_ms",
            param_name
        )?;

        let mut series_points: Vec<(f64, f64)> = Vec::new();

        for &value in values {
            let x = to_x(value);
            let mut distances = Vec::new();
            let mut times_ms = Vec::new();
            let mut feasible = 0usize;

            for _ in 0..RUNS_PER_CONFIG {
                let solution = Solution::simplest(&problem);
                let params = make_params(value);
                let mut algorithm = aco::build_algorithm(&problem, &solution, &params, true);

                let start = Instant::now();
                algorithm.step(&problem, usize::MAX);
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;

                let best = algorithm.current_solution();
                distances.push(best.total_distance(&problem));
                times_ms.push(elapsed);
                if best.is_feasible(&problem) {
                    feasible += 1;
                }
            }

            let mean_d = mean(&distances);
            let std_d = std_dev(&distances);
            let mean_t = mean(&times_ms);
            let feasible_rate = feasible as f64 / RUNS_PER_CONFIG as f64;

            writeln!(
                csv,
                "{:.6},{:.6},{:.6},{:.3},{:.3}",
                x, mean_d, std_d, feasible_rate, mean_t
            )?;

            series_points.push((x, mean_d));
        }

        let series = vec![("mean_distance", series_points)];
        let plot_path = format!("plots/{}_{}.png", csv_name, instance_name);
        plot_multi_series_line(
            &plot_path,
            &format!("{} — {}", plot_title, instance_name),
            x_label,
            "mean distance",
            &series,
        )?;

        println!("  Saved plots/{}_{}.png and .csv", csv_name, instance_name);
    }

    println!(
        "ACO {} variability analysis complete. CSVs and PNGs in plots/",
        param_name
    );
    Ok(())
}
