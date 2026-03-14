pub type INT = u32;

pub trait Localizable {
    fn coords(&self) -> (INT, INT);
    fn id(&self) -> &str;
}

#[derive(Debug)]
pub struct InputData {
    pub name: String,
    pub graph_type: String,
    pub coordinates_type: String,
    pub repositories_nb: INT,
    pub clients_nb: INT,
    pub max_quantity: INT,
    pub repositories: Vec<Repository>,
    pub clients: Vec<Client>,
}

#[derive(Debug, Clone)]
pub struct Repository {
    pub id: String,
    pub x: INT,
    pub y: INT,
    pub read_time: INT,
    pub due_time: INT,
}

impl Localizable for Repository {
    fn coords(&self) -> (INT, INT) {
        (self.x, self.y)
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    pub id: String,
    pub x: INT,
    pub y: INT,
    pub ready_time: INT,
    pub due_time: INT,
    pub demand: INT,
    pub service: INT,
}

impl Localizable for Client {
    fn coords(&self) -> (INT, INT) {
        (self.x, self.y)
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(PartialEq)]
enum Section {
    Header,
    Depots,
    Clients,
}

impl InputData {
    pub fn new(
        name: String,
        graph_type: String,
        coordinates_type: String,
        repositories_nb: INT,
        clients_nb: INT,
        max_quantity: INT,
        repositories: Vec<Repository>,
        clients: Vec<Client>,
    ) -> Self {
        InputData {
            name,
            graph_type,
            coordinates_type,
            repositories_nb,
            clients_nb,
            max_quantity,
            repositories,
            clients,
        }
    }

    pub fn parse_input(input: &str) -> Self {
        let mut name = String::new();
        let mut graph_type = String::new();
        let mut coordinates_type = String::new();
        let mut repositories_nb: INT = 0;
        let mut clients_nb: INT = 0;
        let mut max_quantity: INT = 0;
        let mut repositories: Vec<Repository> = Vec::new();
        let mut clients: Vec<Client> = Vec::new();

        let mut section = Section::Header;

        for line in input.lines() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            // Detect section transitions
            if line.starts_with("DATA_DEPOTS") {
                section = Section::Depots;
                continue;
            }
            if line.starts_with("DATA_CLIENTS") {
                section = Section::Clients;
                continue;
            }

            match section {
                Section::Header => {
                    if let Some((key, value)) = line.split_once(':') {
                        let key = key.trim();
                        let value = value.trim();
                        match key {
                            "NAME" => name = value.to_string(),
                            "TYPE" => graph_type = value.to_string(),
                            "COORDINATES" => coordinates_type = value.to_string(),
                            "NB_DEPOTS" => repositories_nb = value.parse().unwrap_or(0),
                            "NB_CLIENTS" => clients_nb = value.parse().unwrap_or(0),
                            "MAX_QUANTITY" => max_quantity = value.parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                }
                Section::Depots => {
                    // Format: idName x y readyTime dueTime
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        repositories.push(Repository {
                            id: parts[0].to_string(),
                            x: parts[1].parse().unwrap_or(0),
                            y: parts[2].parse().unwrap_or(0),
                            read_time: parts[3].parse().unwrap_or(0),
                            due_time: parts[4].parse().unwrap_or(0),
                        });
                    }
                }
                Section::Clients => {
                    // Format: idName x y readyTime dueTime demand service
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 6 {
                        clients.push(Client {
                            id: parts[0].to_string(),
                            x: parts[1].parse().unwrap_or(0),
                            y: parts[2].parse().unwrap_or(0),
                            ready_time: parts[3].parse().unwrap_or(0),
                            due_time: parts[4].parse().unwrap_or(0),
                            demand: parts[5].parse().unwrap_or(0),
                            service: parts[6].parse().unwrap_or(0),
                        });
                    }
                }
            }
        }

        InputData {
            name,
            graph_type,
            coordinates_type,
            repositories_nb,
            clients_nb,
            max_quantity,
            repositories,
            clients,
        }
    }
}
