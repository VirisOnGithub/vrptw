mod aco_sensitivity_common;

use vrptw_code::optimizers::aco::AcoParams;

#[test]
#[ignore = "ACO tau_min variability: long-running, run explicitly"]
fn test_aco_tau_min_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![0.001_f64, 0.01, 0.05, 0.1, 0.2];
    aco_sensitivity_common::run_aco_sweep(
        "tau_min",
        "aco_tau_min_variability",
        "ACO: minimum pheromone sensitivity",
        "tau_min",
        &values,
        |tau_min| AcoParams { tau_min, ..AcoParams::default() },
        |tau_min| tau_min,
    )
}
