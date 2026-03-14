use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::solution::{Problem, Solution};

// ─── Parameters ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct TabuParams {
    /// How many iterations a moved client stays forbidden.
    pub tabu_tenure: usize,
    /// Total number of iterations before stopping.
    pub max_iterations: usize,
    /// Random neighbours evaluated at each iteration.
    pub n_neighbors: usize,
}

impl Default for TabuParams {
    fn default() -> Self {
        TabuParams {
            tabu_tenure: 10,
            max_iterations: 5_000,
            n_neighbors: 50,
        }
    }
}

// ─── Step-based Tabu Search (used by the GUI) ─────────────────────────────────

pub struct TabuSearch {
    pub current: Solution,
    pub current_cost: f64,
    pub best: Solution,
    pub best_cost: f64,
    pub iteration: usize,
    pub improved: usize, // number of times best was improved
    pub params: TabuParams,
    /// `tabu_until[c]` = iteration index until which client `c` is tabu.
    tabu_until: Vec<usize>,
    rng: ThreadRng,
}

impl TabuSearch {
    pub fn new(problem: &Problem, params: TabuParams) -> Self {
        let n = problem.clients.len();
        let initial = Solution::random(problem);
        let cost = initial.total_distance(problem);
        TabuSearch {
            current: initial.clone(),
            current_cost: cost,
            best: initial,
            best_cost: cost,
            iteration: 0,
            improved: 0,
            params,
            tabu_until: vec![0; n],
            rng: rand::thread_rng(),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.iteration >= self.params.max_iterations
    }

    /// Run up to `steps` iterations of tabu search.
    pub fn step(&mut self, problem: &Problem, steps: usize) {
        for _ in 0..steps {
            if self.is_finished() {
                return;
            }

            let mut best_candidate: Option<(Solution, f64, Vec<usize>)> = None;

            for _ in 0..self.params.n_neighbors {
                let (candidate, moved) =
                    generate_neighbor_with_move(&self.current, problem, &mut self.rng);
                let cost = candidate.total_distance(problem);

                // Aspiration criterion: always accept a solution better than global best.
                let aspiration = cost < self.best_cost;

                // A move is tabu when ANY moved client is still forbidden.
                let is_tabu = moved.iter().any(|&c| self.tabu_until[c] > self.iteration);

                if !is_tabu || aspiration {
                    if best_candidate
                        .as_ref()
                        .map(|(_, bc, _)| cost < *bc)
                        .unwrap_or(true)
                    {
                        best_candidate = Some((candidate, cost, moved));
                    }
                }
            }

            // If every candidate was tabu (rare), fall back to the cheapest regardless.
            if best_candidate.is_none() {
                let mut fallback_cost = f64::INFINITY;
                for _ in 0..self.params.n_neighbors {
                    let (candidate, moved) =
                        generate_neighbor_with_move(&self.current, problem, &mut self.rng);
                    let cost = candidate.total_distance(problem);
                    if cost < fallback_cost {
                        fallback_cost = cost;
                        best_candidate = Some((candidate, cost, moved));
                    }
                }
            }

            if let Some((sol, cost, moved)) = best_candidate {
                // Mark moved clients as tabu for the next `tabu_tenure` iterations.
                let until = self.iteration + self.params.tabu_tenure;
                for c in moved {
                    self.tabu_until[c] = until;
                }

                self.current = sol;
                self.current_cost = cost;

                if cost < self.best_cost {
                    self.best = self.current.clone();
                    self.best_cost = cost;
                    self.improved += 1;
                }
            }

            self.iteration += 1;
        }
    }
}

// ─── Neighbourhood operators (return moved-client indices) ────────────────────

/// Pick one of three neighbourhood operators at random and return the resulting
/// solution together with the list of client indices whose position changed.
fn generate_neighbor_with_move(
    sol: &Solution,
    problem: &Problem,
    rng: &mut impl Rng,
) -> (Solution, Vec<usize>) {
    match rng.gen_range(0..3u8) {
        0 => relocate(sol, problem, rng),
        1 => two_opt_intra(sol, rng),
        _ => swap_inter(sol, problem, rng),
    }
}

/// Move one client from a route to any position in another route.
fn relocate(sol: &Solution, problem: &Problem, rng: &mut impl Rng) -> (Solution, Vec<usize>) {
    let non_empty: Vec<usize> = sol
        .routes
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.is_empty())
        .map(|(i, _)| i)
        .collect();

    if non_empty.len() < 2 {
        return (sol.clone(), vec![]);
    }

    let &from = non_empty.choose(rng).unwrap();
    let pos_from = rng.gen_range(0..sol.routes[from].len());
    let client = sol.routes[from][pos_from];

    let to_candidates: Vec<usize> = non_empty.iter().filter(|&&i| i != from).cloned().collect();
    let &to = to_candidates.choose(rng).unwrap();

    let demand = problem.route_demand(&sol.routes[to]) + problem.clients[client].demand;
    if demand > problem.capacity {
        return (sol.clone(), vec![]);
    }

    let pos_to = rng.gen_range(0..=sol.routes[to].len());
    let mut new_routes = sol.routes.clone();
    new_routes[from].remove(pos_from);
    new_routes[to].insert(pos_to, client);
    new_routes.retain(|r| !r.is_empty());

    (Solution { routes: new_routes }, vec![client])
}

/// Reverse a sub-sequence inside a single route (2-opt intra). The tabu
/// attributes are the two boundary clients of the reversed segment.
fn two_opt_intra(sol: &Solution, rng: &mut impl Rng) -> (Solution, Vec<usize>) {
    let long_routes: Vec<usize> = sol
        .routes
        .iter()
        .enumerate()
        .filter(|(_, r)| r.len() >= 2)
        .map(|(i, _)| i)
        .collect();

    if long_routes.is_empty() {
        return (sol.clone(), vec![]);
    }

    let &ri = long_routes.choose(rng).unwrap();
    let n = sol.routes[ri].len();
    let i = rng.gen_range(0..n - 1);
    let j = rng.gen_range(i + 1..n);

    let c_i = sol.routes[ri][i];
    let c_j = sol.routes[ri][j];

    let mut new_routes = sol.routes.clone();
    new_routes[ri][i..=j].reverse();

    (Solution { routes: new_routes }, vec![c_i, c_j])
}

/// Swap one client from route A with one client from route B.
fn swap_inter(sol: &Solution, problem: &Problem, rng: &mut impl Rng) -> (Solution, Vec<usize>) {
    let non_empty: Vec<usize> = sol
        .routes
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.is_empty())
        .map(|(i, _)| i)
        .collect();

    if non_empty.len() < 2 {
        return (sol.clone(), vec![]);
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
        return (sol.clone(), vec![]);
    }

    let mut new_routes = sol.routes.clone();
    new_routes[r1][p1] = c2;
    new_routes[r2][p2] = c1;

    (Solution { routes: new_routes }, vec![c1, c2])
}
