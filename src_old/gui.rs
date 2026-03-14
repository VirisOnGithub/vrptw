use eframe::egui;
use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};

use crate::parser::InputData;
use crate::simulated_annealing::{SAParams, SimulatedAnnealing};
use crate::solution::{Problem, Solution};
use crate::tabu_search::{TabuParams, TabuSearch};

// ─── Colour palette for routes ────────────────────────────────────────────────

const PALETTE: &[Color32] = &[
    Color32::from_rgb(220, 50, 47),
    Color32::from_rgb(38, 139, 210),
    Color32::from_rgb(133, 153, 0),
    Color32::from_rgb(211, 54, 130),
    Color32::from_rgb(42, 161, 152),
    Color32::from_rgb(181, 137, 0),
    Color32::from_rgb(108, 113, 196),
    Color32::from_rgb(203, 75, 22),
    Color32::from_rgb(0, 168, 132),
    Color32::from_rgb(255, 128, 0),
    Color32::from_rgb(0, 102, 204),
    Color32::from_rgb(153, 0, 204),
];

fn route_color(idx: usize) -> Color32 {
    PALETTE[idx % PALETTE.len()]
}

// ─── Solver selection ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SolverKind {
    SimulatedAnnealing,
    TabuSearch,
}

/// Unified wrapper so the GUI can drive either solver with a single field.
pub enum ActiveSolver {
    SA(SimulatedAnnealing),
    Tabu(TabuSearch),
}

impl ActiveSolver {
    pub fn is_finished(&self) -> bool {
        match self {
            ActiveSolver::SA(s) => s.is_finished(),
            ActiveSolver::Tabu(s) => s.is_finished(),
        }
    }

    pub fn step(&mut self, problem: &Problem, steps: usize) {
        match self {
            ActiveSolver::SA(s) => s.step(problem, steps),
            ActiveSolver::Tabu(s) => s.step(problem, steps),
        }
    }

    pub fn best(&self) -> &Solution {
        match self {
            ActiveSolver::SA(s) => &s.best,
            ActiveSolver::Tabu(s) => &s.best,
        }
    }

    pub fn best_cost(&self) -> f64 {
        match self {
            ActiveSolver::SA(s) => s.best_cost,
            ActiveSolver::Tabu(s) => s.best_cost,
        }
    }

    pub fn current_cost(&self) -> f64 {
        match self {
            ActiveSolver::SA(s) => s.current_cost,
            ActiveSolver::Tabu(s) => s.current_cost,
        }
    }

    /// Extra status lines specific to each algorithm.
    pub fn status_lines(&self) -> Vec<(String, String)> {
        match self {
            ActiveSolver::SA(s) => vec![
                ("Température".to_string(), format!("{:.4}", s.temperature)),
                ("Itérations".to_string(), format!("{}", s.total_iterations)),
                (
                    "Taux accept.".to_string(),
                    format!("{:.1} %", s.acceptance_rate() * 100.0),
                ),
            ],
            ActiveSolver::Tabu(s) => vec![
                (
                    "Itération".to_string(),
                    format!("{} / {}", s.iteration, s.params.max_iterations),
                ),
                ("Améliorations".to_string(), format!("{}", s.improved)),
            ],
        }
    }
}

// ─── Coordinate transform ─────────────────────────────────────────────────────

struct MapTransform {
    min_x: f64,
    max_y: f64,
    scale: f64,
    origin: Pos2,
}

impl MapTransform {
    fn build(problem: &Problem, rect: Rect) -> Self {
        let inner = rect.shrink(40.0);

        let xs = problem.clients.iter().map(|c| c.x).chain([problem.depot.0]);
        let ys = problem.clients.iter().map(|c| c.y).chain([problem.depot.1]);

        let min_x = xs.clone().fold(f64::INFINITY, f64::min);
        let max_x = xs.fold(f64::NEG_INFINITY, f64::max);
        let min_y = ys.clone().fold(f64::INFINITY, f64::min);
        let max_y = ys.fold(f64::NEG_INFINITY, f64::max);

        let sx = inner.width() as f64 / (max_x - min_x).max(1.0);
        let sy = inner.height() as f64 / (max_y - min_y).max(1.0);

        MapTransform {
            min_x,
            max_y,
            scale: sx.min(sy),
            origin: inner.min,
        }
    }

    #[inline]
    fn to_screen(&self, x: f64, y: f64) -> Pos2 {
        Pos2 {
            x: self.origin.x + ((x - self.min_x) * self.scale) as f32,
            y: self.origin.y + ((self.max_y - y) * self.scale) as f32,
        }
    }
}

// ─── Drawing helpers ──────────────────────────────────────────────────────────

fn draw_arrow(painter: &Painter, from: Pos2, to: Pos2, stroke: Stroke) {
    painter.line_segment([from, to], stroke);
    let dir = (to - from).normalized();
    let perp = Vec2::new(-dir.y, dir.x);
    let tip = to - dir * 8.0;
    painter.line_segment([to, tip + perp * 4.0], stroke);
    painter.line_segment([to, tip - perp * 4.0], stroke);
}

