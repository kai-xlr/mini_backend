use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt};

use tokio::net::TcpStream;
use tokio::sync::{Mutex, broadcast};

use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use rusqlite::Connection;

use crate::db::{save_event, save_message};
use crate::state::{ChatEvent, ServerState};

pub async fn handle_websocket(
    mut ws_stream: WebSocketStream<TcpStream>,
    tx: Arc<broadcast::Sender<String>>,
    state: Arc<Mutex<ServerState>>,
    db: Arc<Mutex<Connection>>,
    addr: SocketAddr,
) {
    {
        let mut s = state.lock().await;

        s.add_client(addr.to_string());

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        s.record_event(ChatEvent::ClientConnected(addr.to_string()));

        let conn = db.lock().await;
        let _ = save_event(&conn, timestamp, "ClientConnected", &addr.to_string());

        println!("[INFO] {} Joined chat (Total: {})", addr, s.client_count());
    }

    let mut rx = tx.subscribe();

    loop {
        tokio::select! {

            result = ws_stream.next() => {
                match result {

                    Some(Ok(message)) => {

                        if message.is_text()
                            && let Ok(text) = message.to_text()
                        {
                            if text.trim().is_empty() {
                                continue;
                            }

                            // -------------------------
                            // Message Received
                            // -------------------------
                            {
                                let mut s = state.lock().await;

                                let timestamp = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();

                                s.record_event(ChatEvent::MessageReceived {
                                    sender: addr.to_string(),
                                    body: text.to_string(),
                                });

                                let conn = db.lock().await;
                                let _ = save_event(
                                    &conn,
                                    timestamp,
                                    "MessageReceived",
                                    &format!("{}: {}", addr, text),
                                );
                            }

                            let broadcast_msg =
                                format!("{}: {}", addr, text);

                            {
                                let mut s = state.lock().await;
                                s.add_message(broadcast_msg.clone());
                            }

                            {
                                let conn = db.lock().await;
                                let _ = save_message(&conn, &broadcast_msg);
                            }

                            // -------------------------
                            // Message Broadcast
                            // -------------------------
                            {
                                let mut s = state.lock().await;

                                let timestamp = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();

                                s.record_event(ChatEvent::MessageBroadcast(
                                    broadcast_msg.clone(),
                                ));

                                let conn = db.lock().await;
                                let _ = save_event(
                                    &conn,
                                    timestamp,
                                    "MessageBroadcast",
                                    &broadcast_msg,
                                );
                            }

                            if tx.send(broadcast_msg).is_err() {
                                break;
                            }
                        }

                        if message.is_close() {
                            break;
                        }
                    }

                    Some(Err(_)) => break,
                    None => break,
                }
            }

            result = rx.recv() => {
                if let Ok(msg) = result {
                    let _ = ws_stream
                        .send(Message::Text(msg))
                        .await;
                } else {
                    break;
                }
            }
        }
    }

    // -------------------------
    // Client Disconnected
    // -------------------------
    {
        let mut s = state.lock().await;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        s.remove_client(&addr.to_string());

        s.record_event(ChatEvent::ClientDisconnected(addr.to_string()));

        let conn = db.lock().await;
        let _ = save_event(&conn, timestamp, "ClientDisconnected", &addr.to_string());
    }
}
