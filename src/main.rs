mod routes;
mod utils;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) {
    println!("[CONNECTION] accepted");
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            let request_str = String::from_utf8_lossy(&buffer[..size]);

            // Log the incoming request line
            if let Some(line) = request_str.lines().next() {
                println!("[REQUEST] {}", line);
            }

            // Route and handle validation results
            let (status, response) = routes::route_request(&request_str);
            println!("[RESPONSE] {}", status);

            if let Err(e) = stream.write_all(response.as_bytes()) {
                eprintln!("[ERROR] Failed to write response: {}", e);
                return;
            }

            if let Err(e) = stream.flush() {
                eprintln!("[ERROR] Failed to flush stream: {}", e);
            }
        }
        Ok(_) => {}
        Err(e) => eprintln!("[ERROR] Failed to read from stream: {}", e),
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on http://127.0.0.1:8080");

    for stream in listener.incoming().flatten() {
        handle_connection(stream);
    }

    Ok(())
}
