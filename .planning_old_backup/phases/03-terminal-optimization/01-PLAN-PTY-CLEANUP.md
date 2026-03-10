# Plan 01: PTY Lifecycle & Cleanup

**Objective:** Ensure all processes started within a terminal session (shell and children) are correctly terminated when the terminal tab or application is closed.

**Context:**
- Currently, `kill_process_group` exists but is only called in `drop`.
- When a terminal tab is closed via UI, it might be removed from the list before `drop` is fully processed or might leak if the object is leaked.

**Tasks:**
1. In `src/app/ui/terminal/mod.rs` (or where tabs are handled):
   - Ensure `kill_process_group` is explicitly called when a terminal tab is closed.
2. Verify in `src/app/ui/terminal/instance/mod.rs`:
   - `kill_process_group` correctly handles the process group PID (negative PID in `libc::kill`).

**Verification:**
- Run a long-running command (e.g., `top`) in a terminal tab.
- Close the tab via the UI (X button).
- Run `ps aux | grep top` to ensure the process is gone.
