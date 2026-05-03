mod aco_sensitivity_common;

use vrptw_code::optimizers::aco::AcoParams;

#[test]
#[ignore = "ACO max_iterations variability: long-running, run explicitly"]
fn test_aco_max_iterations_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![50usize, 100, 200, 400, 800, 1200, 1600];
    aco_sensitivity_common::run_aco_sweep(
        "max_iterations",
        "aco_max_iterations_variability",
        "ACO: iteration budget sensitivity",
        "max_iterations",
        &values,
        |max_iterations| AcoParams { max_iterations, ..AcoParams::default() },
        |max_iterations| max_iterations as f64,
    )
}
