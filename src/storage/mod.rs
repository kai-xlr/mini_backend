use rusqlite::{Connection, Result};

pub fn init_db() -> Result<Connection> {
    let path = std::env::var("CHAT_DB_PATH").unwrap_or_else(|_| "chat.db".to_string());
    let conn = Connection::open(&path)?;

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            body TEXT NOT NULL
        )
        ",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS event_store (
            id INTEGER PRIMARY KEY,
            timestamp INTEGER NOT NULL,
            event_type TEXT NOT NULL,
            details TEXT NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}

pub fn save_message(conn: &Connection, message: &str) -> Result<()> {
    conn.execute("INSERT INTO messages (body) VALUES (?1)", [message])?;
    Ok(())
}

pub fn load_messages(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT body FROM messages ORDER BY id ASC")?;

    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

    let mut messages = Vec::new();

    for msg in rows {
        messages.push(msg?);
    }

    Ok(messages)
}

pub fn save_event(
    conn: &Connection,
    timestamp: u64,
    event_type: &str,
    details: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO event_store (timestamp, event_type, details)
         VALUES (?1, ?2, ?3)",
        rusqlite::params![timestamp, event_type, details],
    )?;
    Ok(())
}

pub fn load_events(conn: &Connection) -> Result<Vec<(u64, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, event_type, details FROM event_store ORDER BY id ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, u64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut events = Vec::new();
    for event in rows {
        events.push(event?);
    }

    Ok(events)
}

pub fn get_event_count(conn: &Connection) -> Result<usize> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM event_store")?;
    let count: usize = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE messages (id INTEGER PRIMARY KEY, body TEXT NOT NULL);
             CREATE TABLE event_store (id INTEGER PRIMARY KEY, timestamp INTEGER NOT NULL, event_type TEXT NOT NULL, details TEXT NOT NULL);"
        ).unwrap();
        conn
    }

    #[test]
    fn test_save_and_load_messages() {
        let conn = setup();
        save_message(&conn, "hello").unwrap();
        save_message(&conn, "world").unwrap();
        let msgs = load_messages(&conn).unwrap();
        assert_eq!(msgs, vec!["hello", "world"]);
    }

    #[test]
    fn test_load_messages_empty() {
        let conn = setup();
        let msgs = load_messages(&conn).unwrap();
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_save_event_and_count() {
        let conn = setup();
        save_event(&conn, 1000, "ClientConnected", "127.0.0.1:9999").unwrap();
        save_event(&conn, 1001, "MessageReceived", "127.0.0.1:9999: hi").unwrap();
        assert_eq!(get_event_count(&conn).unwrap(), 2);
    }

    #[test]
    fn test_get_event_count_empty() {
        let conn = setup();
        assert_eq!(get_event_count(&conn).unwrap(), 0);
    }

    #[test]
    fn test_load_events_ordered() {
        let conn = setup();
        save_event(&conn, 100, "ClientConnected", "a").unwrap();
        save_event(&conn, 200, "MessageBroadcast", "hello").unwrap();
        save_event(&conn, 300, "ClientDisconnected", "a").unwrap();
        let events = load_events(&conn).unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], (100, "ClientConnected".into(), "a".into()));
        assert_eq!(events[1], (200, "MessageBroadcast".into(), "hello".into()));
        assert_eq!(events[2], (300, "ClientDisconnected".into(), "a".into()));
    }

    #[test]
    fn test_load_events_empty() {
        let conn = setup();
        let events = load_events(&conn).unwrap();
        assert!(events.is_empty());
    }
}
