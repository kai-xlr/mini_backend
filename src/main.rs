mod routes;
mod utils;

use futures_util::{SinkExt, StreamExt};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use tokio_tungstenite::accept_async;

async fn handle_connection(mut stream: TcpStream) {
    println!("[CONNECTION] accepted");

    let mut buffer = [0; 1024];

    match stream.read(&mut buffer).await {
        Ok(size) if size > 0 => {
            let request_str = String::from_utf8_lossy(&buffer[..size]);

            if let Some(line) = request_str.lines().next() {
                println!("[REQUEST] {}", line);
            }

            let (status, response) = routes::route_request(&request_str);

            println!("[RESPONSE] {}", status);

            // WebSocket route
            if status == "101 SWITCHING PROTOCOLS" {
                match accept_async(stream).await {
                    Ok(mut ws_stream) => {
                        println!("[WS] Handshake successful");

                        while let Some(msg) = ws_stream.next().await {
                            match msg {
                                Ok(message) => {
                                    if message.is_close() {
                                        println!("[WS] Client disconnected");
                                        break;
                                    }

                                    if message.is_text() {
                                        if let Ok(text) = message.to_text() {
                                            println!("[WS MESSAGE] {}", text);
                                        }

                                        if let Err(e) = ws_stream.send(message).await {
                                            eprintln!("[WS ERROR] Failed to send: {}", e);
                                            break;
                                        }
                                    }
                                }

                                Err(e) => {
                                    eprintln!("[WS ERROR] {}", e);
                                    break;
                                }
                            }
                        }
                    }

                    Err(e) => {
                        eprintln!("[WS ERROR] Handshake failed: {}", e);
                    }
                }

                return;
            }

            // Normal HTTP response
            if let Err(e) = stream.write_all(response.as_bytes()).await {
                eprintln!("[ERROR] Failed to write response: {}", e);
                return;
            }

            if let Err(e) = stream.flush().await {
                eprintln!("[ERROR] Failed to flush stream: {}", e);
            }
        }

        Ok(_) => {}

        Err(e) => eprintln!("[ERROR] Failed to read from stream: {}", e),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("Server listening on http://127.0.0.1:8080");

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            handle_connection(stream).await;
        });
    }
}
