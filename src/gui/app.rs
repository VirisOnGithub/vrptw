use egui::{Color32, Rect, Stroke, Vec2};
use std::any::Any;
use std::path::PathBuf;

use crate::{
    algorithm_runner::AlgorithmRunner,
    gui::gui_canvas::{MapTransform, draw_arrow, route_color},
    gui::gui_sidebar::Sidebar,
    optimizers::{OptimizerDescriptor, available_optimizers},
    parser::InputData,
    problem::{Problem, Solution},
};

pub struct VrpApp {
    pub files: Vec<PathBuf>,
    pub selected_file_id: usize,
    pub time_into_account: bool,
    pub buffer: String,
    pub problem: Option<Problem>,
    pub solution: Option<Solution>,
    pub iter_per_frame: usize,
    pub algorithm_runner: Option<AlgorithmRunner>,
    pub iterations_done: usize,
    pub optimizers: Vec<&'static OptimizerDescriptor>,
    pub selected_optimizer: usize,
    pub optimizer_params: Vec<Box<dyn Any + Send + Sync>>,
}

impl Default for VrpApp {
    fn default() -> Self {
        Self::new()
    }
}

impl VrpApp {
    pub fn new() -> Self {
        let data_dir = "data";
        let optimizers = available_optimizers();
        let optimizer_params = optimizers
            .iter()
            .map(|descriptor| (descriptor.create_default_params)())
            .collect();
        let files = std::fs::read_dir(data_dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|file| file.path())
            .filter(|file| file.extension().map(|e| e == "vrp").unwrap_or(false))
            .collect();
        Self {
            files,
            selected_file_id: 0,
            time_into_account: false,
            buffer: String::new(),
            problem: None,
            solution: None,
            iter_per_frame: 5000,
            algorithm_runner: None,
            iterations_done: 0,
            optimizers,
            selected_optimizer: 0,
            optimizer_params,
        }
    }

    pub fn load_file(&mut self, selected_file: PathBuf) -> InputData {
        let file_contents = std::fs::read_to_string(selected_file);
        InputData::parse_input(file_contents.expect("IO error for file").as_str())
    }
}

impl eframe::App for VrpApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.show_sidebar(ctx);
        if let Some(runner) = self.algorithm_runner.as_mut() {
            if let Some(update) = runner.poll_latest_update() {
                self.solution = Some(update.solution.clone());
                self.iterations_done = update.total_iterations;
                self.buffer = format!(
                    "it/frame done: {}\nitérations totales: {}\n{:#?}",
                    update.iterations_done, update.total_iterations, update.solution
                );
            }
            if !runner.is_finished() {
                runner.request_step(self.iter_per_frame);
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(problem) = &self.problem else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Chargez un fichier .vrp pour commencer")
                            .size(18.0)
                            .color(Color32::GRAY),
                    );
                });
                return;
            };

            let rect = ui.available_rect_before_wrap();
            let t = MapTransform::build(problem, rect);
            let painter = ui.painter();
            let Some(solution) = &self.solution else {
                return;
            };

            let depot = t.to_screen(problem.repo.x, problem.repo.y);
            for (ri, route) in solution.routes.iter().enumerate() {
                if route.is_empty() {
                    continue;
                }
                let color = route_color(ri);
                let stroke = Stroke::new(1.8, color);

                let p_first = t.to_screen(problem.clients[route[0]].x, problem.clients[route[0]].y);
                draw_arrow(painter, depot, p_first, stroke);

                for w in route.windows(2) {
                    let a = &problem.clients[w[0]];
                    let b = &problem.clients[w[1]];
                    draw_arrow(
                        painter,
                        t.to_screen(a.x, a.y),
                        t.to_screen(b.x, b.y),
                        stroke,
                    );
                }

                let last = &problem.clients[*route.last().unwrap()];
                draw_arrow(painter, t.to_screen(last.x, last.y), depot, stroke);
            }

            for (i, client) in problem.clients.iter().enumerate() {
                let pos = t.to_screen(client.x, client.y);
                let ri = solution
                    .routes
                    .iter()
                    .position(|r| r.contains(&i))
                    .unwrap_or(0);
                let color = route_color(ri);
                painter.circle_filled(pos, 5.0, color);
                painter.circle_stroke(pos, 5.0, Stroke::new(1.0, Color32::WHITE));
            }

            // repo
            let size = 10.0;
            let square = Rect::from_center_size(depot, Vec2::splat(size));
            painter.rect_filled(square, 0.0, Color32::WHITE);
        });
    }
}
