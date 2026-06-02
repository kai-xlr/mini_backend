---
name: "Week 5 — Issue 2: Add Conflict Resolution Strategy"
about: High Energy (Day 3)
title: "Basic Conflict Resolution Strategy"
labels: ["week-5", "high-energy", "conflict-resolution"]
assignees: []
---

## Goal

Define what happens when state conflicts occur. Choose one simple strategy and implement it. The goal is not perfect distributed consensus — it's a predictable, documented policy for resolving ordering ambiguity.

## Background

When multiple events arrive "at the same time" (or close enough), the system needs a rule to decide which one wins. Without one, the result depends on thread scheduling and network timing — which means it's unpredictable.

You will pick ONE strategy and implement it:

- **Last-write-wins**: Later timestamp overwrites earlier
- **Sequence-based ordering**: Sequence ID determines authoritative order
- **Timestamp ordering**: Wall-clock time determines order (with tiebreaker)
- **Duplicate suppression**: Identical events are collapsed into one

Sequence-based ordering is the simplest given the existing architecture.

## Requirements

1. **Choose and document your strategy** in a brief comment at the top of `src/state/mod.rs`

2. **Implement a `resolve_conflicts` method** on `ServerState` that:

   - Takes a list of incoming events
   - Applies your chosen strategy to merge them with existing state
   - Returns the resolved event list

3. **Add a `POST /conflicts/resolve` endpoint** that:

   - Accepts a JSON body of conflicting events (you define the format — use a simple text format, no serde dependency needed)
   - Runs the resolution
   - Returns the resolved result

4. **Unit tests** covering:

   - Two conflicting events with different timestamps — correct one wins
   - Duplicate events — correctly suppressed (if that's your strategy)
   - Empty input — no crash
   - Events with same timestamp but different content — tiebreaker works

## Constraints

- Do NOT add serde, JSON parsing, or any new dependencies
- Use a simple text-based format (e.g., `event_type|sequence_id|timestamp|details` per line)
- Do NOT implement CRDTs or vector clocks
- Do NOT change existing event processing in the WebSocket handler — this is a separate resolution endpoint
- Keep the logic under 60 lines

## Suggested Implementation Notes

- Parse the POST body with simple string splitting (the same way HTTP headers are parsed)
- A simple strategy: "higher sequence_id wins; if same sequence_id, later timestamp wins"
- The resolution endpoint is purely diagnostic — it doesn't modify the live event store
- You can test the endpoint with `curl`:
  ```bash
  curl -X POST -d "MessageReceived|0|100|alice: hello\nMessageReceived|1|101|bob: world" \
    http://127.0.0.1:8080/conflicts/resolve
  ```

## Acceptance Criteria

- `cargo test` passes (existing + new tests)
- Resolution correctly handles all test cases
- The chosen strategy is documented inline
- `POST /conflicts/resolve` returns the resolved events in order
- You can explain in one sentence why you chose this strategy

## Recommended Reading

- "Last writer wins" conflict resolution (Wikipedia)
- [Event ordering in distributed systems](https://en.wikipedia.org/wiki/Event_ordering)
- The existing `ChatEvent::from_event_store` in `src/events/mod.rs`

## Manual Verification Steps

1. Start the server: `cargo run`
2. Send conflicting events via curl
3. Verify the output matches your strategy's predictions
4. Test boundary cases: same timestamp, same sequence_id
5. Check that existing `/events` and `/events/audit` still work
