use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, broadcast};

use tokio_tungstenite::accept_async;

use crate::routes::route_request;
use crate::state::ServerState;
use crate::websocket::handle_websocket;

pub async fn handle_connection(
    mut stream: TcpStream,
    tx: Arc<broadcast::Sender<String>>,
    state: Arc<Mutex<ServerState>>,
) {
    let addr = stream.peer_addr().ok();

    if let Some(addr) = addr {
        println!("[CONN] {} Connected", addr);
    }

    let mut buffer = [0; 1024];

    match stream.read(&mut buffer).await {
        Ok(size) if size > 0 => {
            let request_str = String::from_utf8_lossy(&buffer[..size]);

            let (status, response) = route_request(&request_str);

            // Upgrade to websocket
            if status == "101 SWITCHING PROTOCOLS" {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        if let Some(addr) = addr {
                            handle_websocket(ws_stream, tx, state, addr).await;
                        }
                    }

                    Err(e) => {
                        eprintln!("[ERR] WebSocket handshake failed: {}", e);
                    }
                }

                return;
            }

            // Standard HTTP response
            if let Err(e) = stream.write_all(response.as_bytes()).await {
                eprintln!("[ERR] Failed to write response: {}", e);

                return;
            }

            if let Err(e) = stream.flush().await {
                eprintln!("[ERR] Failed to flush stream: {}", e);
            }
        }

        Ok(_) => {}

        Err(e) => {
            eprintln!("[ERR] Failed to read from stream: {}", e);
        }
    }
}
