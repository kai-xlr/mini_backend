use rusqlite::{Connection, Result};

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("chat.db")?;

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            body TEXT NOT NULL
        )
        ",
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

    let rows = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;

    let mut messages = Vec::new();

    for msg in rows {
        messages.push(msg?);
    }

    Ok(messages)
}
