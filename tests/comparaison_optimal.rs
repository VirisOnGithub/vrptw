use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use vrptw_code::{
    graph_utils::plot_series_y_vs_index,
    optimizers::{
        OptimizationAlgorithm, aco, aco::AcoParams, hill_climbing, hill_climbing::HCParams,
        simulated_annealing, simulated_annealing::SAParams,
    },
    parser::InputData,
    problem::{Problem, Solution},
};

const RUNS_PER_INSTANCE: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum AlgoKind {
    SA,
    ACO,
    HC,
}

impl AlgoKind {
    fn label(self) -> &'static str {
        match self {
            Self::SA => "Simulated Annealing (Optimal)",
            Self::ACO => "Ant Colony Optimization (Optimal)",
            Self::HC => "Hill Climbing",
        }
    }

    fn all() -> [AlgoKind; 3] {
        [Self::SA, Self::ACO, Self::HC]
    }
}

#[derive(Debug, Clone)]
struct RunOutcome {
    distance: f64,
    elapsed_ms: f64,
    feasible: bool,
    routes: usize,
}

#[derive(Debug, Clone)]
struct AlgoStats {
    algo: AlgoKind,
    best_distance: f64,
    mean_distance: f64,
    std_distance: f64,
    mean_elapsed_ms: f64,
    feasible_rate: f64,
    mean_routes: f64,
    normalized_gap_pct: f64,
}

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
    Ok(files)
}

/// Extract optimal parameters from rapport
fn get_optimal_sa_params() -> SAParams {
    SAParams {
        // D'après le rapport:
        // - alpha: plus élevé = mieux (mais courbe quadratique/cubique, donc on prend 0.9995 qui est très élevé)
        // - t_final: entre 1 et 10 degrés optimaux (on prend 5.0 au milieu)
        // - t_initial: le rapport dit pas d'influence claire, on garde la valeur par défaut
        // - iter_per_temp: entre 50 et 100 itérations est optimal (on prend 75)
        t_initial: 500.0,
        t_final: 5.0,      // Optimal entre 1 et 10 selon rapport
        alpha: 0.9995,     // Plus élevé = mieux selon rapport
        iter_per_temp: 75, // Optimal entre 50 et 100 selon rapport
    }
}

/// Extract optimal parameters from rapport
fn get_optimal_aco_params() -> AcoParams {
    AcoParams {
        // D'après le rapport:
        // - alpha: entre 1.5 et 2 est optimal
        // - beta: difficile à analyser, dépend du problème (on garde la valeur par défaut)
        // - rho: inférieur à 0.05 est optimal
        // - q: entre 300 et 500 unités optimal
        // - n_ants: plus augmente = mieux
        // - k_candidates: aucune importance selon rapport
        // - max_iterations: efficacité décroissante, on augmente un peu
        n_ants: 50,          // Plus de fourmis = mieux
        alpha: 1.75,         // Optimal entre 1.5 et 2
        beta: 2.5,           // Pas de recommandation spécifique, garde défaut
        rho: 0.03,           // Optimal < 0.05
        q: 400.0,            // Optimal entre 300 et 500
        tau_min: 0.01,       // Garde défaut
        tau_max: 10.0,       // Garde défaut
        max_iterations: 750, // Un peu plus que défaut
        k_candidates: 10,    // Aucune importance selon rapport
        use_two_opt: true,   // Garde défaut
    }
}

