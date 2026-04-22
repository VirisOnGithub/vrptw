use crate::problem::{Problem, Solution};
mod relocate;
mod top_opt;

pub trait Neighbouring: Send + Sync {
    fn generate_neighbor(
        &self,
        current_solution: &Solution,
        problem: &Problem,
        rng: &mut dyn rand::RngCore,
        time_into_account: bool,
    ) -> Solution;
}

pub struct NeighbouringFactory(pub fn() -> Box<dyn Neighbouring>);

inventory::collect!(NeighbouringFactory);

// generate neighbor using a random strategy among the available strategies
pub fn generate_neighbor(
    current_solution: &Solution,
    problem: &Problem,
    rng: &mut impl rand::Rng,
    time_into_account: bool,
) -> Solution {
    let strategies: Vec<_> = inventory::iter::<NeighbouringFactory>().collect();
    let index = rng.gen_range(0..strategies.len());
    let strategy = (strategies[index].0)();
    strategy.generate_neighbor(current_solution, problem, rng, time_into_account)
}
