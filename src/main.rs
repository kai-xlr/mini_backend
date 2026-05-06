use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    if stream.read(&mut buffer).is_err() {
        return;
    }

    let request = String::from_utf8_lossy(&buffer);

    let response = if request.starts_with("GET /health") {
        "HTTP/1.1 200 OK\r\n\r\nOK".to_string()
    } else if request.starts_with("GET /echo/") {
        let path_start = "GET /echo/".len();

        let end = request[path_start..]
            .find(' ')
            .map(|i| path_start + i)
            .unwrap_or(request.len());

        let message = &request[path_start..end];

        format!("HTTP/1.1 200 OK\r\n\r\n{}", message)
    } else {
        "HTTP/1.1 404 NOT FOUND\r\n\r\nPage not found".to_string()
    };

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on port 8080");

    for stream in listener.incoming().flatten() {
        handle_connection(stream);
    }

    Ok(())
}
