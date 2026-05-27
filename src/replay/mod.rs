use rusqlite::Connection;

use crate::events::ChatEvent;
use crate::state::ServerState;
use crate::storage;

/// Full reconstruction: clears state, then rebuilds messages and events
/// from the stored event log ordered by sequence_id.
/// MessageBroadcast events reconstruct the messages vec.
/// next_seq is restored so new events continue the monotonic sequence.
/// Returns (event_count, msg_count).
pub fn reconstruct_from_events(
    state: &mut ServerState,
    stored: Vec<(u64, u64, String, String)>,
) -> (usize, usize) {
    state.reset_for_replay();

    let mut msg_count = 0;

    for (seq, timestamp, event_type, details) in stored {
        if let Some(event) = ChatEvent::from_event_store(&event_type, &details) {
            if let ChatEvent::MessageBroadcast(ref msg) = event {
                state.add_message(msg.clone());
                msg_count += 1;
            }
            state.record_reconstructed(seq, timestamp, event);
        }
    }

    state.finalize_replay();

    (state.events().len(), msg_count)
}

/// Load events from the database and reconstruct state in one call.
#[allow(dead_code)]
pub fn replay_from_db(
    state: &mut ServerState,
    conn: &Connection,
) -> Result<(usize, usize), rusqlite::Error> {
    let stored = storage::load_events(conn)?;
    Ok(reconstruct_from_events(state, stored))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> ServerState {
        ServerState::new()
    }

    #[test]
    fn test_reconstruct_empty() {
        let mut s = make_state();
        let (ec, mc) = reconstruct_from_events(&mut s, vec![]);
        assert_eq!(ec, 0);
        assert_eq!(mc, 0);
        assert!(s.events().is_empty());
        assert!(s.messages().is_empty());
        // next_seq stays 0
        let (seq, _) = s.record_event(ChatEvent::ClientConnected("a".into()));
        assert_eq!(seq, 0);
    }

    #[test]
    fn test_reconstruct_all_event_types() {
        let mut s = make_state();
        let stored = vec![
            (0, 10, "ClientConnected".into(), "alice".into()),
            (1, 20, "MessageReceived".into(), "alice: hey".into()),
            (2, 30, "MessageBroadcast".into(), "alice: hey".into()),
            (3, 40, "ClientDisconnected".into(), "alice".into()),
        ];

        let (ec, mc) = reconstruct_from_events(&mut s, stored);
        assert_eq!(ec, 4);
        assert_eq!(mc, 1);
        assert_eq!(s.messages(), &["alice: hey"]);
        assert_eq!(s.events().len(), 4);
    }

    #[test]
    fn test_reconstruct_order_preserved() {
        let mut s = make_state();
        let stored = vec![
            (0, 10, "MessageBroadcast".into(), "first".into()),
            (1, 20, "MessageBroadcast".into(), "second".into()),
            (2, 30, "MessageBroadcast".into(), "third".into()),
        ];

        reconstruct_from_events(&mut s, stored);
        assert_eq!(s.messages(), &["first", "second", "third"]);
    }

    #[test]
    fn test_reconstruct_next_seq_restored() {
        let mut s = make_state();

        let stored = (0..5)
            .map(|i| {
                (
                    i,
                    i * 10,
                    "MessageBroadcast".into(),
                    format!("msg_{}", i),
                )
            })
            .collect::<Vec<_>>();

        reconstruct_from_events(&mut s, stored);

        // Next event gets seq = 5
        let (seq, _) = s.record_event(ChatEvent::ClientConnected("new".into()));
        assert_eq!(seq, 5);
    }

    #[test]
    fn test_reconstruct_non_contiguous_sequences() {
        let mut s = make_state();
        // Sequences have gaps — should still work
        let stored = vec![
            (5, 10, "MessageBroadcast".into(), "a".into()),
            (10, 20, "MessageBroadcast".into(), "b".into()),
            (42, 30, "MessageBroadcast".into(), "c".into()),
        ];

        reconstruct_from_events(&mut s, stored);
        assert_eq!(s.messages(), &["a", "b", "c"]);

        // next_seq = max + 1 = 43
        let (seq, _) = s.record_event(ChatEvent::ClientConnected("x".into()));
        assert_eq!(seq, 43);
    }

    #[test]
    fn test_reconnect_scenario() {
        let mut s = make_state();

        // Simulate: connect, message, broadcast, disconnect, reconnect, disconnect
        let events = vec![
            (0, 10, "ClientConnected".into(), "client_A".into()),
            (1, 20, "MessageReceived".into(), "client_A: hello".into()),
            (2, 30, "MessageBroadcast".into(), "client_A: hello".into()),
            (3, 40, "ClientDisconnected".into(), "client_A".into()),
            // Reconnect
            (4, 50, "ClientConnected".into(), "client_A".into()),
            (5, 60, "ClientDisconnected".into(), "client_A".into()),
        ];

        reconstruct_from_events(&mut s, events);

        // Messages should only contain broadcasts
        assert_eq!(s.messages(), &["client_A: hello"]);

        // Events should contain all 6
        assert_eq!(s.events().len(), 6);
    }

    #[test]
    fn test_replay_persistence_roundtrip() {
        use rusqlite::Connection;

        // Use in-memory DB
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE event_store (
                id INTEGER PRIMARY KEY,
                sequence_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                details TEXT NOT NULL
            )",
        )
        .unwrap();

        // Save events manually
        crate::storage::save_event(&conn, 0, 100, "ClientConnected", "alice").unwrap();
        crate::storage::save_event(&conn, 1, 200, "MessageReceived", "alice: hi").unwrap();
        crate::storage::save_event(&conn, 2, 300, "MessageBroadcast", "alice: hi").unwrap();

        // Replay from DB
        let mut state = ServerState::new();
        let (ec, mc) = replay_from_db(&mut state, &conn).unwrap();

        assert_eq!(ec, 3);
        assert_eq!(mc, 1);
        assert_eq!(state.messages(), &["alice: hi"]);
        assert_eq!(state.events().len(), 3);

        // next_seq continues
        let (seq, _) = state.record_event(ChatEvent::ClientConnected("bob".into()));
        assert_eq!(seq, 3);
    }

    #[test]
    fn test_replay_with_invalid_events_skips_them() {
        let mut s = make_state();
        let stored = vec![
            (0, 10, "ClientConnected".into(), "a".into()),
            (1, 20, "INVALID_TYPE".into(), "garbage".into()),          // skipped
            (2, 30, "MessageBroadcast".into(), "valid".into()),
            (3, 40, "MessageReceived".into(), "bad-format".into()),    // skipped (no ": ")
            (4, 50, "ClientDisconnected".into(), "a".into()),
        ];

        let (ec, mc) = reconstruct_from_events(&mut s, stored);
        assert_eq!(ec, 3);  // 5 stored, 2 invalid = 3 parsed
        assert_eq!(mc, 1);  // only "valid" broadcast
        assert_eq!(s.messages(), &["valid"]);
    }
}
