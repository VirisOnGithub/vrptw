use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use vrptw_code::{
    graph_utils::plot_multi_series_line,
    optimizers::{
        aco, aco::AcoParams, hill_climbing, hill_climbing::HCParams, simulated_annealing,
        simulated_annealing::SAParams,
    },
    parser::InputData,
    problem::{Problem, Solution},
};

const RUNS_PER_CONFIG: usize = 5; // Fewer runs per param combo to keep total time reasonable
const TEST_INSTANCES: usize = 3; // Test on subset of instances

#[derive(Debug, Clone)]
struct ParamResult {
    config_label: String,
    mean_distance: f64,
    std_distance: f64,
    mean_elapsed_ms: f64,
    feasible_rate: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// UTILITIES
// ═══════════════════════════════════════════════════════════════════════════════

fn get_vrp_files() -> Result<Vec<PathBuf>, std::io::Error> {
    let path = Path::new("./data/");
    let entries = fs::read_dir(path)?;

    let mut files = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("vrp") {
            files.push(path);
        }
    }
    files.sort();
    files.truncate(TEST_INSTANCES);
    Ok(files)
}

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

// ═══════════════════════════════════════════════════════════════════════════════
// SIMULATED ANNEALING PARAMETER SWEEP
// ═══════════════════════════════════════════════════════════════════════════════

fn run_sa_sweep(problems: &[Problem]) -> Result<Vec<ParamResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    // Grid: t_initial, alpha, iter_per_temp
    let t_initials = vec![100.0, 500.0, 1000.0];
    let alphas = vec![0.95, 0.995, 0.999];
    let iter_per_temps = vec![50, 150, 300];

    for &t_init in &t_initials {
        for &alpha in &alphas {
            for &iter_per_temp in &iter_per_temps {
                let label = format!(
                    "SA[t_init={:.0},α={:.3},iter={:>3}]",
                    t_init, alpha, iter_per_temp
                );

                let mut all_distances = Vec::new();
                let mut all_times = Vec::new();
                let mut feasible_count = 0;
                let mut total_runs = 0;

                for problem in problems {
                    for _ in 0..RUNS_PER_CONFIG {
                        total_runs += 1;
                        let solution = Solution::simplest(problem);
                        let params = SAParams {
                            t_initial: t_init,
                            t_final: 0.1,
                            alpha,
                            iter_per_temp,
                        };

                        let mut algorithm =
                            simulated_annealing::build_algorithm(problem, &solution, &params, true);

                        let start = Instant::now();
                        algorithm.step(problem, usize::MAX);
                        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

                        let best = algorithm.current_solution();
                        all_distances.push(best.total_distance(problem));
                        all_times.push(elapsed);
                        if best.is_feasible(problem) {
                            feasible_count += 1;
                        }
                    }
                }

                results.push(ParamResult {
                    config_label: label,
                    mean_distance: mean(&all_distances),
                    std_distance: std_dev(&all_distances),
                    mean_elapsed_ms: mean(&all_times),
                    feasible_rate: feasible_count as f64 / total_runs as f64,
                });
            }
        }
    }

    Ok(results)
}

// ═══════════════════════════════════════════════════════════════════════════════
// ANT COLONY OPTIMIZATION PARAMETER SWEEP
// ═══════════════════════════════════════════════════════════════════════════════

