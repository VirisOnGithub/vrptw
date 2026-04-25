pub(crate) mod simulated_annealing;
pub(crate) mod aco;

use std::any::Any;

use crate::problem::{Problem, Solution};

pub trait OptimizationAlgorithm {
    fn total_iterations(&self) -> usize;
    fn current_solution(&self) -> &Solution;
    fn step(&mut self, problem: &Problem, nb_steps: usize);
    fn is_finished(&self) -> bool;
}

pub struct OptimizerDescriptor {
    pub id: &'static str,
    pub label: &'static str,
    pub create_default_params: fn() -> Box<dyn Any + Send + Sync>,
    pub draw_params_ui: fn(&mut dyn Any, &mut egui::Ui),
    pub build_algorithm: fn(
        problem: &Problem,
        solution: &Solution,
        params: &dyn Any,
        time_into_account: bool,
    ) -> Box<dyn OptimizationAlgorithm + Send + Sync>,
}

inventory::collect!(OptimizerDescriptor);

pub(crate) fn available_optimizers() -> Vec<&'static OptimizerDescriptor> {
    let mut optimizers: Vec<&'static OptimizerDescriptor> =
        inventory::iter::<OptimizerDescriptor>.into_iter().collect();
    optimizers.sort_by(|a, b| a.label.cmp(b.label).then(a.id.cmp(b.id)));
    optimizers
}
