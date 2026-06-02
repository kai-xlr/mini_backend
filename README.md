# mini_backend

An async HTTP + WebSocket backend in Rust built on tokio with event-sourced state management. Tracks connected WebSocket clients and persists messages + lifecycle events to SQLite. Uses event sourcing for state reconstruction on startup.

## Endpoints

| Method | Path | Response |
|--------|------|----------|
| `GET` | `/health` | `200 OK` with body `OK` |
| `GET` | `/echo/<message>` | `200 OK` with body `<message>` |
| `GET` | `/messages` | `200 OK` — all messages broadcast in the chat session (newline-separated) |
| `GET` | `/events` | `200 OK` — chronological event log with sequence IDs (connections, messages, disconnections) |
| `GET` | `/events/audit` | `200 OK` — integrity check comparing in-memory event count against SQLite |
| `GET` | `/replay` | `200 OK` — replay status (event count, message count, client count) |
| `POST` | `/replay` | `200 OK` — triggers full event-sourced state reconstruction from DB |
| `GET` | `/ws` | WebSocket upgrade (101) — broadcast chat |
| `GET` | any other path | `404 NOT FOUND` |
| any | anything | `400 BAD REQUEST` if method is not GET |

WebSocket clients connected to `/ws` participate in a broadcast chat — every text message is forwarded to all connected clients. Messages and lifecycle events are persisted to SQLite (`chat.db`). On restart, the event log is replayed to reconstruct in-memory state.

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

Database path can be configured via the `CHAT_DB_PATH` environment variable (defaults to `chat.db` in the working directory).

## Tests

```bash
# Run all tests
cargo test

# Run the concurrent integration test with output
cargo test --test concurrent_updates -- --nocapture
```

The concurrent updates test (`tests/concurrent_updates.rs`) spawns 5 WebSocket clients that each send 3 messages simultaneously, then verifies:
- Total messages broadcast matches expected count
- Every client's messages are present in the event log
- No message corruption (partial, overlapping, or malformed)
- Event sequence IDs are contiguous with no gaps
- In-memory and database event counts match

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
  lib.rs         — Library root: public module declarations and serve() entry point
  main.rs        — Binary entry: DB init, state restore via replay, listener, broadcast channel
  http.rs        — HTTP connection lifecycle: read, route, upgrade or respond
  events/        — ChatEvent enum, RecordedEvent struct, parse/serialize helpers
  state/         — ServerState tracking clients, messages, events, and sequence IDs
  storage/       — SQLite schema (messages, event_store) and CRUD helpers
  websocket/     — WebSocket handler with broadcast fan-out, event recording, persistence
  routes/        — HTTP request parsing, routing, and response helpers
  replay/        — Event-sourced state reconstruction from stored events
tests/
  concurrent_updates.rs — Concurrent integration test (5 clients × 3 messages)
```
