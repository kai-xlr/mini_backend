---
name: "Week 5 — Issue 5: Debug a Synchronization Bug"
about: Debug Task (non-negotiable)
title: "Intentionally Break + Fix Synchronization"
labels: ["week-5", "debug", "high-energy"]
assignees: []
---

## Goal

Intentionally create a synchronization failure, reproduce it, diagnose it, fix it, and write notes about what you learned. This is the single most important task this week — it's where distributed-systems debugging intuition starts.

## Background

You cannot debug synchronization bugs until you've seen one. Real production bugs are rare, non-deterministic, and hard to reproduce. So you'll create one on purpose, understand exactly what causes it, and fix it deliberately.

## Requirements

1. **Choose one failure mode from this list:**

   - **Duplicate events**: Make it so a single message can produce two identical events in the event store
   - **Race-condition ordering**: Two concurrent messages arrive in different order than they're processed
   - **Stale replay state**: After replay, the state doesn't match what was live
   - **Inconsistent broadcasts**: Some clients get a message, others don't (but should)
   - **Conflicting simultaneous updates**: Two state writes interfere and produce corrupt state

2. **Reproduce it deterministically:**

   - Write a test that consistently triggers the bug (use `#[tokio::test]`)
   - The test must fail ~100% of the time (not flaky)
   - Use concurrent tasks with specific timing

3. **Diagnose it:**

   - Add temporary trace logging (or use the tracer from Issue 4)
   - Identify the root cause in a comment
   - Write a short diagnosis:
     - What caused it?
     - How did you identify it?
     - Why was the test reliable?

4. **Fix it:**

   - Apply the minimal fix (usually 1-5 lines changed)
   - The fix should be obviously correct on inspection
   - The test that previously failed should now pass

5. **Document what you learned:**

   - Add a `docs/debug-notes-week5.md` with:
     - The failure mode you chose
     - How you reproduced it
     - The root cause
     - The fix
     - What you'd look for in a real codebase

## Constraints

- The bug must be in the server code (not in tests)
- The fix must be minimal — no refactoring
- Do NOT use `tokio::time::sleep` or `thread::sleep` to create the race — use cancellation, ordering, or lock timing instead
- The test must be `#[ignore]`-gated so `cargo test` passes without it (but the bug description must be in the code)
- Keep `docs/debug-notes-week5.md` under 200 lines

## Suggested Implementation Notes

- **Easiest failure mode**: Remove the state lock in one of the `record_event` + `save_event` pairs in `websocket.rs`, creating a gap where two concurrent handlers can interleave
- **Another option**: Move `s.add_message()` outside the lock so two handlers can push to the messages vec simultaneously
- **Third option**: In `record_event`, introduce a small `yield_now()` between reading `next_seq` and incrementing it, causing two events to get the same sequence_id
- Use `tokio::task::yield_now()` to force interleaving at specific points
- The fix for most of these is "close the gap" — move operations inside the lock

## Acceptance Criteria

- The bug is documented in `docs/debug-notes-week5.md`
- A test exists that triggers the bug consistently
- The fix is applied and the test passes
- You can explain the root cause without looking at the code
- All other tests still pass

## Recommended Reading

- [Race conditions explained](https://en.wikipedia.org/wiki/Race_condition)
- Tokio `yield_now()` docs
- The lock scopes in `src/websocket/mod.rs` — look for gaps between `state.lock()` and `db.lock()`

## Manual Verification Steps

1. Read the bug documentation
2. Un-ignore the test, run it — confirm it fails
3. Apply the fix (if not already applied)
4. Run the test again — confirm it passes
5. Re-ignore the test and document the intentional bug
