use std::process;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::{Mutex, broadcast};

use mini_backend::state::ServerState;
use mini_backend::storage::{init_db, load_events, load_messages};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let conn = match init_db() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("[ERR] DB init failed: {}", e);
            process::exit(1);
        }
    };

    println!("[DB] SQLite initialized");

    let conn = Arc::new(Mutex::new(conn));

    let mut state_inner = ServerState::new();

    {
        let conn_lock = conn.lock().await;

        if let Ok(stored) = load_events(&conn_lock) {
            if !stored.is_empty() {
                let (event_count, msg_count) =
                    mini_backend::replay::reconstruct_from_events(&mut state_inner, stored);
                println!(
                    "[REPLAY] Restored {} events, {} messages from event store",
                    event_count, msg_count
                );
            } else if let Ok(messages) = load_messages(&conn_lock) {
                for msg in messages {
                    state_inner.add_message(msg);
                }
                println!(
                    "[STATE] Restored {} messages from messages table (legacy)",
                    state_inner.messages().len()
                );
            }
        }
    }

    let state = Arc::new(Mutex::new(state_inner));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let (tx, _) = broadcast::channel::<String>(16);
    let tx = Arc::new(tx);

    println!("[SERVER] Listening on http://127.0.0.1:8080");

    {
        let s = state.lock().await;
        println!(
            "[SYSTEM READY] Ready to accept incoming WebSockets. Active Clients: {} | Historically Cached Messages: {}",
            s.client_count(),
            s.messages().len()
        );
    }

    mini_backend::serve(listener, conn, state, tx).await;

    Ok(())
}
