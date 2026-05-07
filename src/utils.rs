pub fn ok(body: &str) -> String {
    format!("HTTP/1.1 200 OK\r\n\r\n{}", body)
}

pub fn not_found() -> String {
    "HTTP/1.1 404 NOT FOUND\r\n\r\nPage not found".to_string()
}

pub fn bad_request() -> String {
    "HTTP/1.1 400 BAD REQUEST\r\n\r\nBad Request".to_string()
}
