# Plan 02: Focus-aware Throttling

**Objective:** Implement throttled repainting when the window is not focused or is minimized.

**Context:**
- `src/app/ui/workspace/mod.rs` is the main entry point for workspace rendering.
- `ctx.input(|i| i.viewport().focused)` can be used to detect focus.
- `ctx.input(|i| i.viewport().minimized)` can be used to detect minimization (if supported by the backend).

**Tasks:**
1. In `src/app/ui/workspace/mod.rs`:
   - Update `render_workspace` to check for focus and minimization.
   - If `!focused` or `minimized`:
     - Use `ctx.request_repaint_after(std::time::Duration::from_secs(2))`.
     - Skip the heavy `has_active_work` check or throttle it further.
   - If `focused`:
     - Keep the existing `has_active_work` check with `config::REPAINT_INTERVAL_MS`.

**Verification:**
- Run the editor and observe CPU usage in idle + unfocused state.
- Ensure UI remains responsive when focused back.
