# mini_backend

A minimal HTTP backend written in Rust.

## Features

- Health check endpoint: `GET /health` returns 200 OK
- Echo endpoint: `GET /echo/<message>` returns the message
- Otherwise returns 404 NOT FOUND

## Running

```bash
cargo run
```

Server listens on `127.0.0.1:8080`.
