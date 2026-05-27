mod events;
mod http;
mod replay;
mod routes;
mod state;
mod storage;
mod websocket;

use std::process;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::{Mutex, broadcast};

use crate::http::handle_connection;
use crate::state::ServerState;
use crate::storage::{init_db, load_events, load_messages};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // -------------------------
    // DB INIT
    // -------------------------
    let conn = match init_db() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("[ERR] DB init failed: {}", e);
            process::exit(1);
        }
    };

    println!("[DB] SQLite initialized");

    let conn = Arc::new(Mutex::new(conn));

    // -------------------------
    // LOAD STATE VIA REPLAY (EVENT SOURCING)
    // -------------------------
    let mut state_inner = ServerState::new();

    {
        let conn_lock = conn.lock().await;

        if let Ok(stored) = load_events(&conn_lock) {
            if !stored.is_empty() {
                let (event_count, msg_count) = replay::reconstruct_from_events(&mut state_inner, stored);
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

    // -------------------------
    // SERVER START
    // -------------------------
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let (tx, _) = broadcast::channel::<String>(16);
    let tx = Arc::new(tx);

    println!("[SERVER] Listening on http://127.0.0.1:8080");

    // Assignment 6: Safe initialization snapshot visibility log
    {
        let s = state.lock().await;
        println!(
            "[SYSTEM READY] Ready to accept incoming WebSockets. Active Clients: {} | Historically Cached Messages: {}",
            s.client_count(),
            s.messages().len()
        );
        // Lock safely drops here before entry to loop
    }

    loop {
        let (stream, _) = listener.accept().await?;

        let tx = Arc::clone(&tx);
        let state = Arc::clone(&state);
        let db = Arc::clone(&conn);

        tokio::spawn(async move {
            handle_connection(stream, tx, state, db).await;
        });
    }
}
