mod aco_sensitivity_common;

use vrptw_code::optimizers::aco::AcoParams;

#[test]
#[ignore = "ACO alpha variability: long-running, run explicitly"]
fn test_aco_alpha_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![0.1_f64, 0.5, 1.0, 1.5, 2.0, 3.0, 5.0];
    aco_sensitivity_common::run_aco_sweep(
        "alpha",
        "aco_alpha_variability",
        "ACO: pheromone weight sensitivity",
        "alpha",
        &values,
        |alpha| AcoParams { alpha, ..AcoParams::default() },
        |alpha| alpha,
    )
}
