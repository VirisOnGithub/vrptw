mod aco_sensitivity_common;

use vrptw_code::optimizers::aco::AcoParams;

#[test]
#[ignore = "ACO beta variability: long-running, run explicitly"]
fn test_aco_beta_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![0.1_f64, 0.5, 1.0, 2.0, 2.5, 3.5, 5.0];
    aco_sensitivity_common::run_aco_sweep(
        "beta",
        "aco_beta_variability",
        "ACO: heuristic weight sensitivity",
        "beta",
        &values,
        |beta| AcoParams { beta, ..AcoParams::default() },
        |beta| beta,
    )
}
