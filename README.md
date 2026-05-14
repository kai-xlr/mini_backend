# mini_backend

An async HTTP + WebSocket backend written in Rust, built on tokio.

## Endpoints

| Method | Path | Response |
|--------|------|----------|
| `GET` | `/health` | `200 OK` with body `OK` |
| `GET` | `/echo/<message>` | `200 OK` with body `<message>` |
| `GET` | `/ws` | WebSocket upgrade (101) — broadcast chat |
| `GET` | any other path | `404 NOT FOUND` |
| any | anything | `400 BAD REQUEST` if method is not GET |

WebSocket clients connected to `/ws` participate in a broadcast chat — every text message sent by any client is forwarded to all connected clients.

## Usage

```bash
cargo run
```

Server listens on `http://127.0.0.1:8080`.

```bash
# HTTP
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/echo/hello

# WebSocket chat — open two terminals
websocat ws://127.0.0.1:8080/ws
# Messages typed in one terminal appear in all others
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime (TCP listener, I/O, broadcast channels) |
| `tokio-tungstenite` | WebSocket upgrade and frame handling |
| `futures-util` | Async stream/sink combinators |

## Project Structure

```
src/
  main.rs      — Server setup: listener, broadcast channel, spawns connections
  http.rs      — HTTP connection lifecycle: read, route, upgrade or respond
  routes.rs    — HTTP request parsing, routing logic, and response helpers
  websocket.rs — WebSocket handler with broadcast fan-out via tokio::sync::broadcast
```
