use crate::parser::InputData;

#[derive(Clone, Debug)]
pub struct ClientNode {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub demand: u32,
}

#[derive(Clone, Debug)]
pub struct Problem {
    #[allow(dead_code)]
    pub name: String,
    pub depot: (f64, f64),
    pub clients: Vec<ClientNode>, // 0-indexed
    pub capacity: u32,
}

#[derive(Clone, Debug)]
pub struct Solution {
    pub routes: Vec<Vec<usize>>, // each route: 0-based indices into problem.clients
}

/// Basic
impl Problem {
    pub fn from_input(data: &InputData) -> Self {
        let depot = &data.repositories[0];
        Problem {
            name: data.name.clone(),
            depot: (depot.x as f64, depot.y as f64),
            clients: data
                .clients
                .iter()
                .map(|c| ClientNode {
                    id: c.id.clone(),
                    x: c.x as f64,
                    y: c.y as f64,
                    demand: c.demand,
                })
                .collect(),
            capacity: data.max_quantity,
        }
    }

    #[inline]
    pub fn dist(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
        ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
    }

    pub fn route_distance(&self, route: &[usize]) -> f64 {
        if route.is_empty() {
            return 0.0;
        }
        let (dx, dy) = self.depot;
        let first = &self.clients[route[0]];
        let last = &self.clients[*route.last().unwrap()];

        let mut d = Problem::dist(dx, dy, first.x, first.y);
        for w in route.windows(2) {
            let a = &self.clients[w[0]];
            let b = &self.clients[w[1]];
            d += Problem::dist(a.x, a.y, b.x, b.y);
        }
        d += Problem::dist(last.x, last.y, dx, dy);
        d
    }

    pub fn route_demand(&self, route: &[usize]) -> u32 {
        route.iter().map(|&i| self.clients[i].demand).sum()
    }
}

/// Solve using basic methods, used for initial solution generation
impl Solution {
    pub fn total_distance(&self, problem: &Problem) -> f64 {
        self.routes.iter().map(|r| problem.route_distance(r)).sum()
    }

    #[allow(dead_code)]
    /// Used only for initial solution generation, not as bad as random ones
    pub fn greedy(problem: &Problem) -> Self {
        let n = problem.clients.len();
        let mut visited = vec![false; n];
        let mut routes: Vec<Vec<usize>> = Vec::new();

        while visited.iter().any(|&v| !v) {
            let mut route = Vec::new();
            let mut cap_left = problem.capacity;
            let (mut cx, mut cy) = problem.depot;

            loop {
                let next = (0..n)
                    .filter(|&i| !visited[i] && problem.clients[i].demand <= cap_left)
                    .min_by(|&a, &b| {
                        let da = Problem::dist(cx, cy, problem.clients[a].x, problem.clients[a].y);
                        let db = Problem::dist(cx, cy, problem.clients[b].x, problem.clients[b].y);
                        da.partial_cmp(&db).unwrap()
                    });

                match next {
                    Some(i) => {
                        visited[i] = true;
                        cap_left -= problem.clients[i].demand;
                        cx = problem.clients[i].x;
                        cy = problem.clients[i].y;
                        route.push(i);
                    }
                    None => break,
                }
            }

            if !route.is_empty() {
                routes.push(route);
            }
        }

        Solution { routes }
    }

    pub fn random(problem: &Problem) -> Self {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let mut clients: Vec<usize> = (0..problem.clients.len()).collect();
        clients.shuffle(&mut rng);

        let mut routes: Vec<Vec<usize>> = Vec::new();
        let mut current_route: Vec<usize> = Vec::new();
        let mut cap_left = problem.capacity;

        for i in clients {
            let demand = problem.clients[i].demand;
            if demand as f32 <= cap_left as f32 * 0.6 {
                current_route.push(i);
                cap_left -= demand;
            } else {
                if !current_route.is_empty() {
                    routes.push(current_route);
                }
                current_route = vec![i];
                cap_left = problem.capacity - demand;
            }
        }

        if !current_route.is_empty() {
            routes.push(current_route);
        }

        Solution { routes }
    }
}
