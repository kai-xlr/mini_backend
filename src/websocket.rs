use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};

use tokio::net::TcpStream;
use tokio::sync::{Mutex, broadcast};

use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::state::ServerState;

pub async fn handle_websocket(
    mut ws_stream: WebSocketStream<TcpStream>,
    tx: Arc<broadcast::Sender<String>>,
    state: Arc<Mutex<ServerState>>,
    addr: SocketAddr,
) {
    {
        let mut s = state.lock().await;

        s.add_client(addr.to_string());

        println!("[INFO] {} Joined chat (Total: {})", addr, s.client_count());
    }

    let mut rx = tx.subscribe();

    loop {
        tokio::select! {

            // Incoming websocket message
            result = ws_stream.next() => {
                match result {
                    Some(Ok(message)) => {

                        if message.is_text() {
                            if let Ok(text) = message.to_text() {

                                println!(
                                    "[MSG] {}: {}",
                                    addr,
                                    text
                                );

                                let broadcast_msg =
                                    format!("{}: {}", addr, text);

                                if tx.send(broadcast_msg).is_err() {
                                    break;
                                }
                            }
                        }

                        if message.is_close() {
                            break;
                        }
                    }

                    Some(Err(e)) => {
                        eprintln!(
                            "[ERR] WebSocket receive failed: {}",
                            e
                        );

                        break;
                    }

                    None => {
                        break;
                    }
                }
            }

            // Incoming broadcast message
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if let Err(e) = ws_stream
                            .send(Message::Text(msg.into()))
                            .await
                        {
                            eprintln!(
                                "[ERR] WebSocket send failed: {}",
                                e
                            );

                            break;
                        }
                    }

                    Err(e) => {
                        eprintln!(
                            "[ERR] Broadcast receive failed: {}",
                            e
                        );

                        break;
                    }
                }
            }
        }
    }

    // Cleanup disconnected client
    {
        let mut s = state.lock().await;

        s.remove_client(&addr.to_string());

        println!("[INFO] {} Left chat (Total: {})", addr, s.client_count());
    }
}
