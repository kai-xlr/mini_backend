# mini_backend

An async HTTP + WebSocket backend written in Rust, built on tokio.

## Endpoints

| Method | Path | Response |
|--------|------|----------|
| `GET` | `/health` | `200 OK` with body `OK` |
| `GET` | `/echo/<message>` | `200 OK` with body `<message>` |
| `GET` | `/ws` | WebSocket upgrade (101) — echoes text frames back |
| `GET` | any other path | `404 NOT FOUND` |
| any | anything | `400 BAD REQUEST` if method is not GET |

WebSocket clients connected to `/ws` receive an echo of every text message sent.

## Usage

```bash
cargo run
```

Server listens on `http://127.0.0.1:8080`.

```bash
# HTTP
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/echo/hello

# WebSocket (using websocat or similar)
websocat ws://127.0.0.1:8080/ws
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime (TCP listener, I/O) |
| `tokio-tungstenite` | WebSocket upgrade and frame handling |
| `futures-util` | Async stream/sink combinators |

## Project Structure

```
src/
  main.rs   — TCP listener, connection handling, WebSocket echo
  routes.rs — HTTP request parsing and routing
  utils.rs  — HTTP response body helpers
```