fn run_aco_sweep(problems: &[Problem]) -> Result<Vec<ParamResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    // Grid: n_ants, alpha, beta (smaller grid to keep runtime reasonable)
    let n_ants_list = vec![10, 20, 50];
    let alphas = vec![0.5, 1.0, 2.0];
    let betas = vec![1.5, 2.5, 3.5];

    for &n_ants in &n_ants_list {
        for &alpha in &alphas {
            for &beta in &betas {
                let label = format!("ACO[ants={:>2},α={:.1},β={:.1}]", n_ants, alpha, beta);

                let mut all_distances = Vec::new();
                let mut all_times = Vec::new();
                let mut feasible_count = 0;
                let mut total_runs = 0;

                for problem in problems {
                    for _ in 0..RUNS_PER_CONFIG {
                        total_runs += 1;
                        let solution = Solution::simplest(problem);
                        let params = AcoParams {
                            n_ants,
                            alpha,
                            beta,
                            rho: 0.1,
                            q: 100.0,
                            tau_min: 0.01,
                            tau_max: 10.0,
                            max_iterations: 500,
                            k_candidates: 10,
                            use_two_opt: true,
                        };

                        let mut algorithm = aco::build_algorithm(problem, &solution, &params, true);

                        let start = Instant::now();
                        algorithm.step(problem, usize::MAX);
                        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

                        let best = algorithm.current_solution();
                        all_distances.push(best.total_distance(problem));
                        all_times.push(elapsed);
                        if best.is_feasible(problem) {
                            feasible_count += 1;
                        }
                    }
                }

                results.push(ParamResult {
                    config_label: label,
                    mean_distance: mean(&all_distances),
                    std_distance: std_dev(&all_distances),
                    mean_elapsed_ms: mean(&all_times),
                    feasible_rate: feasible_count as f64 / total_runs as f64,
                });
            }
        }
    }

    Ok(results)
}

// ═══════════════════════════════════════════════════════════════════════════════
// HILL CLIMBING (NO PARAMS, BUT BASELINE)
// ═══════════════════════════════════════════════════════════════════════════════

fn run_hc_baseline(problems: &[Problem]) -> Result<Vec<ParamResult>, Box<dyn std::error::Error>> {
    let mut all_distances = Vec::new();
    let mut all_times = Vec::new();
    let mut feasible_count = 0;
    let mut total_runs = 0;

    for problem in problems {
        for _ in 0..RUNS_PER_CONFIG {
            total_runs += 1;
            let solution = Solution::simplest(problem);
            let params = HCParams::default();

            let mut algorithm = hill_climbing::build_algorithm(problem, &solution, &params, true);

            let start = Instant::now();
            algorithm.step(problem, usize::MAX);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;

            let best = algorithm.current_solution();
            all_distances.push(best.total_distance(problem));
            all_times.push(elapsed);
            if best.is_feasible(problem) {
                feasible_count += 1;
            }
        }
    }

    Ok(vec![ParamResult {
        config_label: "HC [baseline]".to_string(),
        mean_distance: mean(&all_distances),
        std_distance: std_dev(&all_distances),
        mean_elapsed_ms: mean(&all_times),
        feasible_rate: feasible_count as f64 / total_runs as f64,
    }])
}

// ═══════════════════════════════════════════════════════════════════════════════
// REPORT GENERATION
// ═══════════════════════════════════════════════════════════════════════════════

