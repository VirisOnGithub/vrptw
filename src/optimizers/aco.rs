use rand::Rng;
use std::any::Any;

use crate::optimizers::{OptimizationAlgorithm, OptimizerDescriptor};
use crate::problem::{Float, Problem, Solution};

// ── Paramètres ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AcoParams {
    pub n_ants: usize, // nombre de fourmis par itération
    pub alpha: f64,    // poids des phéromones
    pub beta: f64,     // poids de l'heuristique (1/distance)
    pub rho: f64,      // taux d'évaporation [0, 1]
    pub q: f64,        // constante de dépôt de phéromones
    pub tau_min: f64,  // phéromone minimale (évite la stagnation)
    pub tau_max: f64,  // phéromone maximale
    pub max_iterations: usize,
}

impl Default for AcoParams {
    fn default() -> Self {
        Self {
            n_ants: 20,
            alpha: 1.0,
            beta: 2.5,
            rho: 0.1,
            q: 100.0,
            tau_min: 0.01,
            tau_max: 10.0,
            max_iterations: 500,
        }
    }
}

// ── Algorithme ────────────────────────────────────────────────────────────────

pub struct AcoAlgorithm {
    params: AcoParams,
    pheromones: Vec<Vec<f64>>, // [n+1][n+1], index 0 = dépôt
    current_solution: Solution,
    best_solution: Solution,
    best_distance: f64,
    iteration: usize,
    time_into_account: bool,
}

impl AcoAlgorithm {
    pub fn new(
        problem: &Problem,
        initial_solution: &Solution,
        params: AcoParams,
        time_into_account: bool,
    ) -> Self {
        let n = problem.clients.len() + 1; // +1 pour le dépôt (index 0)
        let pheromones = vec![vec![params.tau_max / 2.0; n]; n];
        let dist = initial_solution.total_distance(problem);

        Self {
            params,
            pheromones,
            current_solution: initial_solution.clone(),
            best_solution: initial_solution.clone(),
            best_distance: dist,
            iteration: 0,
            time_into_account,
        }
    }

    /// Construit une solution pour une fourmi.
    /// Les indices clients sont en base 1 dans la phéromone (0 = dépôt).
    fn construct_solution(&self, problem: &Problem) -> Solution {
        let mut rng = rand::thread_rng();
        let n_clients = problem.clients.len();
        let mut unvisited: Vec<usize> = (0..n_clients).collect();
        let mut routes: Vec<Vec<usize>> = Vec::new();

        while !unvisited.is_empty() {
            let mut route: Vec<usize> = Vec::new();
            let mut current_load: u32 = 0;
            // current_node : index dans pheromones (0 = dépôt, i+1 = client i)
            let mut current_node: usize = 0;
            let mut current_time: Float = problem.repo.ready_time as Float;

            loop {
                // Candidats faisables depuis la position courante
                let candidates: Vec<usize> = unvisited
                    .iter()
                    .copied()
                    .filter(|&ci| {
                        self.is_feasible_next(problem, current_node, ci, current_load, current_time)
                    })
                    .collect();

                if candidates.is_empty() {
                    break;
                }

                // Calcul des poids (règle ACS/AS)
                let weights: Vec<f64> = candidates
                    .iter()
                    .map(|&ci| {
                        let tau = self.pheromones[current_node][ci + 1];
                        let dist = self.dist_from_node(problem, current_node, ci);
                        let eta = if dist > 1e-9 { 1.0 / dist } else { 1e9 };
                        tau.powf(self.params.alpha) * eta.powf(self.params.beta)
                    })
                    .collect();

                let total: f64 = weights.iter().sum();
                let mut pick = rng.gen_range(0.0..total);
                let chosen = candidates
                    .iter()
                    .zip(weights.iter())
                    .find(|(_, w)| {
                        pick -= *w;
                        pick <= 0.0
                    })
                    .map(|(&ci, _)| ci)
                    .unwrap_or(*candidates.last().unwrap());

                // Mise à jour état courant
                let client = &problem.clients[chosen];
                let travel = self.dist_from_node(problem, current_node, chosen);
                current_time += travel;
                current_time = current_time.max(client.ready_time as Float);
                current_time += client.service as Float;
                current_load += client.demand;

                route.push(chosen);
                unvisited.retain(|&x| x != chosen);
                current_node = chosen + 1; // +1 car 0 est le dépôt
            }

            if !route.is_empty() {
                routes.push(route);
            } else {
                // Sécurité : si aucun client insérable, forcer le premier non visité
                // dans une route dédiée (ne devrait pas arriver si les données sont saines)
                let forced = unvisited.remove(0);
                routes.push(vec![forced]);
            }
        }

        Solution { routes }
    }

