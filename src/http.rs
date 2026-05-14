use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast;

use tokio_tungstenite::accept_async;

use crate::routes::route_request;
use crate::websocket::handle_websocket;

pub async fn handle_connection(mut stream: TcpStream, tx: Arc<broadcast::Sender<String>>) {
    println!("[CONNECTION] accepted");

    let mut buffer = [0; 1024];

    match stream.read(&mut buffer).await {
        Ok(size) if size > 0 => {
            let request_str = String::from_utf8_lossy(&buffer[..size]);

            if let Some(line) = request_str.lines().next() {
                println!("[REQUEST] {}", line);
            }

            let (status, response) = route_request(&request_str);

            println!("[RESPONSE] {}", status);

            // Upgrade to websocket
            if status == "101 SWITCHING PROTOCOLS" {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        handle_websocket(ws_stream, tx).await;
                    }

                    Err(e) => {
                        eprintln!("[WS ERROR] Handshake failed: {}", e);
                    }
                }

                return;
            }

            // Standard HTTP response
            if let Err(e) = stream.write_all(response.as_bytes()).await {
                eprintln!("[ERROR] Failed to write response: {}", e);
                return;
            }

            if let Err(e) = stream.flush().await {
                eprintln!("[ERROR] Failed to flush stream: {}", e);
            }
        }

        Ok(_) => {}

        Err(e) => {
            eprintln!("[ERROR] Failed to read from stream: {}", e);
        }
    }
}
