use std::{fs, io::Write, time::Instant};

use vrptw_code::{
    graph_utils::plot_multi_series_line,
    optimizers::aco::{self, AcoParams},
    problem::Solution,
};

mod aco_sensitivity_common;

#[test]
#[ignore = "ACO use_two_opt variability: long-running, run explicitly"]
fn test_aco_use_two_opt_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    let problems = aco_sensitivity_common::load_problems()?;
    fs::create_dir_all("plots")?;

    let values = [(false, 0.0_f64), (true, 1.0_f64)];

    for (instance_name, problem) in problems {
        let mut csv = fs::File::create(format!("plots/aco_use_two_opt_variability_{}.csv", instance_name))?;
        writeln!(csv, "use_two_opt,mean_distance,std_distance,feasible_rate,mean_elapsed_ms")?;

        let mut series = Vec::new();

        for (flag, x) in values {
            let mut distances = Vec::new();
            let mut times_ms = Vec::new();
            let mut feasible = 0usize;

            for _ in 0..aco_sensitivity_common::RUNS_PER_CONFIG {
                let solution = Solution::simplest(&problem);
                let params = AcoParams {
                    use_two_opt: flag,
                    ..AcoParams::default()
                };
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

            let mean_distance = aco_sensitivity_common::mean(&distances);
            let std_distance = aco_sensitivity_common::std_dev(&distances);
            let mean_elapsed_ms = aco_sensitivity_common::mean(&times_ms);
            let feasible_rate = feasible as f64 / aco_sensitivity_common::RUNS_PER_CONFIG as f64;

            writeln!(
                csv,
                "{},{:.6},{:.6},{:.3},{:.3}",
                flag, mean_distance, std_distance, feasible_rate, mean_elapsed_ms
            )?;

            series.push((x, mean_distance));
        }

        plot_multi_series_line(
            format!("plots/aco_use_two_opt_variability_{}.png", instance_name),
            &format!("ACO: 2-opt sensitivity — {}", instance_name),
            "use_two_opt (0=false, 1=true)",
            "mean distance",
            &[("mean_distance", series)],
        )?;

        println!("  Saved plots/aco_use_two_opt_variability_{}.png and .csv", instance_name);
    }

    Ok(())
}
