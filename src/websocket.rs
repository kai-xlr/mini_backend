use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};

use tokio::sync::broadcast;

use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use tokio::net::TcpStream;

pub async fn handle_websocket(
    mut ws_stream: WebSocketStream<TcpStream>,
    tx: Arc<broadcast::Sender<String>>,
) {
    println!("[WS] Handshake successful");

    let mut rx = tx.subscribe();

    loop {
        tokio::select! {

            // Incoming websocket message
            result = ws_stream.next() => {
                match result {
                    Some(Ok(message)) => {

                        if message.is_text() {
                            if let Ok(text) = message.to_text() {
                                println!("[WS MESSAGE] {}", text);

                                if tx.send(text.to_string()).is_err() {
                                    break;
                                }
                            }
                        }

                        if message.is_close() {
                            println!("[WS] Client disconnected");
                            break;
                        }
                    }

                    _ => {
                        break;
                    }
                }
            }

            // Incoming broadcast message
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if ws_stream
                            .send(Message::Text(msg.into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }

                    Err(_) => {
                        break;
                    }
                }
            }
        }
    }
}
