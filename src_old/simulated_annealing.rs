use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::solution::{Problem, Solution};

// ─── Parameters ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SAParams {
    pub t_initial: f64,
    pub t_final: f64,
    pub alpha: f64,
    /// Number of neighbour evaluations between temperature drops
    pub iter_per_temp: usize,
}

impl Default for SAParams {
    fn default() -> Self {
        SAParams {
            t_initial: 500.0,
            t_final: 0.1,
            alpha: 0.995,
            iter_per_temp: 150,
        }
    }
}

// ─── Step-based SA (used by the GUI) ─────────────────────────────────────────

pub struct SimulatedAnnealing {
    pub current: Solution,
    pub current_cost: f64,
    pub best: Solution,
    pub best_cost: f64,
    pub temperature: f64,
    pub total_iterations: usize,
    pub accepted: usize,
    params: SAParams,
    iter_in_temp: usize,
    rng: ThreadRng,
}

impl SimulatedAnnealing {
    pub fn new(problem: &Problem, params: SAParams) -> Self {
        let initial = Solution::random(problem);
        let cost = initial.total_distance(problem);
        let temp = params.t_initial;
        SimulatedAnnealing {
            current: initial.clone(),
            current_cost: cost,
            best: initial,
            best_cost: cost,
            temperature: temp,
            total_iterations: 0,
            accepted: 0,
            params,
            iter_in_temp: 0,
            rng: rand::thread_rng(),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.temperature <= self.params.t_final
    }

    /// Run `steps` neighbour evaluations; cool down when iter_per_temp is reached.
    pub fn step(&mut self, problem: &Problem, steps: usize) {
        for _ in 0..steps {
            if self.is_finished() {
                return;
            }

            let candidate = generate_neighbor(&self.current, problem, &mut self.rng);
            let candidate_cost = candidate.total_distance(problem);
            let delta = candidate_cost - self.current_cost;

            if delta < 0.0 || self.rng.gen_range(0.0f64..1.0) < (-delta / self.temperature).exp() {
                self.current = candidate;
                self.current_cost = candidate_cost;
                self.accepted += 1;

                if self.current_cost < self.best_cost {
                    self.best = self.current.clone();
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

    /// Acceptance rate since start
    pub fn acceptance_rate(&self) -> f64 {
        if self.total_iterations == 0 {
            return 0.0;
        }
        self.accepted as f64 / self.total_iterations as f64
    }
}

// ─── Neighbourhood operators ──────────────────────────────────────────────────

fn generate_neighbor(sol: &Solution, problem: &Problem, rng: &mut impl Rng) -> Solution {
    match rng.gen_range(0..3u8) {
        0 => relocate(sol, problem, rng),
        1 => two_opt_intra(sol, rng),
        _ => swap_inter(sol, problem, rng),
    }
}

/// Move one client from a route to any position in another route.
fn relocate(sol: &Solution, problem: &Problem, rng: &mut impl Rng) -> Solution {
    let non_empty: Vec<usize> = sol
        .routes
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.is_empty())
        .map(|(i, _)| i)
        .collect();

    if non_empty.len() < 2 {
        return sol.clone();
    }

    let &from = non_empty.choose(rng).unwrap();
    let pos_from = rng.gen_range(0..sol.routes[from].len());
    let client = sol.routes[from][pos_from];

    let to_candidates: Vec<usize> = non_empty.iter().filter(|&&i| i != from).cloned().collect();
    let &to = to_candidates.choose(rng).unwrap();

    // capacity check
    let demand: u32 = problem.route_demand(&sol.routes[to]) + problem.clients[client].demand;
    if demand > problem.capacity {
        return sol.clone();
    }

    let pos_to = rng.gen_range(0..=sol.routes[to].len());
    let mut new_routes = sol.routes.clone();
    new_routes[from].remove(pos_from);
    new_routes[to].insert(pos_to, client);
    new_routes.retain(|r| !r.is_empty());

    Solution { routes: new_routes }
}

/// Reverse a sub-sequence inside a single route (2-opt).
fn two_opt_intra(sol: &Solution, rng: &mut impl Rng) -> Solution {
    let long_routes: Vec<usize> = sol
        .routes
        .iter()
        .enumerate()
        .filter(|(_, r)| r.len() >= 2)
        .map(|(i, _)| i)
        .collect();

    if long_routes.is_empty() {
        return sol.clone();
    }

    let &ri = long_routes.choose(rng).unwrap();
    let n = sol.routes[ri].len();
    let i = rng.gen_range(0..n - 1);
    let j = rng.gen_range(i + 1..n);

    let mut new_routes = sol.routes.clone();
    new_routes[ri][i..=j].reverse();

    Solution { routes: new_routes }
}

/// Swap one client from route A with one client from route B.
fn swap_inter(sol: &Solution, problem: &Problem, rng: &mut impl Rng) -> Solution {
    let non_empty: Vec<usize> = sol
        .routes
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.is_empty())
        .map(|(i, _)| i)
        .collect();

    if non_empty.len() < 2 {
        return sol.clone();
    }

    let &r1 = non_empty.choose(rng).unwrap();
    let r2_candidates: Vec<usize> = non_empty.iter().filter(|&&i| i != r1).cloned().collect();
    let &r2 = r2_candidates.choose(rng).unwrap();

    let p1 = rng.gen_range(0..sol.routes[r1].len());
    let p2 = rng.gen_range(0..sol.routes[r2].len());
    let c1 = sol.routes[r1][p1];
    let c2 = sol.routes[r2][p2];

    let d1 = problem.route_demand(&sol.routes[r1]) - problem.clients[c1].demand
        + problem.clients[c2].demand;
    let d2 = problem.route_demand(&sol.routes[r2]) - problem.clients[c2].demand
        + problem.clients[c1].demand;

    if d1 > problem.capacity || d2 > problem.capacity {
        return sol.clone();
    }

    let mut new_routes = sol.routes.clone();
    new_routes[r1][p1] = c2;
    new_routes[r2][p2] = c1;

    Solution { routes: new_routes }
}
