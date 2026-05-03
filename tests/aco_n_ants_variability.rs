mod aco_sensitivity_common;

use vrptw_code::optimizers::aco::AcoParams;

#[test]
#[ignore = "ACO n_ants variability: long-running, run explicitly"]
fn test_aco_n_ants_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![5usize, 10, 20, 40, 60, 80, 100];
    aco_sensitivity_common::run_aco_sweep(
        "n_ants",
        "aco_n_ants_variability",
        "ACO: number of ants sensitivity",
        "n_ants",
        &values,
        |n_ants| AcoParams {
            n_ants,
            ..AcoParams::default()
        },
        |n_ants| n_ants as f64,
    )
}
