# Plan 02: Terminal Batch Writing

**Objective:** Reduce overhead of terminal updates by batching PTY write commands.

**Context:**
- Each `PtyWrite` event currently calls `backend.process_command(Write(bytes))`.
- This involves locking a mutex and parsing ANSI sequences multiple times per frame.

**Tasks:**
1. Modify `src/app/ui/terminal/instance/mod.rs`:
   - Accumulate all `PtyWrite` data from the frame's throttled loop into a single `Vec<u8>`.
   - Call `backend.process_command(BackendCommand::Write(all_bytes))` only once at the end of the loop.

**Verification:**
- Ensure terminal output remains correct (no missing characters or garbled sequences).
- Observe CPU usage during high-volume output compared to the previous state.
