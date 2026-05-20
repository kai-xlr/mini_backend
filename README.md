# mini_backend

An async HTTP + WebSocket backend written in Rust, built on tokio. Uses shared state via `Arc<Mutex<ServerState>>` to track connected WebSocket clients with SQLite persistence for messages and events.

## Endpoints

| Method | Path | Response |
|--------|------|----------|
| `GET` | `/health` | `200 OK` with body `OK` |
| `GET` | `/echo/<message>` | `200 OK` with body `<message>` |
| `GET` | `/messages` | `200 OK` — all messages broadcast in the chat session (newline-separated) |
| `GET` | `/events` | `200 OK` — chronological event log (connections, messages, disconnections) |
| `GET` | `/events/audit` | `200 OK` — integrity check comparing in-memory event count against SQLite |
| `GET` | `/ws` | WebSocket upgrade (101) — broadcast chat |
| `GET` | any other path | `404 NOT FOUND` |
| any | anything | `400 BAD REQUEST` if method is not GET |

WebSocket clients connected to `/ws` participate in a broadcast chat — every text message sent by any client is forwarded to all connected clients. Messages and lifecycle events are persisted to SQLite (`chat.db`). On restart, persisted messages are loaded back into memory.

## Usage

```bash
cargo run
```

Server listens on `http://127.0.0.1:8080`.

```bash
# HTTP
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/echo/hello
curl http://127.0.0.1:8080/messages
curl http://127.0.0.1:8080/events
curl http://127.0.0.1:8080/events/audit

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
| `rusqlite` | SQLite persistence for messages and events |

## Project Structure

```
src/
  main.rs      — Server setup: DB init, state restore, listener, broadcast channel, spawns connections
  http.rs      — HTTP connection lifecycle: read, route, upgrade or respond (including /messages, /events, /events/audit)
  routes.rs    — HTTP request parsing, routing logic, and response helpers
  state.rs     — Shared ServerState tracking clients, messages, and recorded events (ChatEvent enum)
  websocket.rs — WebSocket handler with broadcast fan-out, event recording, and SQLite persistence
  db.rs        — SQLite schema (messages, event_store) and CRUD helpers
```
