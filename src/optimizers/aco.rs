use rand::Rng;
use std::any::Any;
use std::collections::HashSet;

use crate::optimizers::{OptimizationAlgorithm, OptimizerDescriptor};
use crate::problem::{Float, Problem, Solution};

// ── Paramètres ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AcoParams {
    pub n_ants: usize,
    pub alpha: f64, //phéromones
    pub beta: f64,  //heuristique
    pub rho: f64,   //tx d'évaporation
    pub q: f64,     //qté de phéromones
    pub tau_min: f64,
    pub tau_max: f64,
    pub max_iterations: usize,
    pub k_candidates: usize, //taille liste par noeuds
    pub use_two_opt: bool,
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
            k_candidates: 10,
            use_two_opt: true,
        }
    }
}

// ── Algorithme ────────────────────────────────────────────────────────────────

pub struct AcoAlgorithm {
    params: AcoParams,
    pheromones: Vec<Vec<f64>>,
    candidate_list: Vec<Vec<usize>>,
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
        let n = problem.clients.len() + 1;

        let nn_dist = initial_solution.total_distance(problem).max(1e-9);
        let tau_0 = (1.0 / (n as f64 * nn_dist)).clamp(params.tau_min, params.tau_max);
        let pheromones = vec![vec![tau_0; n]; n];

        let k = params.k_candidates.min(n - 1);
        let candidate_list = Self::build_candidate_list(problem, n, k);

        let dist = initial_solution.total_distance(problem);
        Self {
            params,
            pheromones,
            candidate_list,
            current_solution: initial_solution.clone(),
            best_solution: initial_solution.clone(),
            best_distance: dist,
            iteration: 0,
            time_into_account,
        }
    }

    // ── Pré-calcul des candidate lists ────────────────────────────────────

    fn build_candidate_list(problem: &Problem, n: usize, k: usize) -> Vec<Vec<usize>> {
        (0..n)
            .map(|node| {
                let mut dists: Vec<(usize, Float)> = (0..n)
                    .filter(|&j| j != node)
                    .map(|j| (j, Self::node_dist_static(problem, node, j)))
                    .collect();
                dists.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                dists.iter().take(k).map(|&(j, _)| j).collect()
            })
            .collect()
    }

    /// Distance entre deux nœuds
    fn node_dist_static(problem: &Problem, a: usize, b: usize) -> Float {
        let pa = if a == 0 {
            (&problem.repo.x, &problem.repo.y)
        } else {
            (&problem.clients[a - 1].x, &problem.clients[a - 1].y)
        };
        let pb = if b == 0 {
            (&problem.repo.x, &problem.repo.y)
        } else {
            (&problem.clients[b - 1].x, &problem.clients[b - 1].y)
        };
        let dx = *pa.0 as i64 - *pb.0 as i64;
        let dy = *pa.1 as i64 - *pb.1 as i64;
        ((dx * dx + dy * dy) as Float).sqrt()
    }

    fn construct_solution(&self, problem: &Problem) -> Solution {
        let mut rng = rand::thread_rng();
        let n_clients = problem.clients.len();
        let mut unvisited: HashSet<usize> = (0..n_clients).collect();
        let mut routes: Vec<Vec<usize>> = Vec::new();

        while !unvisited.is_empty() {
            let mut route: Vec<usize> = Vec::new();
            let mut current_load: u32 = 0;
            let mut current_node: usize = 0; // 0 = dépôt
            let mut current_time: Float = problem.repo.ready_time as Float;

            loop {
                let candidates = self.feasible_candidates_from(
                    problem,
                    current_node,
                    current_load,
                    current_time,
                    &unvisited,
                );

                if candidates.is_empty() {
                    break;
                }

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

                let client = &problem.clients[chosen];
                let travel = self.dist_from_node(problem, current_node, chosen);
                current_time += travel;
                current_time = current_time.max(client.ready_time as Float);
                current_time += client.service as Float;
                current_load += client.demand;

                route.push(chosen);
                unvisited.remove(&chosen);
                current_node = chosen + 1;
            }

            if !route.is_empty() {
                routes.push(route);
            } else {
                let forced = *unvisited.iter().next().unwrap();
                unvisited.remove(&forced);
                routes.push(vec![forced]);
            }
        }

        Solution { routes }
    }

    fn feasible_candidates_from(
        &self,
        problem: &Problem,
        current_node: usize,
        current_load: u32,
        current_time: Float,
        unvisited: &HashSet<usize>,
    ) -> Vec<usize> {
        let restricted: Vec<usize> = self.candidate_list[current_node]
            .iter()
            // Les candidats sont en indice phéromone ; client i = phéromone i+1
            .filter_map(|&ph| if ph == 0 { None } else { Some(ph - 1) })
            .filter(|ci| unvisited.contains(ci))
            .filter(|&ci| {
                self.is_feasible_next(problem, current_node, ci, current_load, current_time)
            })
            .collect();

        if !restricted.is_empty() {
            return restricted;
        }

        unvisited
            .iter()
            .copied()
            .filter(|&ci| {
                self.is_feasible_next(problem, current_node, ci, current_load, current_time)
            })
            .collect()
    }

    fn is_feasible_next(
        &self,
        problem: &Problem,
        current_node: usize,
        next_client: usize,
        current_load: u32,
        current_time: Float,
    ) -> bool {
        let client = &problem.clients[next_client];

        if current_load + client.demand > problem.max_capacity {
            return false;
        }

        if !self.time_into_account {
            return true;
        }

        let travel = self.dist_from_node(problem, current_node, next_client);
        let arrival = current_time + travel;
        if arrival > client.due_time as Float {
            return false;
        }

        let depart = arrival.max(client.ready_time as Float) + client.service as Float;
        let back_to_depot = depart + Problem::dist(client, &problem.repo);
        back_to_depot <= problem.repo.due_time as Float
    }

    fn dist_from_node(&self, problem: &Problem, node: usize, client_idx: usize) -> Float {
        let client = &problem.clients[client_idx];
        if node == 0 {
            Problem::dist(&problem.repo, client)
        } else {
            Problem::dist(&problem.clients[node - 1], client)
        }
    }

    fn two_opt(&self, problem: &Problem, solution: &mut Solution) {
        for route in &mut solution.routes {
            let n = route.len();
            if n < 4 {
                continue;
            }
            let mut improved = true;
            while improved {
                improved = false;
                'outer: for i in 0..n - 1 {
                    for j in i + 2..n {
                        let a = if i == 0 { 0 } else { route[i - 1] + 1 }; // nœud phéromone avant i
                        let b = route[i] + 1;
                        let c = route[j] + 1;
                        let d = if j + 1 < n { route[j + 1] + 1 } else { 0 };

                        let dist_before = Self::node_dist_static(problem, a, b)
                            + Self::node_dist_static(problem, c, d);
                        let dist_after = Self::node_dist_static(problem, a, c)
                            + Self::node_dist_static(problem, b, d);

                        if dist_after < dist_before - 1e-9 {
                            route[i..=j].reverse();
                            improved = true;
                            break 'outer;
                        }
                    }
                }
            }
        }
    }

    fn update_pheromones_mmas(&mut self, problem: &Problem, best_iter: &Solution) {
        let rho = self.params.rho;
        let tau_min = self.params.tau_min;
        let tau_max = self.params.tau_max;

        // Évaporation
        for row in &mut self.pheromones {
            for cell in row.iter_mut() {
                *cell = (*cell * (1.0 - rho)).clamp(tau_min, tau_max);
            }
        }

        // Dépôt : on alterne entre best_iter et best_global pour diversification
        let depositor = if self.iteration % 5 == 0 {
            &self.best_solution.clone()
        } else {
            best_iter
        };

        let dist = depositor.total_distance(problem);
        if dist < 1e-9 {
            return;
        }
        let delta = self.params.q / dist;

        for route in &depositor.routes {
            if let Some(&first) = route.first() {
                self.pheromones[0][first + 1] =
                    (self.pheromones[0][first + 1] + delta).clamp(tau_min, tau_max);
            }
            for w in route.windows(2) {
                let (a, b) = (w[0] + 1, w[1] + 1);
                self.pheromones[a][b] = (self.pheromones[a][b] + delta).clamp(tau_min, tau_max);
            }
            if let Some(&last) = route.last() {
                self.pheromones[last + 1][0] =
                    (self.pheromones[last + 1][0] + delta).clamp(tau_min, tau_max);
            }
        }
    }
}

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

            let solutions: Vec<Solution> = (0..self.params.n_ants)
                .map(|_| {
                    let mut sol = self.construct_solution(problem);
                    if self.params.use_two_opt {
                        self.two_opt(problem, &mut sol);
                    }
                    sol
                })
                .collect();

            let best_iter = solutions
                .iter()
                .min_by(|a, b| {
                    a.total_distance(problem)
                        .partial_cmp(&b.total_distance(problem))
                        .unwrap()
                })
                .unwrap()
                .clone();

            self.update_pheromones_mmas(problem, &best_iter);

            let best_iter_dist = best_iter.total_distance(problem);
            if best_iter_dist < self.best_distance {
                self.best_distance = best_iter_dist;
                self.best_solution = best_iter.clone();
            }

            self.current_solution = best_iter;
            self.iteration += 1;
        }
    }

    fn is_finished(&self) -> bool {
        self.iteration >= self.params.max_iterations
    }
}

