mod routes;
mod utils;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) {
    println!("[CONNECTION] accepted");

    if let Err(e) = read_request(&mut stream) {
        eprintln!("[ERROR] Failed to handle request: {}", e);
    }
}

fn read_request(stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let mut buffer = [0; 1024];

    let size = stream.read(&mut buffer)?;

    if size == 0 {
        return Ok(());
    }

    let request_str = String::from_utf8_lossy(&buffer[..size]);

    if let Some(line) = request_str.lines().next() {
        println!("[REQUEST] {}", line);
    }

    let (status, response) = routes::route_request(&request_str);

    println!("[RESPONSE] {}", status);

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}
fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on http://127.0.0.1:8080");

    for stream in listener.incoming().flatten() {
        handle_connection(stream);
    }

    Ok(())
}