fn write_csv_report(
    path: &str,
    algo_name: &str,
    results: &[ParamResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(path)?;
    writeln!(
        file,
        "algorithm,config,mean_distance,std_distance,mean_elapsed_ms,feasible_rate"
    )?;

    for res in results {
        writeln!(
            file,
            "{},{},{:.6},{:.6},{:.3},{:.3}",
            algo_name,
            res.config_label,
            res.mean_distance,
            res.std_distance,
            res.mean_elapsed_ms,
            res.feasible_rate
        )?;
    }

    Ok(())
}

fn write_markdown_report(
    path: &str,
    algo_name: &str,
    results: &[ParamResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(path)?;

    writeln!(file, "# Sensitivity Analysis: {}", algo_name)?;
    writeln!(file)?;
    writeln!(
        file,
        "| Configuration | Dist. moyenne | Ecart-type | Temps moyen (ms) | Faisabilite |"
    )?;
    writeln!(file, "|---|---:|---:|---:|---:|")?;

    for res in results {
        writeln!(
            file,
            "| {} | {:.3} | {:.3} | {:.2} | {:.1}% |",
            res.config_label,
            res.mean_distance,
            res.std_distance,
            res.mean_elapsed_ms,
            res.feasible_rate * 100.0
        )?;
    }

    writeln!(file)?;
    writeln!(file, "## Key Observations")?;
    writeln!(file)?;

    // Find best
    if let Some(best) = results.iter().min_by(|a, b| {
        a.mean_distance
            .partial_cmp(&b.mean_distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        writeln!(
            file,
            "- **Best configuration**: {} (distance: {:.3})",
            best.config_label, best.mean_distance
        )?;
    }

    // Find fastest
    if let Some(fastest) = results.iter().min_by(|a, b| {
        a.mean_elapsed_ms
            .partial_cmp(&b.mean_elapsed_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        writeln!(
            file,
            "- **Fastest**: {} ({:.2} ms)",
            fastest.config_label, fastest.mean_elapsed_ms
        )?;
    }

    // Feasibility
    let avg_feas = mean(&results.iter().map(|r| r.feasible_rate).collect::<Vec<_>>());
    writeln!(file, "- **Average feasibility**: {:.1}%", avg_feas * 100.0)?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// PLOTTING HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn plot_sensitivity_by_param(
    algo_name: &str,
    results: &[ParamResult],
    _param_name: &str,
    _param_values: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    // Group all results into a single curve showing progression through configs
    let points: Vec<(f64, f64)> = results
        .iter()
        .enumerate()
        .map(|(idx, res)| (idx as f64, res.mean_distance))
        .collect();

    let series = vec![("Distance progression", points)];
    let path = format!(
        "plots/sensitivity_{}_progression.png",
        algo_name.to_lowercase()
    );

    plot_multi_series_line(
        &path,
        &format!("{}: Configuration Progression", algo_name),
        "Configuration index",
        "Mean distance",
        &series,
    )?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN TEST
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "Parameter sensitivity test: long-running, launch explicitly"]
fn test_parameter_sensitivity() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading VRP instances...");
    let files = get_vrp_files()?;

    let mut problems = Vec::new();
    for file in &files {
        let raw = fs::read_to_string(file)?;
        let parsed = InputData::parse_input(&raw);
        problems.push(Problem::new(parsed));
    }

    fs::create_dir_all("plots")?;

    // ─────────────────────────────────────────────────────────────────────────
    // Hill Climbing (baseline)
    // ─────────────────────────────────────────────────────────────────────────
    println!("Running Hill Climbing baseline...");
    let hc_results = run_hc_baseline(&problems)?;
    write_csv_report("plots/sensitivity_hc.csv", "HC", &hc_results)?;
    write_markdown_report("plots/sensitivity_hc.md", "Hill Climbing", &hc_results)?;

    println!(
        "  HC baseline: {} (distance={:.3})",
        hc_results[0].config_label, hc_results[0].mean_distance
    );

    // ─────────────────────────────────────────────────────────────────────────
    // Simulated Annealing
    // ─────────────────────────────────────────────────────────────────────────
    println!("Running Simulated Annealing parameter sweep...");
    let sa_results = run_sa_sweep(&problems)?;
    write_csv_report("plots/sensitivity_sa.csv", "SA", &sa_results)?;
    write_markdown_report(
        "plots/sensitivity_sa.md",
        "Simulated Annealing",
        &sa_results,
    )?;

    println!("  SA configs tested: {}", sa_results.len());
    if let Some(best) = sa_results.iter().min_by(|a, b| {
        a.mean_distance
            .partial_cmp(&b.mean_distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        println!(
            "  Best SA: {} (distance={:.3})",
            best.config_label, best.mean_distance
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Ant Colony Optimization
    // ─────────────────────────────────────────────────────────────────────────
    println!("Running Ant Colony Optimization parameter sweep...");
    let aco_results = run_aco_sweep(&problems)?;
    write_csv_report("plots/sensitivity_aco.csv", "ACO", &aco_results)?;
    write_markdown_report(
        "plots/sensitivity_aco.md",
        "Ant Colony Optimization",
        &aco_results,
    )?;

    println!("  ACO configs tested: {}", aco_results.len());
    if let Some(best) = aco_results.iter().min_by(|a, b| {
        a.mean_distance
            .partial_cmp(&b.mean_distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        println!(
            "  Best ACO: {} (distance={:.3})",
            best.config_label, best.mean_distance
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Summary
    // ─────────────────────────────────────────────────────────────────────────
    println!("\nParameter Sensitivity Analysis Complete!");
    println!("Reports generated:");
    println!("  - plots/sensitivity_sa.csv / .md");
    println!("  - plots/sensitivity_aco.csv / .md");
    println!("  - plots/sensitivity_hc.csv / .md");

    Ok(())
}
