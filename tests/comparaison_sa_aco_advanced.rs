use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use vrptw_code::{
    graph_utils::plot_series_y_vs_index,
    optimizers::{
        OptimizationAlgorithm, aco, aco::AcoParams, simulated_annealing,
        simulated_annealing::SAParams,
    },
    parser::InputData,
    problem::{Problem, Solution},
};

const RUNS_PER_INSTANCE: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum AlgoKind {
    SAAdvanced,
    ACOAdvanced,
}

impl AlgoKind {
    fn short(self) -> &'static str {
        match self {
            Self::SAAdvanced => "SA",
            Self::ACOAdvanced => "ACO",
        }
    }

    fn all() -> [AlgoKind; 2] {
        [Self::SAAdvanced, Self::ACOAdvanced]
    }
}

#[derive(Debug, Clone)]
struct RunOutcome {
    distance: f64,
    elapsed_ms: f64,
    feasible: bool,
}

#[derive(Debug, Clone)]
struct AlgoStats {
    algo: AlgoKind,
    mean_distance: f64,
    std_distance: f64,
    mean_elapsed_ms: f64,
    std_elapsed_ms: f64,
    feasible_rate: f64,
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

fn get_advanced_sa_params() -> SAParams {
    // Parametres "efficaces" selon le rapport:
    // alpha eleve, t_final bas, iter_per_temp dans [50, 100] pour un bon compromis qualite/temps.
    SAParams {
        t_initial: 500.0,
        t_final: 2.0,
        alpha: 0.999,
        iter_per_temp: 75,
    }
}

fn get_advanced_aco_params() -> AcoParams {
    // Parametres "efficaces" selon le rapport:
    // alpha dans [1.5, 2], rho < 0.05, q dans [300, 500],
    // et reglages moderes de population/iterations pour limiter le temps.
    AcoParams {
        n_ants: 40,
        alpha: 1.8,
        beta: 2.5,
        rho: 0.03,
        q: 400.0,
        tau_min: 0.01,
        tau_max: 10.0,
        max_iterations: 600,
        k_candidates: 10,
        use_two_opt: true,
    }
}

fn run_algo_once(problem: &Problem, algo: AlgoKind) -> RunOutcome {
    let initial_solution = Solution::simplest(problem);
    let mut algorithm: Box<dyn OptimizationAlgorithm + Send + Sync> = match algo {
        AlgoKind::SAAdvanced => {
            let params = get_advanced_sa_params();
            simulated_annealing::build_algorithm(problem, &initial_solution, &params, true)
        }
        AlgoKind::ACOAdvanced => {
            let params = get_advanced_aco_params();
            aco::build_algorithm(problem, &initial_solution, &params, true)
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

    let mean_distance = mean(&distances);
    let mean_elapsed_ms = mean(&times);

    AlgoStats {
        algo,
        mean_distance,
        std_distance: std_dev(&distances, mean_distance),
        mean_elapsed_ms,
        std_elapsed_ms: std_dev(&times, mean_elapsed_ms),
        feasible_rate: runs.iter().filter(|r| r.feasible).count() as f64 / runs.len() as f64,
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
        "instance,algorithm,mean_distance,std_distance,mean_elapsed_ms,std_elapsed_ms,feasible_rate,normalized_gap_pct"
    )?;

    for (instance, st) in rows {
        writeln!(
            file,
            "{},{},{:.6},{:.6},{:.3},{:.3},{:.3},{:.3}",
            instance,
            st.algo.short(),
            st.mean_distance,
            st.std_distance,
            st.mean_elapsed_ms,
            st.std_elapsed_ms,
            st.feasible_rate,
            st.normalized_gap_pct
        )?;
    }

    Ok(())
}

#[test]
#[ignore = "Benchmark long: a lancer explicitement"]
fn test_comparaison_sa_aco_advanced() -> Result<(), Box<dyn std::error::Error>> {
    let files = get_vrp_files()?;

    let mut table: Vec<(String, AlgoStats)> = Vec::new();
    let mut sa_distance_curve: Vec<f64> = Vec::new();
    let mut aco_distance_curve: Vec<f64> = Vec::new();
    let mut sa_time_curve: Vec<f64> = Vec::new();
    let mut aco_time_curve: Vec<f64> = Vec::new();

    for file in &files {
        let raw = fs::read_to_string(file)?;
        let parsed = InputData::parse_input(&raw);
        let problem = Problem::new(parsed);
        let instance_name = file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut current_stats: Vec<AlgoStats> = Vec::new();
        for algo in AlgoKind::all() {
            let mut runs: Vec<RunOutcome> = Vec::with_capacity(RUNS_PER_INSTANCE);
            for _ in 0..RUNS_PER_INSTANCE {
                runs.push(run_algo_once(&problem, algo));
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
                AlgoKind::SAAdvanced => {
                    sa_distance_curve.push(st.mean_distance);
                    sa_time_curve.push(st.mean_elapsed_ms);
                }
                AlgoKind::ACOAdvanced => {
                    aco_distance_curve.push(st.mean_distance);
                    aco_time_curve.push(st.mean_elapsed_ms);
                }
            }
            table.push((instance_name.clone(), st));
        }
    }

    fs::create_dir_all("plots")?;
    write_csv_report("plots/comp_sa_aco_advanced_stats.csv", &table)?;

    plot_series_y_vs_index(
        "plots/comp_sa_aco_advanced_distance_mean.png",
        "SA vs ACO avances (distance moyenne)",
        "Distance moyenne",
        &[
            ("SA avance", sa_distance_curve),
            ("ACO avance", aco_distance_curve),
        ],
    )?;

    plot_series_y_vs_index(
        "plots/comp_sa_aco_advanced_time_mean_ms.png",
        "SA vs ACO avances (temps moyen)",
        "Temps moyen (ms)",
        &[("SA avance", sa_time_curve), ("ACO avance", aco_time_curve)],
    )?;

    println!("Comparaison SA/ACO avances terminee.");
    println!("- plots/comp_sa_aco_advanced_stats.csv");
    println!("- plots/comp_sa_aco_advanced_distance_mean.png");
    println!("- plots/comp_sa_aco_advanced_time_mean_ms.png");

    Ok(())
}
