# Plan 04: Typing FPS Cap

**Objective:** Cap the repaint rate while the user is actively typing to avoid 1:1 keystroke-to-repaint ratio that peaks CPU.

**Context:**
- RPNT-04: "FPS cap při psaní (ne bezpodmínečný repaint po každém keystroke)".
- Target: ~30 FPS (33ms) during active typing.

**Tasks:**
1. In `src/app/ui/workspace/mod.rs` (or `WorkspaceState`):
   - Track `last_keystroke_time: Option<Instant>`.
   - In `render_workspace`, check `ctx.input(|i| !i.events.is_empty())` to detect input events.
   - If a keyboard event is detected, update `last_keystroke_time`.
   - If `last_keystroke_time` is within the last 500ms:
     - Use `ctx.request_repaint_after(std::time::Duration::from_millis(33))` to cap at 30 FPS.

**Verification:**
- Typing in the editor should feel smooth but not consume excessive CPU.
- Verify that `request_repaint()` is NOT called on every keystroke in the editor widget itself.
