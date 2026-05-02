use std::{fs, io::Write, path::PathBuf, time::Instant};

use vrptw_code::{
    graph_utils::plot_multi_series_line,
    optimizers::simulated_annealing::{self, SAParams},
    parser::InputData,
    problem::{Problem, Solution},
};

const RUNS_PER_CONFIG: usize = 6;

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn std_dev(values: &[f64]) -> f64 {
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

fn get_vrp_files() -> Result<Vec<PathBuf>, std::io::Error> {
    let path = PathBuf::from("./data/");
    let mut files: Vec<PathBuf> = fs::read_dir(&path)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("vrp"))
        .collect();
    files.sort();
    Ok(files)
}

#[test]
#[ignore = "SA iter_per_temp variability: long-running, run explicitly"]
fn test_sa_iter_per_temp_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading VRP instances...");
    let files = get_vrp_files()?;
    fs::create_dir_all("plots")?;

    // Values to test for iterations per temperature
    let iter_values = vec![10usize, 30, 50, 100, 150, 300, 600];

    for file in files {
        let fname = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("instance");
        println!("Processing {}...", fname);

        let raw = fs::read_to_string(&file)?;
        let parsed = InputData::parse_input(&raw);
        let problem = Problem::new(parsed);

        let mut csv = fs::File::create(format!("plots/sa_iter_{}.csv", fname))?;
        writeln!(
            csv,
            "iter_per_temp,mean_distance,std_distance,feasible_rate,mean_elapsed_ms,mean_iterations"
        )?;

        let mut series_points: Vec<(f64, f64)> = Vec::new();

        for &iter_per_temp in &iter_values {
            let mut distances = Vec::new();
            let mut times_ms = Vec::new();
            let mut iterations = Vec::new();
            let mut feasible = 0usize;

            for _ in 0..RUNS_PER_CONFIG {
                let solution = Solution::simplest(&problem);
                let params = SAParams {
                    t_initial: 500.0,
                    t_final: 0.1,
                    alpha: 0.995,
                    iter_per_temp,
                };

                let mut alg =
                    simulated_annealing::build_algorithm(&problem, &solution, &params, true);

                let start = Instant::now();
                alg.step(&problem, usize::MAX);
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;

                let best = alg.current_solution();
                distances.push(best.total_distance(&problem));
                times_ms.push(elapsed);
                iterations.push(alg.total_iterations() as f64);
                if best.is_feasible(&problem) {
                    feasible += 1;
                }
            }

            let mean_d = mean(&distances);
            let std_d = std_dev(&distances);
            let mean_t = mean(&times_ms);
            let mean_iter = mean(&iterations);
            let feasible_rate = feasible as f64 / RUNS_PER_CONFIG as f64;

            writeln!(
                csv,
                "{},{:.6},{:.6},{:.3},{:.3},{:.1}",
                iter_per_temp, mean_d, std_d, feasible_rate, mean_t, mean_iter
            )?;

            series_points.push((iter_per_temp as f64, mean_d));
        }

        let series = vec![("mean_distance", series_points.clone())];
        let plot_path = format!("plots/sa_iter_{}.png", fname);
        plot_multi_series_line(
            &plot_path,
            &format!("SA: iter_per_temp sensitivity — {}", fname),
            "iter_per_temp",
            "mean distance",
            &series,
        )?;

        println!("  Saved plots/sa_iter_{}.png and .csv", fname);
    }

    println!("SA iter_per_temp variability analysis complete. CSVs and PNGs in plots/");
    Ok(())
}
