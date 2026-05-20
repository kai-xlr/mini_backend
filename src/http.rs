use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, broadcast};

use tokio_tungstenite::accept_async;

use crate::routes::{ok, route_request};
use crate::state::{ChatEvent, ServerState};
use crate::websocket::handle_websocket;
use rusqlite::Connection;

// -------------------------
// Helpers
// -------------------------

fn parse_request_line(request: &str) -> (&str, &str) {
    let line = request.lines().next().unwrap_or("");
    let mut parts = line.split_whitespace();

    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");

    (method, path)
}

async fn write_response(stream: &mut TcpStream, response: String) {
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;
}

// -------------------------
// Connection Handler
// -------------------------

pub async fn handle_connection(
    mut stream: TcpStream,
    tx: Arc<broadcast::Sender<String>>,
    state: Arc<Mutex<ServerState>>,
    db: Arc<Mutex<Connection>>,
) {
    let addr = stream.peer_addr().ok();

    let mut buffer = [0; 1024];

    match stream.read(&mut buffer).await {
        Ok(size) if size > 0 => {
            let request = String::from_utf8_lossy(&buffer[..size]);

            let (method, path) = parse_request_line(&request);

            // -------------------------
            // /messages endpoint
            // -------------------------
            if method == "GET" && path == "/messages" {
                let s = state.lock().await;

                let body = s.messages().join("\n");

                let response = ok(&body);

                write_response(&mut stream, response).await;
                return;
            }

            // -------------------------
            // /events endpoint
            // -------------------------
            if method == "GET" && path == "/events" {
                let s = state.lock().await;

                let mut lines = Vec::new();

                for recorded in s.events() {
                    let line = match &recorded.event {
                        ChatEvent::ClientConnected(addr) => {
                            format!("[{}] CONNECTED: {}", recorded.timestamp, addr)
                        }

                        ChatEvent::MessageReceived { sender, body } => {
                            format!(
                                "[{}] RECEIVED from {}: {}",
                                recorded.timestamp, sender, body
                            )
                        }

                        ChatEvent::MessageBroadcast(msg) => {
                            format!("[{}] BROADCAST: {}", recorded.timestamp, msg)
                        }

                        ChatEvent::ClientDisconnected(addr) => {
                            format!("[{}] DISCONNECTED: {}", recorded.timestamp, addr)
                        }
                    };

                    lines.push(line);
                }

                let body = lines.join("\n");

                let response = ok(&body);
                write_response(&mut stream, response).await;
                return;
            }

            if method == "GET" && path == "/events/audit" {
                let memory = state.lock().await;

                let memory_count = memory.events().len();

                let db = db.lock().await;

                let db_count = match crate::db::get_event_count(&db) {
                    Ok(c) => c,
                    Err(_) => {
                        let response = ok("INTEGRITY ERROR: Unable to read database");
                        write_response(&mut stream, response).await;
                        return;
                    }
                };

                let body = if memory_count == db_count {
                    format!(
                        "INTEGRITY OK: [{}] events verified chronologically.",
                        memory_count
                    )
                } else {
                    format!(
                        "INTEGRITY ERROR: Memory has [{}] entries but DB has [{}] entries.",
                        memory_count, db_count
                    )
                };

                let response = ok(&body);
                write_response(&mut stream, response).await;
                return;
            }

            // -------------------------
            // normal routing
            // -------------------------
            let (status, response) = route_request(&request);

            if status == "101 SWITCHING PROTOCOLS" {
                if let Ok(ws_stream) = accept_async(stream).await {
                    if let Some(addr) = addr {
                        handle_websocket(ws_stream, tx, state, db, addr).await;
                    }
                }

                return;
            }

            write_response(&mut stream, response).await;
        }

        _ => {}
    }
}
