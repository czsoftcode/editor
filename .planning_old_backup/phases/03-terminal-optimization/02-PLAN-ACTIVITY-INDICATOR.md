# Plan 02: Terminal Activity Indicator

**Objective:** Provide visual feedback when a terminal tab has new output while it is not focused.

**Context:**
- Users need to know when a background process (like `cargo build` or a running server) has output without switching tabs.

**Tasks:**
1. Modify `Terminal` struct in `src/app/ui/terminal/instance/mod.rs`:
   - Add `pub has_unread_output: bool`.
2. In `Terminal::ui`:
   - Set `has_unread_output = true` if `pty_batch` is not empty and `!focused`.
   - Set `has_unread_output = false` if `focused`.
3. In `src/app/ui/terminal/bottom.rs` (where tabs are rendered):
   - Add a small visual indicator (e.g., a colored dot) next to the terminal tab name if `has_unread_output` is true.

**Verification:**
- Run a command with delayed output (e.g., `sleep 2 && echo done`) and switch to another tab.
- Verify the indicator appears after 2 seconds.
- Switch back to the terminal tab and verify the indicator disappears.
