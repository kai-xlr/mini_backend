use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, broadcast};

use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Role;

use rusqlite::Connection;

use crate::models::ChatEvent;
use crate::routes::{ok, route_request};
use crate::state::ServerState;
use crate::websocket::handle_websocket;

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

// Assignment 5: Extract formatting logic out of the main handler handler block
fn format_events_body(state: &ServerState) -> String {
    let mut lines = Vec::new();

    for recorded in state.events() {
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

    lines.join("\n")
}

// Assignment 5: Extract verification auditing calculations out of the main handler
fn perform_integrity_audit(memory_count: usize, db_count: usize) -> String {
    if memory_count == db_count {
        format!(
            "INTEGRITY OK: [{}] events verified chronologically.",
            memory_count
        )
    } else {
        format!(
            "INTEGRITY ERROR: Memory has [{}] entries but DB has [{}] entries.",
            memory_count, db_count
        )
    }
}

// -------------------------
// WebSocket handshake helpers
// -------------------------

fn base64_encode_sha1(input: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();

    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }

        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }

    out
}

fn websocket_accept_key(key: &str) -> String {
    use sha1::{Digest, Sha1};

    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let digest = hasher.finalize();

    base64_encode_sha1(&digest)
}

fn extract_ws_key(request: &str) -> Option<String> {
    for line in request.lines() {
        if line.to_lowercase().starts_with("sec-websocket-key:") {
            return line.split(':').nth(1).map(|k| k.trim().to_string());
        }
    }
    None
}

async fn handle_websocket_upgrade(
    mut stream: TcpStream,
    request: &str,
    tx: Arc<broadcast::Sender<String>>,
    state: Arc<Mutex<ServerState>>,
    db: Arc<Mutex<Connection>>,
    addr: std::net::SocketAddr,
) {
    let key = match extract_ws_key(request) {
        Some(k) => k,
        None => return,
    };

    let accept = websocket_accept_key(&key);

    let upgrade_response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\
         \r\n",
        accept
    );

    if stream.write_all(upgrade_response.as_bytes()).await.is_err() {
        return;
    }

    if stream.flush().await.is_err() {
        return;
    }

    let ws_stream = WebSocketStream::from_raw_socket(stream, Role::Server, None).await;

    handle_websocket(ws_stream, tx, state, db, addr).await;
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
    let addr = match stream.peer_addr().ok() {
        Some(a) => a,
        None => return,
    };

    let mut buffer = [0; 1024];

    match stream.read(&mut buffer).await {
        Ok(size) if size > 0 => {
            let request = String::from_utf8_lossy(&buffer[..size]);
            let (method, path) = parse_request_line(&request);

            if method == "GET" && path == "/messages" {
                let s = state.lock().await;
                let body = s.messages().join("\n");
                let response = ok(&body);
                write_response(&mut stream, response).await;
                return;
            }

            if method == "GET" && path == "/events" {
                let s = state.lock().await;
                // Assignment 5: Cleaned up call signature
                let body = format_events_body(&s);
                let response = ok(&body);
                write_response(&mut stream, response).await;
                return;
            }

            if method == "GET" && path == "/events/audit" {
                let memory = state.lock().await;
                let memory_count = memory.events().len();

                let db_conn = db.lock().await;
                let db_count = match crate::storage::get_event_count(&db_conn) {
                    Ok(c) => c,
                    Err(_) => {
                        let response = ok("INTEGRITY ERROR: Unable to read database");
                        write_response(&mut stream, response).await;
                        return;
                    }
                };

                // Assignment 5: Cleaned up audit call logic
                let body = perform_integrity_audit(memory_count, db_count);
                let response = ok(&body);
                write_response(&mut stream, response).await;
                return;
            }

            if method == "GET" && path == "/ws" {
                return handle_websocket_upgrade(stream, &request, tx, state, db, addr).await;
            }

            let (_status, response) = route_request(&request);
            write_response(&mut stream, response).await;
        }
        _ => {}
    }
}
