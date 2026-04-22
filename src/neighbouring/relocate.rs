use super::Neighbouring;
use crate::{
    neighbouring::NeighbouringFactory,
    problem::{Problem, Solution},
};
use rand::{Rng, prelude::SliceRandom};

struct RelocateNeighbouring;

impl Neighbouring for RelocateNeighbouring {
    fn generate_neighbor(
        &self,
        current_solution: &Solution,
        problem: &Problem,
        rng: &mut dyn rand::RngCore,
        time_into_account: bool,
    ) -> Solution {
        let sol = current_solution.clone();
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
        if demand > problem.max_capacity {
            return sol.clone();
        }

        let pos_to = rng.gen_range(0..=sol.routes[to].len());

        // time check
        if time_into_account {
            let mut new_routes = sol.routes.clone();
            new_routes[from].remove(pos_from);
            new_routes[to].insert(pos_to, client);
            new_routes.retain(|r| !r.is_empty());
            let new_solution = Solution { routes: new_routes };
            if !new_solution.is_feasible(problem) {
                return sol.clone();
            }
        }

        let mut new_routes = sol.routes.clone();
        new_routes[from].remove(pos_from);
        new_routes[to].insert(pos_to, client);
        new_routes.retain(|r| !r.is_empty());

        Solution { routes: new_routes }
    }
}

inventory::submit!(NeighbouringFactory(|| Box::new(RelocateNeighbouring)));
