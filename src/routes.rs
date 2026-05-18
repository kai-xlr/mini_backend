pub fn ok(body: &str) -> String {
    format!("HTTP/1.1 200 OK\r\n\r\n{}", body)
}

pub fn not_found() -> String {
    "HTTP/1.1 404 NOT FOUND\r\n\r\nPage not found".to_string()
}

pub fn bad_request() -> String {
    "HTTP/1.1 400 BAD REQUEST\r\n\r\nBad Request".to_string()
}

pub fn route_request(request: &str) -> (&'static str, String) {
    let request_line = match request.lines().next() {
        Some(line) => line,
        None => return ("400 BAD REQUEST", bad_request()),
    };

    let mut parts = request_line.split_whitespace();

    let method = match parts.next() {
        Some(m) => m,
        None => return ("400 BAD REQUEST", bad_request()),
    };

    let path = match parts.next() {
        Some(p) => p,
        None => return ("400 BAD REQUEST", bad_request()),
    };

    if method != "GET" {
        return ("400 BAD REQUEST", bad_request());
    }

    if path == "/health" {
        ("200 OK", ok("OK"))
    } else if path == "/ws" {
        ("101 SWITCHING PROTOCOLS", String::new())
    } else if path == "/messages" {
        ("200 OK", ok(""))
    } else if let Some(message) = path.strip_prefix("/echo/") {
        ("200 OK", ok(message))
    } else {
        ("404 NOT FOUND", not_found())
    }
}
