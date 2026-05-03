mod aco_sensitivity_common;

use vrptw_code::optimizers::aco::AcoParams;

#[test]
#[ignore = "ACO k_candidates variability: long-running, run explicitly"]
fn test_aco_k_candidates_variability_per_instance() -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![3usize, 5, 8, 10, 12, 15, 20];
    aco_sensitivity_common::run_aco_sweep(
        "k_candidates",
        "aco_k_candidates_variability",
        "ACO: candidate list size sensitivity",
        "k_candidates",
        &values,
        |k_candidates| AcoParams { k_candidates, ..AcoParams::default() },
        |k_candidates| k_candidates as f64,
    )
}
