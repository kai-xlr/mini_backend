use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::events::{ChatEvent, RecordedEvent};

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

    /// Record a live event. Returns (sequence_id, timestamp).
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

    // --- Replay helpers (used by replay module) ---

    /// Clear messages and events before reconstruction.
    pub(crate) fn reset_for_replay(&mut self) {
        self.messages.clear();
        self.events.clear();
    }

    /// Push a single reconstructed event (does NOT add to messages).
    pub(crate) fn record_reconstructed(&mut self, sequence_id: u64, timestamp: u64, event: ChatEvent) {
        self.events.push(RecordedEvent {
            sequence_id,
            timestamp,
            event,
        });
    }

    /// Restore next_seq from the max sequence_id in the event log.
    pub(crate) fn finalize_replay(&mut self) {
        if let Some(max_seq) = self.events.iter().map(|e| e.sequence_id).max() {
            self.next_seq = max_seq + 1;
        }
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
        let (seq1, t1) = s.record_event(ChatEvent::ClientConnected("addr".into()));
        let (seq2, t2) = s.record_event(ChatEvent::ClientDisconnected("addr".into()));
        assert_eq!(s.events().len(), 2);
        assert_eq!(seq1, 0);
        assert_eq!(seq2, 1);
        assert!(t1 <= t2);
    }

    #[test]
    fn test_replay_helpers_clear_and_rebuild() {
        let mut s = ServerState::new();
        s.add_message("live".into());
        s.record_event(ChatEvent::ClientConnected("live".into()));

        assert_eq!(s.messages().len(), 1);
        assert_eq!(s.events().len(), 1);

        s.reset_for_replay();
        assert!(s.messages().is_empty());
        assert!(s.events().is_empty());

        s.record_reconstructed(0, 100, ChatEvent::ClientConnected("a".into()));
        s.record_reconstructed(1, 200, ChatEvent::MessageBroadcast("hi".into()));
        assert_eq!(s.events().len(), 2);

        s.finalize_replay();
        let (seq, _) = s.record_event(ChatEvent::ClientConnected("b".into()));
        assert_eq!(seq, 2);
    }
}
