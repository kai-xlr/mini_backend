use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{ChatEvent, RecordedEvent};

pub struct ServerState {
    clients: HashSet<String>,
    messages: Vec<String>,
    events: Vec<RecordedEvent>,
    next_seq: u64,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            clients: HashSet::new(),
            messages: Vec::new(),
            events: Vec::new(),
            next_seq: 0,
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
    // Returns (sequence_id, timestamp) — sequence is a deterministic monotonic counter
    pub fn record_event(&mut self, event: ChatEvent) -> (u64, u64) {
        let seq = self.next_seq;
        self.next_seq += 1;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.events.push(RecordedEvent {
            sequence_id: seq,
            timestamp,
            event,
        });

        (seq, timestamp)
    }

    pub fn events(&self) -> &[RecordedEvent] {
        &self.events
    }

    /// Full reconstruction: clears messages and events, then rebuilds
    /// both from the stored event log ordered by sequence_id.
    /// MessageBroadcast events reconstruct the messages vec.
    /// Also restores next_seq so new events continue the sequence.
    /// Returns (event_count, msg_count).
    pub fn reconstruct_from_events(
        &mut self,
        stored: Vec<(u64, u64, String, String)>,
    ) -> (usize, usize) {
        self.messages.clear();
        self.events.clear();

        let mut msg_count = 0;

        for (seq, timestamp, event_type, details) in stored {
            if let Some(event) = ChatEvent::from_event_store(&event_type, &details) {
                if let ChatEvent::MessageBroadcast(ref msg) = event {
                    self.messages.push(msg.clone());
                    msg_count += 1;
                }
                self.events.push(RecordedEvent {
                    sequence_id: seq,
                    timestamp,
                    event,
                });
            }
        }

        // Restore next_seq so new events continue the monotonic sequence
        if let Some(max_seq) = self.events.iter().map(|e| e.sequence_id).max() {
            self.next_seq = max_seq + 1;
        }

        (self.events.len(), msg_count)
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
        let (seq1, t1) = s.record_event(ChatEvent::ClientConnected("addr".into()));
        let (seq2, t2) = s.record_event(ChatEvent::ClientDisconnected("addr".into()));
        assert_eq!(s.events().len(), 2);
        assert_eq!(seq1, 0);
        assert_eq!(seq2, 1);
        assert!(t1 <= t2);
    }

    #[test]
    fn test_reconstruct_from_events_rebuilds_messages() {
        let mut s = ServerState::new();
        s.add_message("should-be-cleared".into());

        let stored = vec![
            (0, 10, "ClientConnected".into(), "a".into()),
            (1, 20, "MessageReceived".into(), "192.168.1.1:9999: hi".into()),
            (2, 30, "MessageBroadcast".into(), "192.168.1.1:9999: hi".into()),
            (3, 40, "ClientDisconnected".into(), "a".into()),
        ];

        let (event_count, msg_count) = s.reconstruct_from_events(stored);
        assert_eq!(event_count, 4);
        assert_eq!(msg_count, 1);
        assert_eq!(s.messages().len(), 1);
        assert_eq!(s.messages()[0], "192.168.1.1:9999: hi");
        assert_eq!(s.events().len(), 4);
        // next_seq restored
        let (seq, _) = s.record_event(ChatEvent::ClientConnected("b".into()));
        assert_eq!(seq, 4);
    }

    #[test]
    fn test_replay_order_matters() {
        let mut s1 = ServerState::new();
        let mut s2 = ServerState::new();

        let stored_correct = vec![
            (0, 10, "MessageBroadcast".into(), "first".into()),
            (1, 20, "MessageBroadcast".into(), "second".into()),
        ];
        let stored_wrong = vec![
            (1, 20, "MessageBroadcast".into(), "second".into()),
            (0, 10, "MessageBroadcast".into(), "first".into()),
        ];

        let (_, m1) = s1.reconstruct_from_events(stored_correct);
        let (_, m2) = s2.reconstruct_from_events(stored_wrong);

        assert_eq!(m1, 2);
        assert_eq!(m2, 2);
        assert_eq!(s1.messages()[0], "first");
        assert_eq!(s2.messages()[0], "second");
        assert_ne!(s1.messages(), s2.messages());
    }

    #[test]
    fn test_deterministic_replay() {
        let events = vec![
            (0, 100, "ClientConnected".into(), "alice".into()),
            (1, 200, "MessageReceived".into(), "alice: hey".into()),
            (2, 300, "MessageBroadcast".into(), "alice: hey".into()),
            (3, 400, "ClientDisconnected".into(), "alice".into()),
        ];

        let mut s1 = ServerState::new();
        let mut s2 = ServerState::new();

        s1.reconstruct_from_events(events.clone());
        s2.reconstruct_from_events(events);

        assert_eq!(s1.messages(), s2.messages());
        assert_eq!(s1.events().len(), s2.events().len());
        assert_eq!(
            s1.events().iter().map(|e| e.sequence_id).collect::<Vec<_>>(),
            s2.events().iter().map(|e| e.sequence_id).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn test_sequence_ids_are_monotonic() {
        let mut s = ServerState::new();

        for i in 0..5u64 {
            let (seq, _) =
                s.record_event(ChatEvent::ClientConnected(format!("client_{}", i)));
            assert_eq!(seq, i);
        }

        // Next seq continues after replay
        let stored = s
            .events()
            .iter()
            .map(|e| (e.sequence_id, e.timestamp, e.event.event_type().to_string(), e.event.details()))
            .collect::<Vec<_>>();

        let mut restored = ServerState::new();
        restored.reconstruct_from_events(stored);

        let (seq, _) = restored.record_event(ChatEvent::ClientConnected("new".into()));
        assert_eq!(seq, 5);
    }
}
