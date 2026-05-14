mod http;
mod routes;
mod websocket;

use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::broadcast;

use http::handle_connection;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let (tx, _rx) = broadcast::channel::<String>(16);

    let tx = Arc::new(tx);

    println!("Server listening on http://127.0.0.1:8080");

    loop {
        let (stream, _) = listener.accept().await?;

        let tx = Arc::clone(&tx);

        tokio::spawn(async move {
            handle_connection(stream, tx).await;
        });
    }
}
