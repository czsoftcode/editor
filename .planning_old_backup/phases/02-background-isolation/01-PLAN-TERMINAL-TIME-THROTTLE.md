# Plan 01: Time-based Terminal Throttling

**Objective:** Prevent UI freezes during high terminal output by limiting the time spent processing PTY events per frame.

**Context:**
- Currently, `Terminal::ui` processes a fixed number of events (`config::TERMINAL_MAX_EVENTS_PER_FRAME = 64`).
- Some events (large `PtyWrite`) are much heavier than others.

**Tasks:**
1. Modify `src/app/ui/terminal/instance/mod.rs`:
   - In `Terminal::ui`, record the start time of the PTY processing loop.
   - Stop processing events if more than 2ms have passed OR the event limit is reached.
   - This ensures that even with massive output, the UI remains responsive at high FPS.

**Verification:**
- Run `cat large_file.rs` in the terminal.
- Verify that the UI (scrolling, clicking other panels) remains smooth during the output.
