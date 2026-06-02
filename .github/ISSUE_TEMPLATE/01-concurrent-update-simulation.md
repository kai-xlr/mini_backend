---
name: "Week 5 — Issue 1: Simulate Concurrent Updates"
about: High Energy (Day 1–2)
title: "Concurrent Update Simulation"
labels: ["week-5", "high-energy", "concurrency"]
assignees: []
---

## Goal

Simulate multiple WebSocket clients sending messages simultaneously and verify the server survives without corruption. Learn what happens when ordering guarantees are tested under concurrency pressure.

## Background

Right now, the system assumes events happen cleanly in order. Each message goes through a predictable sequence — received, saved, broadcast. But in production, multiple clients send at the same time, messages overlap, and the order isn't guaranteed.

You're going to write a test that proves your server can handle concurrent updates.

## Requirements

1. Write a **concurrent integration test** (`#[tokio::test]`) that:

   - Spawns N clients (start with N=5) that each send M messages (start with M=3)
   - Clients send messages **simultaneously** (not sequentially — use `tokio::join!` or spawn tasks)
   - Waits for all sends to complete
   - Verifies the server state afterward

2. Verify these invariants after concurrent sends:

   - Total messages received equals total messages broadcast
   - Every client's messages are present in the event log
   - No messages are corrupted (partial, overlapping, or malformed)
   - Event sequence IDs should be contiguous with no gaps

3. The test must run against a **real server** (start it in the test, connect WebSocket clients to it).

## Constraints

- Do NOT add a test framework — use `#[tokio::test]` from the existing tokio dependency
- Do NOT change the server's synchronization model yet — this is an observation task first
- Do NOT add authentication, rate limiting, or rooms
- Keep the test in a new file: `tests/concurrent_updates.rs`
- Use `websocat` or raw `tokio-tungstenite` for the test clients

## Suggested Implementation Notes

- You'll need `tokio` with `rt-multi-thread` feature for concurrent tasks
- Use `tokio::task::spawn` for each client, then `JoinHandle` to wait
- Create a helper that starts the server on a random port (hint: bind to `127.0.0.1:0`)
- For verification, connect a "monitor" client or hit GET /events after all sends complete
- Each client should send unique messages so you can verify all arrived
- Pay attention to the broadcast channel capacity (16) — it may drop messages if you're too fast

## Acceptance Criteria

- `cargo test --test concurrent_updates` passes
- The test creates real concurrent pressure (not sequential sends in a loop)
- Test output clearly shows what was sent vs what was received
- You can explain where lock contention happened

## Recommended Reading

- [Tokio select! docs](https://docs.rs/tokio/latest/tokio/macro.select.html)
- [Shared state in Tokio](https://docs.rs/tokio/latest/tokio/sync/index.html)
- Rust doc for `tokio::sync::Mutex` vs `std::sync::Mutex`

## Manual Verification Steps

1. Run `cargo test --test concurrent_updates -- --nocapture`
2. Read the output — does it show the expected message counts?
3. Manually run the server and connect 3 websocat clients, type rapidly in each
4. Check `GET /events/audit` — is integrity OK?
5. After the test, inspect `chat.db` — are all events present?
