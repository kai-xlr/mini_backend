use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum ChatEvent {
    ClientConnected(String),
    MessageReceived { sender: String, body: String },
    MessageBroadcast(String),
    ClientDisconnected(String),
}

pub struct ServerState {
    clients: HashSet<String>,
    messages: Vec<String>,
    events: Vec<RecordedEvent>,
}

pub struct RecordedEvent {
    pub timestamp: u64,
    pub event: ChatEvent,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            clients: HashSet::new(),
            messages: Vec::new(),
            events: Vec::new(),
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

    pub fn record_event(&mut self, event: ChatEvent) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.events.push(RecordedEvent { timestamp, event });
    }

    pub fn events(&self) -> &[RecordedEvent] {
        &self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state_is_empty() {
        let s = ServerState::new();
        assert_eq!(s.client_count(), 0);
        assert!(s.messages().is_empty());
        assert!(s.events().is_empty());
    }

    #[test]
    fn test_add_remove_client() {
        let mut s = ServerState::new();
        s.add_client("a".into());
        s.add_client("b".into());
        assert_eq!(s.client_count(), 2);
        s.remove_client("a");
        assert_eq!(s.client_count(), 1);
    }

    #[test]
    fn test_add_message() {
        let mut s = ServerState::new();
        s.add_message("hello".into());
        s.add_message("world".into());
        assert_eq!(s.messages().len(), 2);
        assert_eq!(s.messages()[0], "hello");
    }

    #[test]
    fn test_record_event() {
        let mut s = ServerState::new();
        s.record_event(ChatEvent::ClientConnected("addr".into()));
        s.record_event(ChatEvent::ClientDisconnected("addr".into()));
        assert_eq!(s.events().len(), 2);
    }
}
