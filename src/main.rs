mod http;
mod routes;
mod state;
mod websocket;

use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::{Mutex, broadcast};

use http::handle_connection;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let (tx, _rx) = broadcast::channel::<String>(16);

    let tx = Arc::new(tx);

    let state = Arc::new(Mutex::new(state::ServerState::new()));

    println!("[SERVER] Listening on http://127.0.0.1:8080");

    loop {
        let (stream, _) = listener.accept().await?;

        // We use Arc because tokio::spawn requires captured values
        // to live for 'static. A reference like &tx could become invalid
        // while the spawned task is still running.
        let tx = Arc::clone(&tx);

        let state = Arc::clone(&state);

        tokio::spawn(async move {
            handle_connection(stream, tx, state).await;
        });
    }
}
