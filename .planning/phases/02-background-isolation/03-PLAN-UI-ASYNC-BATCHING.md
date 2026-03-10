# Plan 03: App Action Batching

**Objective:** Efficiently process application actions without blocking the UI thread.

**Context:**
- `src/app/mod.rs` processes `AppAction` via `process_actions`.
- Currently, it takes all actions from the shared state and processes them in a single frame.

**Tasks:**
1. Modify `src/app/mod.rs`:
   - In `process_actions`, implement a time-limited loop (e.g., 2ms) to process actions.
   - If time runs out, leave remaining actions for the next frame.
   - *Note:* Critical actions (like window management) should still be processed immediately.

**Verification:**
- Verify that UI actions (opening files, AI responses) still feel immediate.
- Ensure no actions are lost when the queue is large.
