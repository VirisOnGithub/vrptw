use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::{
    neighbouring::generate_neighbor,
    problem::{Float, Problem, Solution},
};

pub trait OptimizationAlgorithm {
    fn total_iterations(&self) -> usize;
    fn current_solution(&self) -> &Solution;
    fn step(&mut self, problem: &Problem, nb_steps: usize);
    fn is_finished(&self) -> bool;
}

#[derive(Debug)]
pub struct SAParams {
    pub t_initial: f64,
    pub t_final: f64,
    pub alpha: f64,
    /// Number of neighbour evaluations between temperature drops
    pub iter_per_temp: usize,
}

impl Default for SAParams {
    fn default() -> Self {
        Self {
            t_initial: 500.0,
            t_final: 0.1,
            alpha: 0.995,
            iter_per_temp: 150,
        }
    }
}

/// We need to store :
/// current solution => For the ui // for the calculations
/// best solution => For the final result
/// current/best costs : for optimisation -- avoid recalculate each time
pub struct SimulatedAnnealing {
    pub current_solution: Solution,
    pub current_cost: Float,
    pub best_solution: Solution,
    pub best_cost: Float,
    params: SAParams,
    temperature: Float,
    total_iterations: usize,
    rng: StdRng,
    nb_accepted: usize,
    iter_in_temp: usize,
}

impl SimulatedAnnealing {
    /// We have to send a StdRng because Rng is not Send => cannot send it thread-safely
    pub(crate) fn new(problem: &Problem, solution: &Solution, params: SAParams) -> Self {
        let initial_solution = solution.clone();
        let initial_cost = initial_solution.total_distance(problem);
        let initial_temp = params.t_initial;
        Self {
            current_solution: initial_solution.clone(),
            current_cost: initial_cost,
            best_solution: initial_solution,
            best_cost: initial_cost,
            params,
            temperature: initial_temp,
            total_iterations: 0,
            rng: StdRng::from_entropy(),
            nb_accepted: 0,
            iter_in_temp: 0,
        }
    }
}

impl OptimizationAlgorithm for SimulatedAnnealing {
    fn total_iterations(&self) -> usize {
        self.total_iterations
    }

    fn step(&mut self, problem: &Problem, steps: usize) {
        for _ in 0..steps {
            if self.is_finished() {
                return;
            }

            let candidate = generate_neighbor(&self.current_solution, problem, &mut self.rng);
            let candidate_cost = candidate.total_distance(problem);
            let delta = candidate_cost - self.current_cost;

            if delta < 0.0 || self.rng.gen_range(0.0f64..1.0) < (-delta / self.temperature).exp() {
                self.current_solution = candidate;
                self.current_cost = candidate_cost;
                self.nb_accepted += 1;

                if self.current_cost < self.best_cost {
                    self.best_solution = self.current_solution.clone();
                    self.best_cost = self.current_cost;
                }
            }

            self.total_iterations += 1;
            self.iter_in_temp += 1;

            if self.iter_in_temp >= self.params.iter_per_temp {
                self.temperature *= self.params.alpha;
                self.iter_in_temp = 0;
            }
        }
    }

    fn is_finished(&self) -> bool {
        self.temperature <= self.params.t_final
    }

    fn current_solution(&self) -> &Solution {
        &self.current_solution
    }
}
