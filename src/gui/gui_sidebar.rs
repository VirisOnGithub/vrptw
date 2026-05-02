use egui::Widget;

use crate::{
    VrpApp,
    algorithm_runner::AlgorithmRunner,
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
                egui::Checkbox::new(&mut self.show_steps, "Afficher les étapes").ui(ui);
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
                ui.label("Solutions Initiales");
                if ui.button("Plus simple").clicked() {
                    self.gen_basic_solution("simplest".to_string());
                }
                if ui.button("Glouton").clicked() {
                    self.gen_basic_solution("greedy".to_string());
                }
                if ui.button("Aléatoire").clicked() {
                    self.gen_basic_solution("random".to_string());
                }
                if ui.button("Effacer").clicked() {
                    self.reset();
                }
                ui.add_enabled_ui(self.problem.is_some(), |ui| {
                    if ui.button("Résoudre").clicked() {
                        self.starting_time = Some(std::time::Instant::now());
                        if self.optimizers.is_empty() {
                            self.buffer =
                                "Aucun algorithme n'est enregistré dans le registre".to_string();
                            return;
                        }

                        let pb = self.problem.clone().unwrap();
                        let current_solution = self.solution.clone().unwrap();
                        let descriptor = self.optimizers[self.selected_optimizer];
                        let params = &self.optimizer_params[self.selected_optimizer];
                        let algo = (descriptor.build_algorithm)(
                            &pb,
                            &current_solution,
                            params.as_ref(),
                            self.time_into_account,
                        );

                        self.algorithm_runner = Some(AlgorithmRunner::new(algo, pb));
                        self.iterations_done = 0;
                    }
                });

                ui.label("Iter/frame");
                ui.add(egui::Slider::new(&mut self.iter_per_frame, 100..=10000).step_by(100.))
                    .on_hover_text("Nombre d'itérations calculées par frame");
                ui.label(format!("Itérations totales: {}", self.iterations_done));

                if let Some(last_optimization_time) = self.last_optimization_time {
                    ui.label(format!(
                        "Temps écoulé: {:.2}s",
                        last_optimization_time.as_secs_f32()
                    ));
                } else if let Some(start) = self.starting_time {
                    let elapsed = start.elapsed();
                    ui.label(format!("Temps écoulé: {:.2}s", elapsed.as_secs_f32()));
                }

                // Affichage des résultats finaux
                if let (Some(solution), Some(problem)) = (&self.solution, &self.problem) {
                    ui.separator();
                    ui.heading("Résultats");

                    let num_trucks = solution
                        .routes
                        .iter()
                        .filter(|route| !route.is_empty())
                        .count();
                    ui.label(format!("Camions utilisés: {}", num_trucks));

                    let total_score = solution.total_distance(problem);
                    ui.label(format!("Score total (distance): {:.2}", total_score));
                }

                ui.separator();

                self.show_algorithm_parameters(ui);
            });
    }
}

impl VrpApp {
    fn show_algorithm_parameters(&mut self, ui: &mut egui::Ui) {
        ui.heading("Algorithme");

        if self.optimizers.is_empty() {
            ui.label("Aucun algorithme disponible");
            return;
        }

        if self.selected_optimizer >= self.optimizers.len() {
            self.selected_optimizer = 0;
        }

        let selected_text = self.optimizers[self.selected_optimizer].label;
        egui::ComboBox::new("optimizer_selector", "Choix de l'algorithme")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for (i, optimizer) in self.optimizers.iter().enumerate() {
                    ui.selectable_value(&mut self.selected_optimizer, i, optimizer.label);
                }
            });

        let descriptor = self.optimizers[self.selected_optimizer];
        let params = &mut self.optimizer_params[self.selected_optimizer];
        (descriptor.draw_params_ui)(params.as_mut(), ui);
    }

    fn reset(&mut self) {
        self.algorithm_runner = None;
        self.iterations_done = 0;
        self.buffer.clear();
        self.problem = None;
        self.solution = None;
        self.is_random_solution = false;
        self.starting_time = None;
        self.last_optimization_time = None;
    }

    fn gen_basic_solution(&mut self, sol_type: String) {
        self.reset();
        let selected_file = self.files[self.selected_file_id].clone();
        let input_data = self.load_file(selected_file);
        self.buffer = format!("{:#?}", input_data);
        let problem = Problem::new(input_data);
        self.problem = Some(problem.clone());
        let solution = match sol_type.as_str() {
            "simplest" => Solution::simplest(&problem),
            "greedy" => Solution::greedy(&problem),
            "random" => Solution::random(&problem),
            _ => panic!("Unknown solution type"),
        };
        self.solution = Some(solution.clone());
        self.buffer = format!("{:#?}", solution);
        self.is_random_solution = true;
    }
}
