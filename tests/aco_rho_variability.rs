mod aco_sensitivity_common;

use vrptw_code::optimizers::aco::AcoParams;

#[test]
#[ignore = "ACO rho variability: long-running, run explicitly"]
fn test_aco_rho_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![0.01_f64, 0.05, 0.1, 0.2, 0.3, 0.4, 0.5];
    aco_sensitivity_common::run_aco_sweep(
        "rho",
        "aco_rho_variability",
        "ACO: evaporation rate sensitivity",
        "rho",
        &values,
        |rho| AcoParams { rho, ..AcoParams::default() },
        |rho| rho,
    )
}
