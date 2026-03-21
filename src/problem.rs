use crate::parser::{Client, InputData, Int, Localizable, Repository};
use rand::prelude::SliceRandom;

pub type Float = f64;

#[derive(Debug, Clone)]
pub struct Problem {
    pub clients: Vec<Client>,
    pub(crate) repo: Repository,
    pub max_capacity: Int,
}

impl Problem {
    pub fn new(input: InputData) -> Self {
        let clients = input.clients;
        if input.repositories.is_empty() {
            panic!("No repository detected. Aborting.")
        }
        let repo = input.repositories[0].clone();
        let max_capacity = input.max_quantity;
        Self {
            clients,
            repo,
            max_capacity,
        }
    }

    #[inline]
    /// Computes the Euclidean distance between two Localizable objects (Client or Repository)
    pub fn dist(client1: &impl Localizable, client2: &impl Localizable) -> Float {
        let (x1, y1) = client1.coords();
        let (x2, y2) = client2.coords();
        (((x1 - x2).pow(2) + (y1 - y2).pow(2)) as f64).sqrt() as Float
    }

    pub fn route_distance(&self, route: &[usize]) -> f64 {
        if route.is_empty() {
            return 0.0;
        }
        let first = &self.clients[route[0]];
        let last = &self.clients[*route.last().unwrap()];

        let mut d = Problem::dist(&self.repo, first); // from the repo to the first client

        // between each pair of consecutive clients
        for w in route.windows(2) {
            let a = &self.clients[w[0]];
            let b = &self.clients[w[1]];
            d += Problem::dist(a, b);
        }

        d += Problem::dist(last, &self.repo); // from the last client to the repo
        d
    }
}

#[derive(Debug, Clone)]
pub struct Solution {
    pub routes: Vec<Vec<usize>>, // each route is a Vec of client indices
}

impl Solution {
    pub fn total_distance(&self, problem: &Problem) -> f64 {
        self.routes
            .iter()
            .map(|route| problem.route_distance(route))
            .sum()
    }

    pub fn random(problem: &Problem) -> Self {
        let max_capacity_random = (problem.max_capacity as f64) * 0.8;
        let mut current_capacity = max_capacity_random;

        let mut routes = Vec::new();
        let mut current_route = Vec::new();
        let mut clients: Vec<(usize, &Client)> = problem.clients.iter().enumerate().collect();
        clients.shuffle(&mut rand::thread_rng());

        for (i, client) in clients {
            let demand = client.demand;
            // if there is enough capacity, add the client to the current route
            if current_capacity - demand as f64 >= 0.0 {
                current_route.push(i);
                current_capacity -= demand as f64;
            } else {
                // else, publish the current route, start a new one
                routes.push(current_route);
                current_route = vec![i];
                current_capacity = max_capacity_random - demand as f64;
            }
        }

        if !current_route.is_empty() {
            routes.push(current_route);
        }

        Self { routes }
    }
}
