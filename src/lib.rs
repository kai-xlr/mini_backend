pub mod events;
pub mod http;
pub mod replay;
pub mod routes;
pub mod state;
pub mod storage;
pub mod websocket;

use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::{Mutex, broadcast};

use rusqlite::Connection;

pub async fn serve(
    listener: TcpListener,
    db: Arc<Mutex<Connection>>,
    state: Arc<Mutex<state::ServerState>>,
    tx: Arc<broadcast::Sender<String>>,
) {
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let tx = Arc::clone(&tx);
        let state = Arc::clone(&state);
        let db = Arc::clone(&db);
        tokio::spawn(async move {
            http::handle_connection(stream, tx, state, db).await;
        });
    }
}