fn run_algo_once(problem: &Problem, algo: AlgoKind) -> RunOutcome {
    let initial_solution = Solution::simplest(problem);
    let mut algorithm: Box<dyn OptimizationAlgorithm + Send + Sync> = match algo {
        AlgoKind::SA => {
            let params = get_optimal_sa_params();
            simulated_annealing::build_algorithm(problem, &initial_solution, &params, true)
        }
        AlgoKind::ACO => {
            let params = get_optimal_aco_params();
            aco::build_algorithm(problem, &initial_solution, &params, true)
        }
        AlgoKind::HC => {
            let params = HCParams::default();
            hill_climbing::build_algorithm(problem, &initial_solution, &params, true)
        }
    };

    let started = Instant::now();
    algorithm.step(problem, usize::MAX);
    let elapsed = started.elapsed().as_secs_f64() * 1000.0;

    let solution = algorithm.current_solution();
    RunOutcome {
        distance: solution.total_distance(problem),
        elapsed_ms: elapsed,
        feasible: solution.is_feasible(problem),
        routes: solution.routes.len(),
    }
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn std_dev(values: &[f64], avg: f64) -> f64 {
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

fn summarize_runs(algo: AlgoKind, runs: &[RunOutcome]) -> AlgoStats {
    let distances: Vec<f64> = runs.iter().map(|r| r.distance).collect();
    let times: Vec<f64> = runs.iter().map(|r| r.elapsed_ms).collect();
    let routes: Vec<f64> = runs.iter().map(|r| r.routes as f64).collect();

    let mean_distance = mean(&distances);
    let std_distance = std_dev(&distances, mean_distance);

    AlgoStats {
        algo,
        best_distance: distances.iter().cloned().fold(f64::INFINITY, f64::min),
        mean_distance,
        std_distance,
        mean_elapsed_ms: mean(&times),
        feasible_rate: runs.iter().filter(|r| r.feasible).count() as f64 / runs.len() as f64,
        mean_routes: mean(&routes),
        normalized_gap_pct: 0.0,
    }
}

fn write_csv_report(
    path: &str,
    rows: &[(String, AlgoStats)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(path)?;
    writeln!(
        file,
        "instance,algorithm,best_distance,mean_distance,std_distance,mean_elapsed_ms,feasible_rate,mean_routes,normalized_gap_pct"
    )?;

    for (instance, st) in rows {
        writeln!(
            file,
            "{},{},{:.6},{:.6},{:.6},{:.3},{:.3},{:.3},{:.3}",
            instance,
            st.algo.label(),
            st.best_distance,
            st.mean_distance,
            st.std_distance,
            st.mean_elapsed_ms,
            st.feasible_rate,
            st.mean_routes,
            st.normalized_gap_pct
        )?;
    }

    Ok(())
}

fn write_markdown_report(
    path: &str,
    rows: &[(String, AlgoStats)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(path)?;
    writeln!(file, "# Benchmark VRPTW - Paramètres Optimaux")?;
    writeln!(
        file,
        "Benchmark utilisant les paramètres optimaux trouvés dans le rapport.\n"
    )?;
    writeln!(file, "## Paramètres utilisés\n")?;
    writeln!(file, "### Simulated Annealing\n")?;
    writeln!(file, "- t_initial: 500.0")?;
    writeln!(file, "- t_final: 5.0 (optimal entre 1-10 selon rapport)")?;
    writeln!(file, "- alpha: 0.9995 (plus élevé = mieux)")?;
    writeln!(file, "- iter_per_temp: 75 (optimal entre 50-100)\n")?;

    writeln!(file, "### Ant Colony Optimization\n")?;
    writeln!(file, "- n_ants: 50 (plus augmente = mieux)")?;
    writeln!(file, "- alpha: 1.75 (optimal entre 1.5-2)")?;
    writeln!(file, "- beta: 2.5 (pas de recommandation spécifique)")?;
    writeln!(file, "- rho: 0.03 (optimal < 0.05)")?;
    writeln!(file, "- q: 400.0 (optimal entre 300-500)")?;
    writeln!(file, "- max_iterations: 750\n")?;

    writeln!(file, "## Résultats\n")?;
    writeln!(
        file,
        "- Instances: {}\n- Runs par instance et par algorithme: {}\n",
        rows.len() / AlgoKind::all().len(),
        RUNS_PER_INSTANCE
    )?;
    writeln!(
        file,
        "| Instance | Algorithme | Dist. moyenne | Écart-type | Dist. min | Temps moyen (ms) | Faisabilité | Nb routes moyen | Gap normalisé (%) |"
    )?;
    writeln!(file, "|---|---:|---:|---:|---:|---:|---:|---:|---:|")?;

    for (instance, st) in rows {
        writeln!(
            file,
            "| {} | {} | {:.3} | {:.3} | {:.3} | {:.2} | {:.1}% | {:.2} | {:.2} |",
            instance,
            st.algo.label(),
            st.mean_distance,
            st.std_distance,
            st.best_distance,
            st.mean_elapsed_ms,
            st.feasible_rate * 100.0,
            st.mean_routes,
            st.normalized_gap_pct
        )?;
    }

    Ok(())
}

#[test]
#[ignore = "Benchmark long: a lancer explicitement"]
fn test_comparaison_optimal() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== BENCHMARK AVEC PARAMÈTRES OPTIMAUX ===\n");

    let files = get_vrp_files()?;

    let mut table: Vec<(String, AlgoStats)> = Vec::new();
    let mut sa_mean_distance_curve: Vec<f64> = Vec::new();
    let mut aco_mean_distance_curve: Vec<f64> = Vec::new();
    let mut hc_mean_distance_curve: Vec<f64> = Vec::new();

    let mut sa_mean_time_curve: Vec<f64> = Vec::new();
    let mut aco_mean_time_curve: Vec<f64> = Vec::new();
    let mut hc_mean_time_curve: Vec<f64> = Vec::new();

    let mut sa_distances_per_run: Vec<Vec<f64>> = Vec::new();
    let mut aco_distances_per_run: Vec<Vec<f64>> = Vec::new();
    let mut hc_distances_per_run: Vec<Vec<f64>> = Vec::new();

    let mut sa_times_per_run: Vec<Vec<f64>> = Vec::new();
    let mut aco_times_per_run: Vec<Vec<f64>> = Vec::new();
    let mut hc_times_per_run: Vec<Vec<f64>> = Vec::new();

    for (file_idx, file) in files.iter().enumerate() {
        let raw = fs::read_to_string(file)?;
        let parsed = InputData::parse_input(&raw);
        let problem = Problem::new(parsed);
        let instance_name = file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        println!(
            "Processing {} ({}/{})",
            instance_name,
            file_idx + 1,
            files.len()
        );

        let mut current_stats: Vec<AlgoStats> = Vec::new();
        for algo in AlgoKind::all() {
            let mut runs: Vec<RunOutcome> = Vec::with_capacity(RUNS_PER_INSTANCE);
            for _ in 0..RUNS_PER_INSTANCE {
                runs.push(run_algo_once(&problem, algo));
            }

            // Collect individual run data for later visualization
            let distances: Vec<f64> = runs.iter().map(|r| r.distance).collect();
            let times: Vec<f64> = runs.iter().map(|r| r.elapsed_ms).collect();

            match algo {
                AlgoKind::SA => {
                    sa_distances_per_run.push(distances);
                    sa_times_per_run.push(times);
                }
                AlgoKind::ACO => {
                    aco_distances_per_run.push(distances);
                    aco_times_per_run.push(times);
                }
                AlgoKind::HC => {
                    hc_distances_per_run.push(distances);
                    hc_times_per_run.push(times);
                }
            }

            current_stats.push(summarize_runs(algo, &runs));
        }

        let best_mean = current_stats
            .iter()
            .map(|s| s.mean_distance)
            .fold(f64::INFINITY, f64::min)
            .max(1e-12);

        for st in &mut current_stats {
            st.normalized_gap_pct = (st.mean_distance - best_mean) / best_mean * 100.0;
        }

        for st in current_stats {
            match st.algo {
                AlgoKind::SA => {
                    sa_mean_distance_curve.push(st.mean_distance);
                    sa_mean_time_curve.push(st.mean_elapsed_ms);
                }
                AlgoKind::ACO => {
                    aco_mean_distance_curve.push(st.mean_distance);
                    aco_mean_time_curve.push(st.mean_elapsed_ms);
                }
                AlgoKind::HC => {
                    hc_mean_distance_curve.push(st.mean_distance);
                    hc_mean_time_curve.push(st.mean_elapsed_ms);
                }
            }
            table.push((instance_name.clone(), st));
        }
    }

    fs::create_dir_all("plots")?;
    write_csv_report("plots/comp_optimal_stats.csv", &table)?;
    write_markdown_report("plots/comp_optimal_stats.md", &table)?;

    // Distance comparison
    plot_series_y_vs_index(
        "plots/comp_optimal_distance_mean.png",
        "Comparaison des algorithmes (distance moyenne) - Paramètres optimaux",
        "Distance moyenne",
        &[
            ("Simulated Annealing", sa_mean_distance_curve.clone()),
            ("Ant Colony Optimization", aco_mean_distance_curve.clone()),
            ("Hill Climbing", hc_mean_distance_curve.clone()),
        ],
    )?;

    // Time comparison
    plot_series_y_vs_index(
        "plots/comp_optimal_time_mean_ms.png",
        "Comparaison des algorithmes (temps moyen) - Paramètres optimaux",
        "Temps moyen (ms)",
        &[
            ("Simulated Annealing", sa_mean_time_curve.clone()),
            ("Ant Colony Optimization", aco_mean_time_curve.clone()),
            ("Hill Climbing", hc_mean_time_curve.clone()),
        ],
    )?;

    // Generate detailed comparison CSV with individual runs
    let mut detailed_csv = fs::File::create("plots/comp_optimal_detailed.csv")?;
    writeln!(
        detailed_csv,
        "instance_index,algorithm,run_number,distance,time_ms"
    )?;

    for (idx, distances) in sa_distances_per_run.iter().enumerate() {
        for (run, &dist) in distances.iter().enumerate() {
            let time = sa_times_per_run[idx][run];
            writeln!(detailed_csv, "{},SA,{},{:.6},{:.3}", idx, run, dist, time)?;
        }
    }

    for (idx, distances) in aco_distances_per_run.iter().enumerate() {
        for (run, &dist) in distances.iter().enumerate() {
            let time = aco_times_per_run[idx][run];
            writeln!(detailed_csv, "{},ACO,{},{:.6},{:.3}", idx, run, dist, time)?;
        }
    }

    for (idx, distances) in hc_distances_per_run.iter().enumerate() {
        for (run, &dist) in distances.iter().enumerate() {
            let time = hc_times_per_run[idx][run];
            writeln!(detailed_csv, "{},HC,{},{:.6},{:.3}", idx, run, dist, time)?;
        }
    }

    println!("\n=== BENCHMARK TERMINÉ ===");
    println!("Rapports générés:");
    println!("  - plots/comp_optimal_stats.csv");
    println!("  - plots/comp_optimal_stats.md");
    println!("  - plots/comp_optimal_detailed.csv");
    println!("  - plots/comp_optimal_distance_mean.png");
    println!("  - plots/comp_optimal_time_mean_ms.png");

    Ok(())
}
