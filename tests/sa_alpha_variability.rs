use std::{fs, io::Write, path::PathBuf, time::Instant};

use vrptw_code::{
    graph_utils::plot_multi_series_line,
    optimizers::simulated_annealing::{self, SAParams},
    parser::InputData,
    problem::{Problem, Solution},
};

const RUNS_PER_ALPHA: usize = 8;

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
#[ignore = "SA alpha variability: long-running, run explicitly"]
fn test_sa_alpha_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading VRP instances...");
    let files = get_vrp_files()?;
    fs::create_dir_all("plots")?;

    // Alpha grid to explore (coarse -> fine around high values)
    let alphas = vec![
        0.90, 0.92, 0.94, 0.96, 0.98, 0.99, 0.992, 0.995, 0.997, 0.999,
    ];

    for file in files {
        let fname = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("instance");
        println!("Processing {}...", fname);

        let raw = fs::read_to_string(&file)?;
        let parsed = InputData::parse_input(&raw);
        let problem = Problem::new(parsed);

        // Accumulate per-alpha stats
        let mut csv = fs::File::create(format!("plots/sa_alpha_{}.csv", fname))?;
        writeln!(
            csv,
            "alpha,mean_distance,std_distance,feasible_rate,mean_elapsed_ms,mean_iterations"
        )?;

        let mut series_points: Vec<(f64, f64)> = Vec::new();

        for &alpha in &alphas {
            let mut distances = Vec::new();
            let mut times_ms = Vec::new();
            let mut iterations = Vec::new();
            let mut feasible = 0usize;

            for _ in 0..RUNS_PER_ALPHA {
                let solution = Solution::simplest(&problem);
                let params = SAParams {
                    t_initial: 500.0,
                    t_final: 0.1,
                    alpha,
                    iter_per_temp: 150,
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
            let feasible_rate = feasible as f64 / RUNS_PER_ALPHA as f64;

            writeln!(
                csv,
                "{:.3},{:.6},{:.6},{:.3},{:.3},{:.1}",
                alpha, mean_d, std_d, feasible_rate, mean_t, mean_iter
            )?;

            series_points.push((alpha, mean_d));
        }

        // Plot alpha vs mean distance
        let series = vec![("mean_distance", series_points.clone())];
        let plot_path = format!("plots/sa_alpha_{}.png", fname);
        plot_multi_series_line(
            &plot_path,
            &format!("SA: α sensitivity — {}", fname),
            "alpha",
            "mean distance",
            &series,
        )?;

        println!("  Saved plots/sa_alpha_{}.png and .csv", fname);
    }

    println!("SA alpha variability analysis complete. CSVs and PNGs in plots/");
    Ok(())
}
