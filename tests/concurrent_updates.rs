use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, broadcast};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};

use rusqlite::Connection;

async fn start_test_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY, body TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS event_store (
             id INTEGER PRIMARY KEY,
             sequence_id INTEGER NOT NULL,
             timestamp INTEGER NOT NULL,
             event_type TEXT NOT NULL,
             details TEXT NOT NULL
         )",
    )
    .unwrap();
    let conn = Arc::new(Mutex::new(conn));

    let state = Arc::new(Mutex::new(mini_backend::state::ServerState::new()));
    let (tx, _) = broadcast::channel::<String>(16);
    let tx = Arc::new(tx);

    tokio::spawn(async move {
        mini_backend::serve(listener, conn, state, tx).await;
    });

    addr
}

async fn http_get_body(addr: SocketAddr, path: &str) -> String {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        path
    );
    stream.write_all(request.as_bytes()).await.unwrap();

    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    let response = String::from_utf8_lossy(&response);

    if let Some(body) = response.split("\r\n\r\n").nth(1) {
        body.to_string()
    } else {
        String::new()
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_updates() {
    let addr = start_test_server().await;
    let ws_url = format!("ws://{}/ws", addr);

    let num_clients = 5;
    let messages_per_client = 3;
    let total_expected = num_clients * messages_per_client;
    let mut handles = vec![];

    for client_id in 0..num_clients {
        let url = ws_url.clone();
        let handle = tokio::spawn(async move {
            let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
            let (mut write, read) = ws_stream.split();

            tokio::spawn(async move {
                let mut read = read;
                while let Some(msg) = read.next().await {
                    if let Err(e) = msg {
                        eprintln!("[Test] Reader error for client {}: {}", client_id, e);
                    }
                }
            });

            for msg_id in 0..messages_per_client {
                let payload = format!("Client {} - Msg {}", client_id, msg_id);
                println!("[Sending] {}", payload);
                write.send(Message::Text(payload)).await.expect("Failed to send");
                tokio::task::yield_now().await;
            }

            if let Err(e) = write.close().await {
                eprintln!("[Test] Failed to close WS for client {}: {}", client_id, e);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Client task panicked");
    }

    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let events_body = loop {
        let body = http_get_body(addr, "/events").await;
        let broadcast_count = body.lines().filter(|l| l.contains("BROADCAST:")).count();
        if broadcast_count >= total_expected {
            break body;
        }
        if tokio::time::Instant::now() > deadline {
            panic!(
                "Timed out waiting for broadcasts: got {}, expected {}",
                broadcast_count, total_expected
            );
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    };

    println!("=== Server Events ===\n{}", events_body);

    let lines: Vec<&str> = events_body.lines().filter(|l| !l.is_empty()).collect();

    let broadcast_lines: Vec<&&str> = lines.iter().filter(|l| l.contains("BROADCAST:")).collect();
    println!(
        "Broadcast count: {} (expected: {})",
        broadcast_lines.len(),
        total_expected
    );
    assert_eq!(
        broadcast_lines.len(),
        total_expected,
        "Message loss or duplication detected"
    );

    for client_id in 0..num_clients {
        for msg_id in 0..messages_per_client {
            let expected = format!("Client {} - Msg {}", client_id, msg_id);
            let found = broadcast_lines.iter().any(|l| l.contains(&expected));
            assert!(found, "Missing message: {}", expected);
        }
    }
    println!("All {} individual messages verified present", total_expected);

    // Sequence IDs are parsed from the format "[seq_id] [timestamp] EVENT: ..."
    // (see format_events_body in src/http.rs). Any format change there must update
    // this parsing.
    let mut seq_ids: Vec<u64> = lines
        .iter()
        .filter_map(|l| {
            let s = l.trim_start_matches('[');
            let end = s.find(']')?;
            s[..end].parse().ok()
        })
        .collect();
    seq_ids.sort();
    for (i, &seq) in seq_ids.iter().enumerate() {
        assert_eq!(
            seq, i as u64,
            "Gap or non-contiguous sequence at index {}",
            i
        );
    }
    println!(
        "Sequence IDs verified: {} total, contiguous 0..{}",
        seq_ids.len(),
        seq_ids.len().saturating_sub(1)
    );

    let connect_lines: Vec<&&str> = lines.iter().filter(|l| l.contains("] CONNECTED:")).collect();
    let disconnect_lines: Vec<&&str> = lines.iter().filter(|l| l.contains("] DISCONNECTED:")).collect();
    println!(
        "Connections: {} (expected {}), Disconnections: {} (expected {})",
        connect_lines.len(),
        num_clients,
        disconnect_lines.len(),
        num_clients
    );
    assert_eq!(connect_lines.len(), num_clients);
    assert_eq!(disconnect_lines.len(), num_clients);

    let audit = http_get_body(addr, "/events/audit").await;
    println!("=== Audit ===\n{}", audit);
    assert!(audit.contains("INTEGRITY OK"), "Integrity audit failed");

    println!(
        "=== TEST PASSED: {} clients × {} messages = {} messages without corruption ===",
        num_clients, messages_per_client, total_expected
    );
}
