---
name: "Week 5 — Issue 3: Add Synchronization Tests"
about: Medium Energy (Day 4–5, Task 1)
title: "Synchronization Tests for Concurrent Safety"
labels: ["week-5", "medium-energy", "testing"]
assignees: []
---

## Goal

Write focused tests that verify synchronization correctness under concurrent conditions. These tests prove the invariants that must always remain true, no matter how many clients connect and send simultaneously.

## Background

You've simulated concurrent updates (Issue 1) and added a conflict strategy (Issue 2). Now you need targeted tests that verify specific synchronization guarantees. These tests will catch regressions if you change the locking or state management later.

## Requirements

1. **Add to `tests/synchronization.rs`** with these test cases:

   **a. Concurrent connections don't lose clients**
   - Connect 10 clients simultaneously
   - Verify `GET /replay` shows 10 clients (or the event log shows 10 connects)

   **b. Rapid message bursts preserve ordering per client**
   - One client sends 100 messages as fast as possible
   - Verify all 100 arrive and are in order

   **c. Duplicate events are detectable**
   - Write two identical events to the event store directly (via `save_event`)
   - Replay and verify the duplicate is present (for now — just prove you can detect it)

   **d. Reconnect replay consistency**
   - Client connects, sends 3 messages, disconnects
   - Connect a second client, send 1 message
   - Replay from scratch (POST /replay)
   - Verify broadcast messages match: all 4 messages present, client events for both connections

   **e. Concurrent sends + concurrent reads**
   - While clients are sending, another task repeatedly reads GET /events
   - Verify no panics, no corrupted reads, no deadlocks

2. **Document the invariants** in a comment at the top of the test file. At minimum:
   ```
   // Invariants under concurrency:
   // 1. Total events in memory == total events in DB (audit passes)
   // 2. Message order from a single client is preserved
   // 3. All connected clients are tracked (no phantom clients)
   // 4. Replay produces identical state to live session
   ```

## Constraints

- Do NOT change the server code to make tests pass — tests should validate existing behavior
- Do NOT add test dependencies
- Tests must not depend on timing (no `sleep()` calls to "wait for things to happen")
- Each test should verify a specific invariant, not a general "it works"
- Keep tests deterministic — use sequence IDs or event counts, not wall-clock ordering

## Suggested Implementation Notes

- Reuse the test server helper from Issue 1
- For ordering checks, have each client send messages with an incrementing counter: `msg_0`, `msg_1`, `msg_2`...
- For concurrent reads, `tokio::spawn` a reader task alongside writer tasks
- Check `GET /events/audit` as a quick integrity check in each test
- Use `event_store` sequence IDs to verify ordering, not timestamps
- `load_events` returns events ordered by sequence_id — use that as ground truth

## Acceptance Criteria

- `cargo test --test synchronization` passes
- Each test verifies a different invariant
- Tests run in under 10 seconds
- Test failures clearly indicate which invariant was violated
- You can explain "Invariant 1" from memory

## Recommended Reading

- [Testing concurrent code](https://doc.rust-lang.org/book/ch16-05-control-flow.html)
- Existing test patterns in `src/storage/mod.rs` and `src/replay/mod.rs`

## Manual Verification Steps

1. Run `cargo test --test synchronization -- --nocapture`
2. Read the test output — does each test name describe the invariant?
3. Intentionally introduce a bug (e.g., comment out a `save_event` call) — does a test catch it?
4. Restore the bug, run again, confirm all pass
