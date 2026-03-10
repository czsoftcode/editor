# Plan 03: Throttled Background Repaints

**Objective:** Replace unconditional `request_repaint()` calls in background threads and UI updates with throttled versions.

**Context:**
- 18+ unconditional `request_repaint()` calls found in `src/`.
- Frequent repaints from background threads (sys info, file events, AI responses) cause high CPU.

**Tasks:**
1. **Background UI (`src/app/ui/background.rs`)**:
   - Replace `ui_ctx.request_repaint()` with `ui_ctx.request_repaint_after(std::time::Duration::from_millis(100))`.
2. **AI Chat (`src/app/ui/terminal/ai_chat/render.rs` & `approval.rs`)**:
   - Throttle repaints to 100ms.
3. **Plugin Host (`src/app/registry/plugins/host/*.rs`)**:
   - Replace direct `request_repaint()` with `request_repaint_after(Duration::from_millis(100))`.

**Verification:**
- `cargo check`
- Observe CPU usage during active AI chat or background task.
