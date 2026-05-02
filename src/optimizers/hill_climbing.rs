use std::any::Any;

use crate::optimizers::{OptimizationAlgorithm, OptimizerDescriptor};
use rand::{SeedableRng, rngs::StdRng};

use crate::{
    neighbouring::generate_neighbor,
    problem::{Float, Problem, Solution},
};

#[derive(Clone, Debug)]
pub struct HCParams {}

impl Default for HCParams {
    fn default() -> Self {
        Self {}
    }
}

/// We need to store :
/// current solution => For the ui // for the calculations
/// best solution => For the final result
/// current/best costs : for optimisation -- avoid recalculate each time
pub struct HillClimbing {
    pub current_solution: Solution,
    pub current_cost: Float,
    pub best_solution: Solution,
    pub best_cost: Float,
    _params: HCParams,
    total_iterations: usize,
    rng: StdRng,
    nb_accepted: usize,
    time_into_account: bool,
    no_better_solution_iterations: usize,
}

impl HillClimbing {
    /// We have to send a StdRng because Rng is not Send => cannot send it thread-safely
    pub fn new(
        problem: &Problem,
        solution: &Solution,
        params: HCParams,
        time_into_account: bool,
    ) -> Self {
        let initial_solution = solution.clone();
        let initial_cost = initial_solution.total_distance(problem);
        Self {
            current_solution: initial_solution.clone(),
            current_cost: initial_cost,
            best_solution: initial_solution,
            best_cost: initial_cost,
            _params: params,
            total_iterations: 0,
            rng: StdRng::from_entropy(),
            nb_accepted: 0,
            time_into_account,
            no_better_solution_iterations: 0,
        }
    }
}

impl OptimizationAlgorithm for HillClimbing {
    fn total_iterations(&self) -> usize {
        self.total_iterations
    }

    fn step(&mut self, problem: &Problem, steps: usize) {
        for _ in 0..steps {
            if self.is_finished() {
                return;
            }

            let candidate = generate_neighbor(
                &self.current_solution,
                problem,
                &mut self.rng,
                self.time_into_account,
            );
            let candidate_cost = candidate.total_distance(problem);
            let delta = candidate_cost - self.current_cost;

            if delta < 0.0 {
                self.current_solution = candidate;
                self.current_cost = candidate_cost;
                self.nb_accepted += 1;
                self.no_better_solution_iterations = 0;

                if self.current_cost < self.best_cost {
                    self.best_solution = self.current_solution.clone();
                    self.best_cost = self.current_cost;
                }
            } else {
                self.no_better_solution_iterations += 1;
            }

            self.total_iterations += 1;
        }
    }

    fn is_finished(&self) -> bool {
        self.no_better_solution_iterations >= 10000
    }

    fn current_solution(&self) -> &Solution {
        &self.current_solution
    }
}

pub fn create_default_params() -> Box<dyn Any + Send + Sync> {
    Box::new(HCParams::default())
}

fn draw_params_ui(_params: &mut dyn Any, _ui: &mut egui::Ui) {
    // No arguments, so this function does nothing
}

pub fn build_algorithm(
    problem: &Problem,
    solution: &Solution,
    params: &dyn Any,
    time_into_account: bool,
) -> Box<dyn OptimizationAlgorithm + Send + Sync> {
    let params = params
        .downcast_ref::<HCParams>()
        .expect("Invalid HC params type in optimizer registry");

    Box::new(HillClimbing::new(
        problem,
        solution,
        params.clone(),
        time_into_account,
    ))
}

inventory::submit! {
    OptimizerDescriptor {
        id: "hill_climbing",
        label: "Méthode de descente",
        create_default_params,
        draw_params_ui,
        build_algorithm,
    }
}
