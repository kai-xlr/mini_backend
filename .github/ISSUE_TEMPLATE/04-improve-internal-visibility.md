---
name: "Week 5 — Issue 4: Improve Internal Visibility / Logging"
about: Medium Energy (Day 4–5, Task 2)
title: "Event Tracing and State Transition Logging"
labels: ["week-5", "medium-energy", "observability"]
assignees: []
---

## Goal

Add structured event tracing so you can follow every state transition through the system. When something goes wrong under concurrency, you want logs that tell you exactly what happened, in what order, and where.

## Background

Right now logging is inconsistent: some events print with `[INFO]`, errors with `[ERR]` or `[WARN]`, DB issues with `[DB ERR]`. There's no trace tying a single message's lifecycle together. This makes debugging concurrent issues harder than it needs to be.

## Requirements

1. **Add a consistent event tracing format** — every event that flows through the system should log these four phases:

   ```
   [TRACE] event received   → event_type | sequence_id | summary
   [TRACE] event applied    → event_type | sequence_id | state before → state after
   [TRACE] state updated    → field changed | old value → new value
   [TRACE] broadcast sent   → event_type | sequence_id | recipient count
   ```

2. **Add an event counter per connection** — each WebSocket handler should track:

   - How many messages it received
   - How many it broadcast
   - When it connected/disconnected

   Print a summary when the client disconnects:
   ```
   [INFO] Client 127.0.0.1:54321 disconnected. Sent: 12, Received broadcasts: 45, Duration: 34s
   ```

3. **Log lock acquisition** (at debug level) — wrap state and DB lock acquisitions with:
   ```
   [DEBUG] Acquiring state lock...
   [DEBUG] State lock acquired (contention: N)
   ```
   Use an atomic counter to track how many times a task waited for a lock.

4. **Add a `GET /debug/trace` endpoint** that returns the last 50 log lines from an in-memory ring buffer.

## Constraints

- Do NOT add a logging framework (no `log`, `env_logger`, `tracing`) — use `println!`/`eprintln!` with consistent prefixes
- Do NOT change the control flow of existing code — only add logging
- Do NOT log message bodies in production traces (log length and sender only)
- Keep the ring buffer simple — a `VecDeque<String>` behind `Arc<Mutex<...>>` is fine
- The ring buffer should be configurable in size (default 1000 lines)

## Suggested Implementation Notes

- Create a `tracer` module: `src/tracer/mod.rs` with a `TraceLogger` struct
- Use an `Arc<Mutex<VecDeque<String>>>` shared across the server
- Pass it through `handle_connection` and `handle_websocket` like `state` and `db`
- Add an atomic `AtomicU64` for lock contention tracking
- Wrap state lock acquisitions with a helper function:
  ```rust
  async fn with_state<F, T>(state: &Arc<Mutex<ServerState>>, tracer: &TraceLogger, f: F) -> T
  where F: FnOnce(&mut ServerState) -> T
  ```
- The ring buffer replaces the `format_events_body` function's role for live debugging

## Acceptance Criteria

- Every WebSocket message lifecycle logs 4 trace lines (received, applied, state updated, broadcast sent)
- Client disconnect prints a summary
- `GET /debug/trace` returns the last N log entries
- Lock contention counter increments when contention occurs
- All existing tests still pass

## Recommended Reading

- Rust `VecDeque` docs
- `std::sync::atomic::AtomicU64` docs
- The lock acquisition pattern in `src/websocket/mod.rs` (lines 27-43, 67-73, etc.)

## Manual Verification Steps

1. Start the server with `RUST_LOG=debug` (or just run `cargo run` and observe TRACE output)
2. Connect a websocat client, send a few messages
3. Hit `GET /debug/trace` — verify all four phases are logged per message
4. Disconnect the client — verify disconnect summary appears
5. Connect 3 clients simultaneously — verify contention counter increases