// ─── App state ────────────────────────────────────────────────────────────────

pub struct VrpApp {
    data_files: Vec<std::path::PathBuf>,
    selected_idx: usize,

    problem: Option<Problem>,
    solver: Option<ActiveSolver>,

    solver_kind: SolverKind,
    sa_params: SAParams,
    tabu_params: TabuParams,
    steps_per_frame: usize,
    running: bool,
    show_ids: bool,
}

impl VrpApp {
    pub fn new() -> Self {
        let mut data_files: Vec<std::path::PathBuf> = std::fs::read_dir("data")
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "vrp").unwrap_or(false))
            .collect();
        data_files.sort();

        VrpApp {
            data_files,
            selected_idx: 0,
            problem: None,
            solver: None,
            solver_kind: SolverKind::SimulatedAnnealing,
            sa_params: SAParams::default(),
            tabu_params: TabuParams::default(),
            steps_per_frame: 300,
            running: false,
            show_ids: false,
        }
    }

    fn load(&mut self) {
        let Some(path) = self.data_files.get(self.selected_idx) else {
            return;
        };
        if let Ok(text) = std::fs::read_to_string(path) {
            let data = InputData::parse_input(&text);
            let problem = Problem::from_input(&data);
            self.solver = Some(self.make_solver(&problem));
            self.problem = Some(problem);
            self.running = false;
        }
    }

    fn reset(&mut self) {
        if let Some(p) = &self.problem {
            let p = p.clone();
            self.solver = Some(self.make_solver(&p));
        }
        self.running = false;
    }

    fn make_solver(&self, problem: &Problem) -> ActiveSolver {
        match self.solver_kind {
            SolverKind::SimulatedAnnealing => {
                ActiveSolver::SA(SimulatedAnnealing::new(problem, self.sa_params.clone()))
            }
            SolverKind::TabuSearch => {
                ActiveSolver::Tabu(TabuSearch::new(problem, self.tabu_params.clone()))
            }
        }
    }

    // ── Left panel ────────────────────────────────────────────────────────────
    fn ui_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("VRPTW Solver");
        ui.separator();

        // ── Algorithm selector ────────────────────────────────────────────────
        ui.label("Métaheuristique :");
        ui.horizontal(|ui| {
            if ui
                .selectable_value(
                    &mut self.solver_kind,
                    SolverKind::SimulatedAnnealing,
                    "Recuit simulé",
                )
                .clicked()
            {
                self.reset();
            }
            if ui
                .selectable_value(&mut self.solver_kind, SolverKind::TabuSearch, "Liste tabou")
                .clicked()
            {
                self.reset();
            }
        });

        ui.separator();

        ui.label("Fichier de données :");
        if self.data_files.is_empty() {
            ui.label("(aucun fichier .vrp dans data/)");
        } else {
            let current_name = self.data_files[self.selected_idx]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            egui::ComboBox::new("file_combo", "")
                .selected_text(&current_name)
                .show_ui(ui, |ui| {
                    for (i, p) in self.data_files.iter().enumerate() {
                        let label = p.file_name().unwrap().to_string_lossy().to_string();
                        ui.selectable_value(&mut self.selected_idx, i, label);
                    }
                });
        }

        if ui.button("📂  Charger").clicked() {
            self.load();
        }

        ui.separator();

        // ── Algorithm-specific parameters ─────────────────────────────────────
        match self.solver_kind {
            SolverKind::SimulatedAnnealing => {
                ui.label("Paramètres — Recuit Simulé :");
                ui.horizontal(|ui| {
                    ui.label("T₀           ");
                    ui.add(
                        egui::DragValue::new(&mut self.sa_params.t_initial)
                            .speed(10.0)
                            .range(1.0..=10_000.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("T_final    ");
                    ui.add(
                        egui::DragValue::new(&mut self.sa_params.t_final)
                            .speed(0.01)
                            .range(0.001..=50.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("α               ");
                    ui.add(
                        egui::DragValue::new(&mut self.sa_params.alpha)
                            .speed(0.001)
                            .range(0.9..=0.9999_f64),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Iter/T       ");
                    ui.add(
                        egui::DragValue::new(&mut self.sa_params.iter_per_temp)
                            .speed(10.0)
                            .range(10..=5000_usize),
                    );
                });
            }
            SolverKind::TabuSearch => {
                ui.label("Paramètres — Liste Tabou :");
                ui.horizontal(|ui| {
                    ui.label("Tenure       ");
                    ui.add(
                        egui::DragValue::new(&mut self.tabu_params.tabu_tenure)
                            .speed(1.0)
                            .range(1..=100_usize),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Max iter     ");
                    ui.add(
                        egui::DragValue::new(&mut self.tabu_params.max_iterations)
                            .speed(100.0)
                            .range(100..=100_000_usize),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Voisins/iter ");
                    ui.add(
                        egui::DragValue::new(&mut self.tabu_params.n_neighbors)
                            .speed(5.0)
                            .range(5..=500_usize),
                    );
                });
            }
        }

        ui.horizontal(|ui| {
            ui.label("Iter/frame");
            ui.add(
                egui::DragValue::new(&mut self.steps_per_frame)
                    .speed(10.0)
                    .range(1..=5000_usize),
            );
        });

        ui.separator();

        ui.horizontal(|ui| {
            let has_solver = self
                .solver
                .as_ref()
                .map(|s| !s.is_finished())
                .unwrap_or(false);
            let btn = if self.running {
                "⏸  Pause"
            } else {
                "▶  Lancer"
            };
            if ui.add_enabled(has_solver, egui::Button::new(btn)).clicked() {
                self.running = !self.running;
                if self.running {
                    ctx.request_repaint();
                }
            }
            if ui.button("⟳  Reset").clicked() {
                self.reset();
            }
        });

        ui.separator();

        if let Some(solver) = &self.solver {
            ui.label(
                egui::RichText::new(format!("Tournées :   {}", solver.best().routes.len()))
                    .strong(),
            );
            ui.label(format!("Meilleur coût :  {:.2}", solver.best_cost()));
            ui.label(format!("Coût courant :   {:.2}", solver.current_cost()));
            for (label, value) in solver.status_lines() {
                ui.label(format!("{} :   {}", label, value));
            }
            if solver.is_finished() {
                ui.colored_label(Color32::from_rgb(80, 200, 100), "✔  Terminé");
            }
        } else {
            ui.label("Aucune solution chargée.");
        }

        ui.separator();
        ui.checkbox(&mut self.show_ids, "Afficher IDs clients");
    }

    // ── Central canvas ────────────────────────────────────────────────────────
    fn ui_canvas(&self, ui: &mut egui::Ui) {
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

        let Some(solver) = &self.solver else {
            return;
        };

        let available = ui.available_rect_before_wrap();
        let painter = ui.painter_at(available);

        painter.rect_filled(available, 4.0, Color32::from_rgb(28, 28, 32));

        let t = MapTransform::build(problem, available);
        let solution = solver.best();

        for (ri, route) in solution.routes.iter().enumerate() {
            if route.is_empty() {
                continue;
            }
            let color = route_color(ri);
            let stroke = Stroke::new(1.8, color);
            let depot = t.to_screen(problem.depot.0, problem.depot.1);

            let p_first = t.to_screen(problem.clients[route[0]].x, problem.clients[route[0]].y);
            draw_arrow(&painter, depot, p_first, stroke);

            for w in route.windows(2) {
                let a = &problem.clients[w[0]];
                let b = &problem.clients[w[1]];
                draw_arrow(
                    &painter,
                    t.to_screen(a.x, a.y),
                    t.to_screen(b.x, b.y),
                    stroke,
                );
            }

            let last = &problem.clients[*route.last().unwrap()];
            draw_arrow(&painter, t.to_screen(last.x, last.y), depot, stroke);
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
            if self.show_ids {
                painter.text(
                    pos + Vec2::new(6.0, -6.0),
                    egui::Align2::LEFT_BOTTOM,
                    &client.id,
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );
            }
        }

        // depot square
        let dp = t.to_screen(problem.depot.0, problem.depot.1);
        painter.rect_filled(
            Rect::from_center_size(dp, Vec2::splat(16.0)),
            2.0,
            Color32::GOLD,
        );
        painter.rect_stroke(
            Rect::from_center_size(dp, Vec2::splat(16.0)),
            2.0,
            Stroke::new(2.0, Color32::WHITE),
        );
        painter.text(
            dp + Vec2::new(0.0, 18.0),
            egui::Align2::CENTER_TOP,
            "Dépôt",
            egui::FontId::proportional(11.0),
            Color32::GOLD,
        );

        // legend
        let lo = available.min + Vec2::new(8.0, 8.0);
        for (ri, _) in solution.routes.iter().enumerate() {
            let y = lo.y + ri as f32 * 16.0;
            if y > available.max.y - 20.0 {
                break;
            }
            painter.line_segment(
                [Pos2::new(lo.x, y), Pos2::new(lo.x + 20.0, y)],
                Stroke::new(2.5, route_color(ri)),
            );
            painter.text(
                Pos2::new(lo.x + 24.0, y),
                egui::Align2::LEFT_CENTER,
                format!("Route {}", ri + 1),
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
    }
}

impl eframe::App for VrpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.running {
            let finished = self
                .solver
                .as_ref()
                .map(|s| s.is_finished())
                .unwrap_or(true);
            if finished {
                self.running = false;
            } else if let (Some(solver), Some(problem)) = (&mut self.solver, &self.problem) {
                solver.step(problem, self.steps_per_frame);
                ctx.request_repaint();
            }
        }

        egui::SidePanel::left("controls")
            .min_width(250.0)
            .show(ctx, |ui| {
                self.ui_controls(ui, ctx);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui_canvas(ui);
        });
    }
}
