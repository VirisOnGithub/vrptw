mod aco_sensitivity_common;

use vrptw_code::optimizers::aco::AcoParams;

#[test]
#[ignore = "ACO tau_max variability: long-running, run explicitly"]
fn test_aco_tau_max_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![1.0_f64, 5.0, 10.0, 20.0, 50.0, 100.0];
    aco_sensitivity_common::run_aco_sweep(
        "tau_max",
        "aco_tau_max_variability",
        "ACO: maximum pheromone sensitivity",
        "tau_max",
        &values,
        |tau_max| AcoParams { tau_max, ..AcoParams::default() },
        |tau_max| tau_max,
    )
}
