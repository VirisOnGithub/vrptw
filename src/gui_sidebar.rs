use egui::Widget;

use crate::{
    VrpApp,
    algorithm_runner::AlgorithmRunner,
    optimizing_algorithm::{SAParams, SimulatedAnnealing},
    problem::{Problem, Solution},
};

pub trait Sidebar {
    fn show_sidebar(&mut self, ctx: &eframe::egui::Context);
}

impl Sidebar for VrpApp {
    fn show_sidebar(&mut self, ctx: &eframe::egui::Context) {
        egui::SidePanel::left("controls")
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("VRPTW Solver");
                egui::Checkbox::new(&mut self.time_into_account, "Temps pris en compte").ui(ui);
                ui.separator();

                // sélecteur
                let selected_text = self.files[self.selected_file_id]
                    .to_string_lossy()
                    .into_owned();
                egui::ComboBox::new("file_selector", "Select a file")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for (i, file) in self.files.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.selected_file_id,
                                i,
                                file.display().to_string(),
                            );
                        }
                    });
                if ui.button("Charger").clicked() {
                    self.algorithm_runner = None;
                    self.iterations_done = 0;
                    let selected_file = self.files[self.selected_file_id].clone();
                    let input_data = self.load_file(selected_file);
                    self.buffer = format!("{:#?}", input_data);
                    let problem = Problem::new(input_data);
                    self.problem = Some(problem.clone());
                    let solution = Solution::random(
                        &self
                            .problem
                            .clone()
                            .expect("No problem was submitted before solving"),
                    );
                    self.solution = Some(solution.clone());
                    self.buffer = format!("{:#?}", solution);
                }
                if ui.button("Effacer").clicked() {
                    self.algorithm_runner = None;
                    self.iterations_done = 0;
                    self.buffer.clear();
                    self.problem = None;
                    self.solution = None;
                }
                ui.add_enabled_ui(self.problem.is_some(), |ui| {
                    if ui.button("Résoudre").clicked() {
                        let sa_params = SAParams::default();
                        let pb = self.problem.clone().unwrap();
                        let current_solution = self.solution.clone().unwrap();
                        self.algorithm_runner = Some(AlgorithmRunner::new(
                            Box::new(SimulatedAnnealing::new(&pb, &current_solution, sa_params)),
                            pb,
                        ));
                        self.iterations_done = 0;
                    }
                });

                ui.label("Iter/frame");
                ui.add(egui::Slider::new(&mut self.iter_per_frame, 100..=10000).step_by(100.))
                    .on_hover_text("Nombre d'itérations calculées par frame");
                ui.label(format!("Itérations totales: {}", self.iterations_done));

                ui.separator();
            });
    }
}
