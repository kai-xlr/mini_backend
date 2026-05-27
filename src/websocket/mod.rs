use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};

use tokio::net::TcpStream;
use tokio::sync::{Mutex, broadcast};

use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use rusqlite::Connection;

use crate::models::ChatEvent;
use crate::state::ServerState;
use crate::storage::{save_event, save_message};

pub async fn handle_websocket(
    mut ws_stream: WebSocketStream<TcpStream>,
    tx: Arc<broadcast::Sender<String>>,
    state: Arc<Mutex<ServerState>>,
    db: Arc<Mutex<Connection>>,
    addr: SocketAddr,
) {
    // -------------------------
    // Client Connection Phase
    // -------------------------
    {
        let mut s = state.lock().await;
        s.add_client(addr.to_string());

        // Assignment 4: Drive clock entirely from ServerState method
        let (seq, timestamp) = s.record_event(ChatEvent::ClientConnected(addr.to_string()));

        let conn = db.lock().await;
        match save_event(&conn, seq, timestamp, "ClientConnected", &addr.to_string()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("[DB ERR] Failed to save ClientConnected event: {}", e);
            }
        }

        println!("[INFO] {} Joined chat (Total: {})", addr, s.client_count());
    }

    let mut rx = tx.subscribe();

    // -------------------------
    // Core Event Loop Phase
    // -------------------------
    loop {
        tokio::select! {
            result = ws_stream.next() => {
                match result {
                    Some(Ok(message)) => {

                        if message.is_text()
                            && let Ok(text) = message.to_text()
                        {
                            if text.trim().is_empty() {
                                eprintln!("[WARN] Empty message received from {}; ignoring.", addr);
                                continue;
                            }

                            // -------------------------
                            // Message Received & Stored
                            // -------------------------
                            let (seq, timestamp) = {
                                let mut s = state.lock().await;
                                // Assignment 4: Capture baseline time token
                                s.record_event(ChatEvent::MessageReceived {
                                    sender: addr.to_string(),
                                    body: text.to_string(),
                                })
                            };

                            {
                                let conn = db.lock().await;
                                match save_event(
                                    &conn,
                                    seq,
                                    timestamp,
                                    "MessageReceived",
                                    &format!("{}: {}", addr, text),
                                ) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        eprintln!("[DB ERR] Failed to save MessageReceived event: {}", e);
                                    }
                                }
                            }

                            let broadcast_msg = format!("{}: {}", addr, text);

                            {
                                let mut s = state.lock().await;
                                s.add_message(broadcast_msg.clone());
                            }

                            {
                                let conn = db.lock().await;
                                match save_message(&conn, &broadcast_msg) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        eprintln!("[DB ERR] Failed to save broadcast message: {}", e);
                                    }
                                }
                            }

                            // -------------------------
                            // Message Broadcast Tracking
                            // -------------------------
                            let (b_seq, b_timestamp) = {
                                let mut s = state.lock().await;
                                s.record_event(ChatEvent::MessageBroadcast(
                                    broadcast_msg.clone(),
                                ))
                            };

                            {
                                let conn = db.lock().await;
                                match save_event(
                                    &conn,
                                    b_seq,
                                    b_timestamp,
                                    "MessageBroadcast",
                                    &broadcast_msg,
                                ) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        eprintln!("[DB ERR] Failed to save MessageBroadcast event: {}", e);
                                    }
                                }
                            }

                            if let Err(e) = tx.send(broadcast_msg) {
                                eprintln!("[ERR] Broadcast channel broken ({}). Dropping client connection: {}", e, addr);
                                break;
                            }
                        }

                        if message.is_close() {
                            println!("[INFO] Client {} sent close frame.", addr);
                            break;
                        }
                    }

                    Some(Err(e)) => {
                        eprintln!("[WARN] WebSocket error on connection {}: {}", addr, e);
                        break;
                    }
                    None => break,
                }
            }

            result = rx.recv() => {
                if let Ok(msg) = result {
                    if let Err(e) = ws_stream.send(Message::Text(msg)).await {
                        eprintln!("[WARN] Failed to send broadcast message to client {}: {}", addr, e);
                        break;
                    }
                } else {
                    eprintln!("[ERR] Internal broadcast sync drop detected for client: {}", addr);
                    break;
                }
            }
        }
    }

    // -------------------------
    // Client Disconnection Phase
    // -------------------------
    {
        let mut s = state.lock().await;
        s.remove_client(&addr.to_string());

        // Assignment 4: Unified termination tracking
        let (seq, timestamp) = s.record_event(ChatEvent::ClientDisconnected(addr.to_string()));

        let conn = db.lock().await;
        match save_event(&conn, seq, timestamp, "ClientDisconnected", &addr.to_string()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("[DB ERR] Failed to save ClientDisconnected event: {}", e);
            }
        }

        println!("[INFO] {} Left chat (Total: {})", addr, s.client_count());
    }
}
