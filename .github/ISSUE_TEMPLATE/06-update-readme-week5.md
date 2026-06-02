---
name: "Week 5 — Issue 6: Update README"
about: Ship Requirement
title: "Update README with Week 5 Features"
labels: ["week-5", "documentation"]
assignees: []
---

## Goal

Update the README to document Week 5 additions: concurrent update simulation, conflict resolution, synchronization tests, event tracing, and synchronization debugging.

## Requirements

1. **Add new endpoints** to the endpoint table:

   - `POST /conflicts/resolve`
   - `GET /debug/trace`

2. **Document the conflict resolution strategy**:

   - Which strategy was chosen and why
   - How it works (1-2 sentences)

3. **Document test structure**:

   ```markdown
   ## Tests
   - `tests/concurrent_updates.rs` — Concurrent send/receive under load
   - `tests/synchronization.rs` — Synchronization invariant verification
   - `src/state/mod.rs` — State unit tests
   - `src/events/mod.rs` — Event parsing roundtrip tests
   - `src/storage/mod.rs` — Persistence CRUD + integrity tests
   - `src/replay/mod.rs` — Event-sourced reconstruction tests
   ```

4. **Add a "Debugging" section** that explains:

   - Event tracing format (received → applied → state updated → broadcast sent)
   - `GET /debug/trace` endpoint
   - How to check lock contention
   - Link to `docs/debug-notes-week5.md` (your debugging journal)

5. **Fix the Project Structure** section — it lists old filenames (`state.rs`, `websocket.rs`, `db.rs` instead of `state/mod.rs`, `websocket/mod.rs`, `storage/mod.rs`).

## Constraints

- Do NOT add tutorials or getting-started guides
- Do NOT remove existing content
- Do NOT exceed 120 lines total
- Keep the same markdown style as the current README

## Acceptance Criteria

- All Week 5 features are documented
- Project structure reflects actual `src/` layout
- Debugging section tells a newcomer how to trace a message through the system
- README fits in < 120 lines
- `cargo test` still passes (README changes don't break code)
