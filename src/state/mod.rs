use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{ChatEvent, RecordedEvent};

pub struct ServerState {
    clients: HashSet<String>,
    messages: Vec<String>,
    events: Vec<RecordedEvent>,
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

    // Assignment 4: Centralized timestamp generation that returns the value
    pub fn record_event(&mut self, event: ChatEvent) -> u64 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.events.push(RecordedEvent { timestamp, event });

        timestamp
    }

    pub fn events(&self) -> &[RecordedEvent] {
        &self.events
    }

    /// Append-only replay: populates events vec from stored data.
    /// Does NOT touch messages or clients.
    /// Returns the number of events replayed.
    #[allow(dead_code)]
    pub fn replay_events(&mut self, stored: Vec<(u64, String, String)>) -> usize {
        let mut count = 0;
        for (timestamp, event_type, details) in stored {
            if let Some(event) = ChatEvent::from_event_store(&event_type, &details) {
                self.events.push(RecordedEvent { timestamp, event });
                count += 1;
            }
        }
        count
    }

    /// Full reconstruction: clears messages and events, then rebuilds
    /// both from the stored event log. MessageBroadcast events are
    /// used to reconstruct the messages vec. Returns (event_count, msg_count).
    pub fn reconstruct_from_events(
        &mut self,
        stored: Vec<(u64, String, String)>,
    ) -> (usize, usize) {
        self.messages.clear();
        self.events.clear();

        let mut event_count = 0;
        let mut msg_count = 0;

        for (timestamp, event_type, details) in stored {
            if let Some(event) = ChatEvent::from_event_store(&event_type, &details) {
                if let ChatEvent::MessageBroadcast(ref msg) = event {
                    self.messages.push(msg.clone());
                    msg_count += 1;
                }
                self.events.push(RecordedEvent { timestamp, event });
                event_count += 1;
            }
        }

        (event_count, msg_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ChatEvent;

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
        let t1 = s.record_event(ChatEvent::ClientConnected("addr".into()));
        let t2 = s.record_event(ChatEvent::ClientDisconnected("addr".into()));
        assert_eq!(s.events().len(), 2);
        assert!(t1 <= t2); // Quick validation on returned type
    }

    #[test]
    fn test_replay_events_populates_events_only() {
        let mut s = ServerState::new();
        s.add_message("existing".into());

        let stored = vec![
            (10, "ClientConnected".into(), "a".into()),
            (20, "MessageBroadcast".into(), "hello".into()),
            (30, "ClientDisconnected".into(), "a".into()),
        ];

        let count = s.replay_events(stored);
        assert_eq!(count, 3);
        assert_eq!(s.events().len(), 3);
        // Existing messages preserved
        assert_eq!(s.messages().len(), 1);
        assert_eq!(s.messages()[0], "existing");
    }

    #[test]
    fn test_reconstruct_from_events_rebuilds_messages() {
        let mut s = ServerState::new();
        s.add_message("should-be-cleared".into());

        let stored = vec![
            (10, "ClientConnected".into(), "a".into()),
            (20, "MessageReceived".into(), "192.168.1.1:9999: hi".into()),
            (30, "MessageBroadcast".into(), "192.168.1.1:9999: hi".into()),
            (40, "ClientDisconnected".into(), "a".into()),
        ];

        let (event_count, msg_count) = s.reconstruct_from_events(stored);
        assert_eq!(event_count, 4);
        assert_eq!(msg_count, 1);
        // Old messages cleared
        assert_eq!(s.messages().len(), 1);
        assert_eq!(s.messages()[0], "192.168.1.1:9999: hi");
        assert_eq!(s.events().len(), 4);
    }

    #[test]
    fn test_replay_order_matters() {
        // Verify that replay order affects reconstructed state
        let mut s1 = ServerState::new();
        let mut s2 = ServerState::new();

        let stored_correct = vec![
            (10, "MessageBroadcast".into(), "first".into()),
            (20, "MessageBroadcast".into(), "second".into()),
        ];
        let stored_wrong = vec![
            (10, "MessageBroadcast".into(), "second".into()),
            (20, "MessageBroadcast".into(), "first".into()),
        ];

        let (_, m1) = s1.reconstruct_from_events(stored_correct);
        let (_, m2) = s2.reconstruct_from_events(stored_wrong);

        assert_eq!(m1, 2);
        assert_eq!(m2, 2);
        // Order differs
        assert_eq!(s1.messages()[0], "first");
        assert_eq!(s2.messages()[0], "second");
        assert_ne!(s1.messages(), s2.messages());
    }
}
