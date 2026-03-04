# Plan 03: Path Cache Optimization

**Objective:** Reduce CPU usage during terminal interaction by optimizing clickable path detection.

**Context:**
- Currently, path detection (regex) runs whenever the mouse moves over the terminal.
- While it only parses the line under the cursor, it could be more efficient by caching the results for a specific grid line state.

**Tasks:**
1. In `src/app/ui/terminal/instance/mod.rs`:
   - Improve `path_cache` to include a hash or timestamp of the line content.
   - Only re-run the regex if the line content or its position in the grid has changed.

**Verification:**
- Ensure clickable paths still work correctly (navigate to file:line:col).
- Moving the mouse rapidly over a terminal with lots of text should not cause CPU spikes.
