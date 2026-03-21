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
        problem: &Problem,
        rng: &mut dyn rand::RngCore,
    ) -> Solution {
        todo!()
    }
}

inventory::submit!(NeighbouringFactory(|| Box::new(TwoOptNeighbouring)));
