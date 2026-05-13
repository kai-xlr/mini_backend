use crate::utils;

pub fn route_request(request: &str) -> (&'static str, String) {
    let request_line = match request.lines().next() {
        Some(line) => line,
        None => return ("400 BAD REQUEST", utils::bad_request()),
    };

    let mut parts = request_line.split_whitespace();

    let method = match parts.next() {
        Some(m) => m,
        None => return ("400 BAD REQUEST", utils::bad_request()),
    };

    let path = match parts.next() {
        Some(p) => p,
        None => return ("400 BAD REQUEST", utils::bad_request()),
    };

    if method != "GET" {
        return ("400 BAD REQUEST", utils::bad_request());
    }

    if path == "/health" {
        ("200 OK", utils::ok("OK"))
    } else if path == "/ws" {
        ("101 SWITCHING PROTOCOLS", String::new())
    } else if let Some(message) = path.strip_prefix("/echo/") {
        ("200 OK", utils::ok(message))
    } else {
        ("404 NOT FOUND", utils::not_found())
    }
}
