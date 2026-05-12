mod routes;
mod utils;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

async fn handle_connection(mut stream: TcpStream) {
    println!("[CONNECTION] accepted");

    let mut buffer = [0; 1024];

    // Read initial HTTP request
    let size = match stream.read(&mut buffer).await {
        Ok(size) if size > 0 => size,
        Ok(_) => return,
        Err(e) => {
            eprintln!("[ERROR] Failed to read stream: {}", e);
            return;
        }
    };

    let request = String::from_utf8_lossy(&buffer[..size]);

    if let Some(line) = request.lines().next() {
        println!("[REQUEST] {}", line);
    }

    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    // =========================
    // WebSocket Route
    // =========================
    if path == "/ws" {
        match accept_async(stream).await {
            Ok(mut ws_stream) => {
                println!("[WS] Handshake successful");

                // 🔢 Per-client message counter
                let mut msg_count = 0;

                while let Some(msg_result) = ws_stream.next().await {
                    match msg_result {
                        Ok(msg) => match msg {
                            Message::Text(text) => {
                                msg_count += 1;

                                println!("[WS] Received: {}", text);

                                let response = if text.to_uppercase() == "PING" {
                                    "PONG".to_string()
                                } else {
                                    format!("Message #{}: {}", msg_count, text)
                                };

                                if let Err(e) = ws_stream.send(Message::Text(response)).await {
                                    eprintln!("[WS ERROR] Send failed: {}", e);
                                    break;
                                }
                            }

                            Message::Close(_) => {
                                println!("[WS] Client disconnected (close frame received)");
                                break;
                            }

                            _ => {
                                // Ignore Ping/Pong/Binary frames
                            }
                        },

                        Err(e) => {
                            eprintln!("[WS ERROR] Message error: {}", e);
                            break;
                        }
                    }
                }

                println!("[WS] Connection task terminating for this client");
            }

            Err(e) => {
                eprintln!("[WS ERROR] Handshake failed: {}", e);
            }
        }

        return;
    }

    // =========================
    // HTTP Route Handling
    // =========================
    let (status, response) = routes::route_request(&request);

    println!("[RESPONSE] {}", status);

    if let Err(e) = stream.write_all(response.as_bytes()).await {
        eprintln!("[ERROR] Failed to write response: {}", e);
        return;
    }

    if let Err(e) = stream.flush().await {
        eprintln!("[ERROR] Failed to flush stream: {}", e);
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("Server running at http://127.0.0.1:8080");

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            handle_connection(stream).await;
        });
    }
}