    /// Vérifie si l'on peut visiter `next_client` depuis `current_node`
    /// sans violer capacité ni (si activées) fenêtres de temps.
    fn is_feasible_next(
        &self,
        problem: &Problem,
        current_node: usize, // 0 = dépôt
        next_client: usize,
        current_load: u32,
        current_time: Float,
    ) -> bool {
        let client = &problem.clients[next_client];

        // Contrainte capacité
        if current_load + client.demand > problem.max_capacity {
            return false;
        }

        if !self.time_into_account {
            return true;
        }

        // Contrainte fenêtre de temps du client
        let travel = self.dist_from_node(problem, current_node, next_client);
        let arrival = current_time + travel;
        if arrival > client.due_time as Float {
            return false;
        }

        // Contrainte retour dépôt après service
        let depart = arrival.max(client.ready_time as Float) + client.service as Float;
        let back_to_depot = depart + Problem::dist(client, &problem.repo);
        if back_to_depot > problem.repo.due_time as Float {
            return false;
        }

        true
    }

    /// Distance depuis un nœud phéromone (0 = dépôt) vers un client.
    fn dist_from_node(&self, problem: &Problem, node: usize, client_idx: usize) -> Float {
        let client = &problem.clients[client_idx];
        if node == 0 {
            Problem::dist(&problem.repo, client)
        } else {
            Problem::dist(&problem.clients[node - 1], client)
        }
    }

    /// Évaporation + dépôt de phéromones.
    fn update_pheromones(&mut self, problem: &Problem, solutions: &[Solution]) {
        let rho = self.params.rho;
        let tau_min = self.params.tau_min;
        let tau_max = self.params.tau_max;

        // Évaporation globale
        for row in &mut self.pheromones {
            for cell in row.iter_mut() {
                *cell = (*cell * (1.0 - rho)).clamp(tau_min, tau_max);
            }
        }

        // Dépôt proportionnel à la qualité
        for sol in solutions {
            let dist = sol.total_distance(problem);
            if dist < 1e-9 {
                continue;
            }
            let delta = self.params.q / dist;

            for route in &sol.routes {
                // Dépôt → premier client
                if let Some(&first) = route.first() {
                    self.pheromones[0][first + 1] =
                        (self.pheromones[0][first + 1] + delta).clamp(tau_min, tau_max);
                }
                // Arcs entre clients
                for w in route.windows(2) {
                    let (a, b) = (w[0] + 1, w[1] + 1);
                    self.pheromones[a][b] = (self.pheromones[a][b] + delta).clamp(tau_min, tau_max);
                }
                // Dernier client → dépôt
                if let Some(&last) = route.last() {
                    self.pheromones[last + 1][0] =
                        (self.pheromones[last + 1][0] + delta).clamp(tau_min, tau_max);
                }
            }
        }
    }
}

// ── Trait OptimizationAlgorithm ───────────────────────────────────────────────

impl OptimizationAlgorithm for AcoAlgorithm {
    fn total_iterations(&self) -> usize {
        self.params.max_iterations
    }

    fn current_solution(&self) -> &Solution {
        &self.best_solution
    }

    fn step(&mut self, problem: &Problem, nb_steps: usize) {
        for _ in 0..nb_steps {
            if self.is_finished() {
                break;
            }

            // Chaque fourmi construit une solution
            let solutions: Vec<Solution> = (0..self.params.n_ants)
                .map(|_| self.construct_solution(problem))
                .collect();

            // Mise à jour des phéromones
            self.update_pheromones(problem, &solutions);

            // Mise à jour de la meilleure solution
            for sol in &solutions {
                let d = sol.total_distance(problem);
                if d < self.best_distance {
                    self.best_distance = d;
                    self.best_solution = sol.clone();
                }
            }

            // Solution courante = meilleure fourmi de cette itération
            if let Some(best_ant) = solutions.iter().min_by(|a, b| {
                a.total_distance(problem)
                    .partial_cmp(&b.total_distance(problem))
                    .unwrap()
            }) {
                self.current_solution = best_ant.clone();
            }

            self.iteration += 1;
        }
    }

    fn is_finished(&self) -> bool {
        self.iteration >= self.params.max_iterations
    }
}

// ── OptimizerDescriptor (enregistrement via inventory) ───────────────────────

fn draw_aco_params_ui(params: &mut dyn Any, ui: &mut egui::Ui) {
    let params = params.downcast_mut::<AcoParams>().unwrap();
    ui.add(egui::Slider::new(&mut params.n_ants, 5..=100).text("Fourmis"));
    ui.add(egui::Slider::new(&mut params.alpha, 0.1..=5.0).text("Alpha (phéromones)"));
    ui.add(egui::Slider::new(&mut params.beta, 0.1..=5.0).text("Beta (heuristique)"));
    ui.add(egui::Slider::new(&mut params.rho, 0.01..=0.5).text("Rho (évaporation)"));
    ui.add(egui::Slider::new(&mut params.q, 1.0..=1000.0).text("Q (dépôt)"));
    ui.add(egui::Slider::new(&mut params.max_iterations, 50..=2000).text("Itérations"));
}

inventory::submit!(OptimizerDescriptor {
    id: "aco",
    label: "Ant Colony Optimization",
    create_default_params: || Box::new(AcoParams::default()),
    draw_params_ui: draw_aco_params_ui,
    build_algorithm: |problem, solution, params, time_into_account| {
        let params = params.downcast_ref::<AcoParams>().unwrap().clone();
        Box::new(AcoAlgorithm::new(
            problem,
            solution,
            params,
            time_into_account,
        ))
    },
});
