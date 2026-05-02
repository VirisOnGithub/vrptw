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
            Self::SA => "Simulated Annealing",
            Self::ACO => "Ant Colony Optimization",
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

fn run_algo_once(problem: &Problem, algo: AlgoKind) -> RunOutcome {
    let initial_solution = Solution::simplest(problem);
    let mut algorithm: Box<dyn OptimizationAlgorithm + Send + Sync> = match algo {
        AlgoKind::SA => {
            let params = SAParams::default();
            simulated_annealing::build_algorithm(problem, &initial_solution, &params, true)
        }
        AlgoKind::ACO => {
            let params = AcoParams::default();
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
    writeln!(file, "# Benchmark VRPTW")?;
    writeln!(
        file,
        "- Instances: {}\n- Runs par instance et par algorithme: {}\n",
        rows.len() / AlgoKind::all().len(),
        RUNS_PER_INSTANCE
    )?;
    writeln!(
        file,
        "| Instance | Algorithme | Dist. moyenne | Ecart-type | Dist. min | Temps moyen (ms) | Faisabilite | Nb routes moyen | Gap normalise (%) |"
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
fn test_comparaison() -> Result<(), Box<dyn std::error::Error>> {
    let files = get_vrp_files()?;

    let mut table: Vec<(String, AlgoStats)> = Vec::new();
    let mut sa_mean_distance_curve: Vec<f64> = Vec::new();
    let mut aco_mean_distance_curve: Vec<f64> = Vec::new();
    let mut hc_mean_distance_curve: Vec<f64> = Vec::new();

    let mut sa_mean_time_curve: Vec<f64> = Vec::new();
    let mut aco_mean_time_curve: Vec<f64> = Vec::new();
    let mut hc_mean_time_curve: Vec<f64> = Vec::new();

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
    write_csv_report("plots/comp_stats.csv", &table)?;
    write_markdown_report("plots/comp_stats.md", &table)?;

    plot_series_y_vs_index(
        "plots/comp_distance_mean.png",
        "Comparaison des algorithmes (distance moyenne)",
        "Distance moyenne",
        &[
            ("Simulated Annealing", sa_mean_distance_curve),
            ("Ant Colony Optimization", aco_mean_distance_curve),
            ("Hill Climbing", hc_mean_distance_curve),
        ],
    )?;

    plot_series_y_vs_index(
        "plots/comp_time_mean_ms.png",
        "Comparaison des algorithmes (temps moyen)",
        "Temps moyen (ms)",
        &[
            ("Simulated Annealing", sa_mean_time_curve),
            ("Ant Colony Optimization", aco_mean_time_curve),
            ("Hill Climbing", hc_mean_time_curve),
        ],
    )?;

    println!("Benchmark termine. Rapports generes:");
    println!("- plots/comp_stats.csv");
    println!("- plots/comp_stats.md");
    println!("- plots/comp_distance_mean.png");
    println!("- plots/comp_time_mean_ms.png");

    Ok(())
}
