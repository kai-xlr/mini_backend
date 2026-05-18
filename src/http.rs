use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, broadcast};

use tokio_tungstenite::accept_async;

use crate::routes::{ok, route_request};
use crate::state::ServerState;
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
