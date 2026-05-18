use std::collections::HashSet;

pub struct ServerState {
    clients: HashSet<String>,
    messages: Vec<String>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            clients: HashSet::new(),
            messages: Vec::new(),
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

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn messages(&self) -> &[String] {
        &self.messages
    }
}
