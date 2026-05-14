use std::collections::HashSet;

pub struct ServerState {
    clients: HashSet<String>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            clients: HashSet::new(),
        }
    }

    pub fn add_client(&mut self, addr: String) {
        self.clients.insert(addr);
    }

    pub fn remove_client(&mut self, addr: &str) {
        self.clients.remove(addr);
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}