fn draw_aco_params_ui(params: &mut dyn Any, ui: &mut egui::Ui) {
    let params = params.downcast_mut::<AcoParams>().unwrap();
    ui.add(egui::Slider::new(&mut params.n_ants, 5..=100).text("Fourmis"));
    ui.add(egui::Slider::new(&mut params.alpha, 0.1..=5.0).text("Alpha (phéromones)"));
    ui.add(egui::Slider::new(&mut params.beta, 0.1..=5.0).text("Beta (heuristique)"));
    ui.add(egui::Slider::new(&mut params.rho, 0.01..=0.5).text("Rho (évaporation)"));
    ui.add(egui::Slider::new(&mut params.q, 1.0..=1000.0).text("Q (dépôt)"));
    ui.add(egui::Slider::new(&mut params.k_candidates, 3..=20).text("K candidats"));
    ui.add(egui::Checkbox::new(
        &mut params.use_two_opt,
        "2-opt local search",
    ));
    ui.add(egui::Slider::new(&mut params.max_iterations, 50..=2000).text("Itérations"));
}

pub fn build_algorithm(
    problem: &Problem,
    solution: &Solution,
    params: &dyn Any,
    time_into_account: bool,
) -> Box<dyn OptimizationAlgorithm + Send + Sync> {
    let params = params.downcast_ref::<AcoParams>().unwrap().clone();
    Box::new(AcoAlgorithm::new(
        problem,
        solution,
        params,
        time_into_account,
    ))
}

inventory::submit!(OptimizerDescriptor {
    id: "aco",
    label: "Ant Colony Optimization",
    create_default_params: || Box::new(AcoParams::default()),
    draw_params_ui: draw_aco_params_ui,
    build_algorithm: build_algorithm,
});
