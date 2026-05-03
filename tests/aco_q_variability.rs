mod aco_sensitivity_common;

use vrptw_code::optimizers::aco::AcoParams;

#[test]
#[ignore = "ACO q variability: long-running, run explicitly"]
fn test_aco_q_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![1.0_f64, 10.0, 50.0, 100.0, 250.0, 500.0, 1000.0];
    aco_sensitivity_common::run_aco_sweep(
        "q",
        "aco_q_variability",
        "ACO: pheromone deposit sensitivity",
        "q",
        &values,
        |q| AcoParams { q, ..AcoParams::default() },
        |q| q,
    )
}
