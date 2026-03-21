use rand::{Rng, seq::SliceRandom};

use super::Neighbouring;
use crate::{
    neighbouring::NeighbouringFactory,
    problem::{Problem, Solution},
};

struct TwoOptNeighbouring;

impl Neighbouring for TwoOptNeighbouring {
    fn generate_neighbor(
        &self,
        current_solution: &Solution,
        _problem: &Problem,
        rng: &mut dyn rand::RngCore,
    ) -> Solution {
        let sol = current_solution.clone();
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
}

inventory::submit!(NeighbouringFactory(|| Box::new(TwoOptNeighbouring)));
