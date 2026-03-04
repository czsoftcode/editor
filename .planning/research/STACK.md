# Technology Stack — egui/eframe CPU Optimization

**Project:** PolyCredo Editor — Performance Optimization
**Researched:** 2026-03-04
**Scope:** CPU/memory reduction in existing Rust/eframe/egui desktop editor

---

## Context

This is not a greenfield stack selection. The project uses Rust + eframe/egui (latest confirmed
version is 0.33.x as of 2026). The goal is to identify the specific APIs and patterns that reduce
idle CPU load — not to replace the stack.

Latest confirmed version: **eframe 0.33.3 / egui 0.33.x** (verified via crates.io, 2026).
If the project Cargo.toml pins an older version, upgrading to 0.33.x should be the first step,
as many repaint-related bugs were fixed between 0.22 and 0.33.

---

## Recommended Techniques

### Technique 1: Conditional Repaint via `request_repaint_after`

**API:** `egui::Context::request_repaint_after(duration: Duration)`
**Also:** `egui::Context::request_repaint_after_for(viewport_id, duration)`

**What it does:** Tells eframe "repaint at most after this duration has passed, but only if no other
event triggered an earlier repaint." When the user is actively interacting, winit events trigger
immediate repaints regardless. When idle, the timer controls the repaint cadence.

**Why it works:** eframe's default behaviour is to repaint every time winit fires an event —
including mouse moves anywhere in the focused window. Without `request_repaint_after`, even a
document editor that hasn't changed repaints at 60+ fps whenever the cursor moves.

**Critical bug to know:** In egui 0.22, `request_repaint_after` was ignored when called every
frame (issue #3109 — duration was reset to zero on each call). This was fixed in later versions.
If you see the timeout being ignored, verify you're on 0.29+.

**Correct usage pattern:**
```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Render UI normally...
    self.render_ui(ctx);

    // Only schedule a timed repaint if there's something that changes over time
    // (e.g., git status badge, autosave timer indicator, terminal output)
    // For a fully idle editor with no active timers, don't call this at all.
    if self.has_pending_background_work() {
        ctx.request_repaint_after(Duration::from_millis(200));
    }
    // If nothing pending: no call = eframe sleeps until the next winit event
}
```

**What NOT to do:**
```rust
// BAD: This forces 5fps even when the window is completely idle
// and overrides eframe's natural sleep-on-idle behaviour
ctx.request_repaint_after(Duration::from_millis(200)); // called unconditionally
```

**Confidence:** HIGH — documented in official egui `Context` docs, multiple verified sources.

---

### Technique 2: Per-Timer Conditional Repaint Scheduling

**Applies to:** Git polling (every 5s), autosave timer (every 500ms), terminal watcher.

**Pattern:** Each background "ticker" requests its own repaint only when its deadline arrives,
using the *minimum required* duration. eframe takes the smallest requested duration across all
callers.

**Git polling (currently every 5s):**
```rust
// In workspace update, after checking elapsed time:
if self.git_refresh_needed() {
    self.refresh_git_status(); // runs git command
    self.last_git_check = Instant::now();
}
// Schedule next check without forcing a repaint every frame
ctx.request_repaint_after(Duration::from_secs(5));
```

**Autosave (currently 500ms interval):**
```rust
// Only schedule repaint if file is dirty (has unsaved changes)
if self.is_dirty() {
    ctx.request_repaint_after(Duration::from_millis(500));
}
// If file is clean: no repaint needed, editor is truly idle
```

**Why this matters:** The current code likely calls `request_repaint()` or has unconditional
repaint paths inside these timers. If the git poller runs on the main thread and calls
`request_repaint()` at 5s intervals regardless of whether git status changed, it forces a repaint
even when nothing visible changed. The fix is: only request repaint if the data actually changed.

**Confidence:** HIGH — standard egui pattern, documented in official discussions #342, #995.

---

### Technique 3: Focus-Aware Repaint Throttling

**API:** `ctx.input(|i| i.viewport().focused)` — returns `Option<bool>`

**What it does:** When the window is not focused (user switched to another app), repaint cadence
can be drastically reduced. There is no reason to run at 60fps when the editor is in the
background.

**Pattern:**
```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let focused = ctx.input(|i| i.viewport().focused.unwrap_or(false));

    self.render_ui(ctx);

    // When unfocused: repaint at most once per second (only for background work)
    // When focused but idle: repaint only when background timers need it
    if !focused && self.has_pending_background_work() {
        ctx.request_repaint_after(Duration::from_secs(1));
    }
}
```

**Why it works:** winit fires mouse-move events for all focused windows, even when the mouse is
outside the window bounds (issue #5284 confirms this is by design). Throttling unfocused windows
eliminates this source of unnecessary repaints.

**Pitfall:** Do not call `request_repaint_after(Duration::ZERO)` or plain `request_repaint()` when
unfocused — it negates the throttle.

**Confidence:** HIGH — `ViewportInfo.focused` is part of official egui API, pattern confirmed in
issues #3982, #4956.

---

### Technique 4: Remove `repaint_on_widget_change` (Already Off by Default)

**API:** `ctx.options_mut(|opts| opts.repaint_on_widget_change = false)`

**Status:** This is `false` by default in current egui versions. Do NOT enable it.

**Why it matters:** If somewhere in the codebase this is accidentally set to `true`, it causes
endless repaints whenever any widget changes its layout or ID — which happens constantly in a
text editor. Verify it is not enabled anywhere.

**Confidence:** HIGH — documented in `egui::Options` struct docs.

---

### Technique 5: Disable `accesskit` Feature Flag

**What it does:** The `accesskit` feature enables screen-reader support. It is enabled by default
in eframe and is a known source of significant CPU overhead (issue #4527).

**Cargo.toml change:**
```toml
[dependencies]
eframe = { version = "0.33", default-features = false, features = [
    "default_fonts",
    "glow",          # or "wgpu" — whichever the project uses
    # "accesskit" intentionally omitted
] }
```

**Tradeoff:** Disabling accesskit removes screen-reader accessibility. For a code editor targeting
developers, this is an acceptable tradeoff until the CPU issue is resolved via other means. Can
be re-enabled later if user demand arises.

**Confidence:** MEDIUM — reported and confirmed in issue #4527, but the exact CPU savings depend
on the OS/platform. Worth testing first.

---

### Technique 6: `notify-debouncer-mini` for FileWatcher

**Current crate:** `notify` (raw API or basic API)
**Recommended addition:** `notify-debouncer-mini` or `notify-debouncer-full`

**What it does:** The raw `notify` API fires one event per filesystem change. Some editors (vim,
emacs, cargo) write files in multiple atomic steps (write temp file, rename). Without debouncing,
a single save triggers 3-5 rapid events, each potentially triggering a repaint + tree refresh.

**`notify-debouncer-mini`** collapses events for the same file within a configurable window:
```toml
[dependencies]
notify-debouncer-mini = "0.4"
```

```rust
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::time::Duration;

let (tx, rx) = std::sync::mpsc::channel();
let mut debouncer = new_debouncer(Duration::from_millis(300), tx)?;
debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
```

**Why 300ms:** Fast enough to feel responsive, slow enough to collapse multi-step writes.

**Confidence:** MEDIUM — `notify-debouncer-mini` is the official companion crate for `notify`,
well-documented at docs.rs. The 300ms value is a community recommendation; the project should
validate it against its own save patterns.

---

### Technique 7: Background Thread for Git Polling

**Current situation (likely):** Git polling (`git status`, `git rev-parse HEAD`) called on the
main thread in `update()` every 5s. Even if async, the result may trigger an unconditional
`request_repaint()`.

**Correct pattern:**
```rust
// In a background thread or tokio task:
let new_status = run_git_status(&project_path);
if new_status != cached_git_status {
    cached_git_status = new_status;
    ctx.request_repaint(); // only when data actually changed
}
// No repaint if git status is unchanged
```

**Why this matters:** Running `git status` blocks for 50-200ms on large repos. On the main thread
this causes frame hitches. In the background thread, it avoids blocking but must only signal repaint
when data changes — not on every poll completion.

**Confidence:** MEDIUM — standard pattern, not egui-specific. The egui docs confirm
`ctx.request_repaint()` is safe to call from any thread.

---

### Technique 8: Profiling Before Optimizing

**Tool:** `puffin` + `puffin_egui`

**Why:** Before applying all of the above, measure which component is actually responsible for
the heat. The CPU overhead could be dominated by:
- The render loop itself (egui layout/paint)
- Git polling frequency
- FileWatcher events causing repaints
- `egui_term` terminal emulator busy-looping

**Minimal integration:**
```toml
[dev-dependencies]
puffin = "0.21"
puffin_egui = "0.29"
```

```rust
// In update():
puffin::GlobalProfiler::lock().new_frame();
puffin_egui::profiler_window(ctx);
```

**Overhead:** ~1ns per scope when profiler is off (EmbarkStudios benchmark on M1). Safe to ship
behind a `#[cfg(debug_assertions)]` gate.

**Confidence:** HIGH — puffin is the standard egui profiling tool, integrates natively.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Repaint control | `request_repaint_after` | `request_repaint` everywhere | `request_repaint` forces immediate repaint, no throttling |
| Repaint control | Conditional (only when data changed) | Unconditional timer | Unconditional defeats idle-sleep entirely |
| File watching | `notify-debouncer-mini` | Raw `notify` API | Raw fires 3-5x events per save, each may trigger repaint |
| Git polling | Background thread + change-detect | Main-thread polling | Main-thread blocks UI, and unconditional repaint wastes cycles |
| Profiling | `puffin` | `perf` / `flamegraph` | puffin shows per-frame breakdown inside the UI; better for frame-level diagnosis |
| AccessKit | Disable for now | Keep enabled | Issue #4527 confirms measurable CPU overhead |

---

## Implementation Priority Order

Apply in this order — each step is independently verifiable:

1. **Profile first** (puffin) — identify the actual hotspot before changing anything
2. **Conditional repaint in git poller** — high-frequency, likely unconditional today
3. **Conditional repaint in autosave** — only when `is_dirty == true`
4. **Focus-aware throttling** — unfocused window should rarely repaint
5. **Disable accesskit** — quick Cargo.toml change, measurable win
6. **Debounce FileWatcher** — prevents burst repaints on file saves
7. **Upgrade to eframe 0.33.x** — if not already there; fixes issue #3109

---

## Sources

- [egui::Context docs — request_repaint_after](https://docs.rs/egui/latest/egui/struct.Context.html)
- [egui::Options docs — repaint_on_widget_change](https://docs.rs/egui/latest/egui/struct.Options.html)
- [eframe::App trait docs](https://docs.rs/eframe/latest/eframe/trait.App.html)
- [eframe::NativeOptions docs](https://docs.rs/eframe/latest/eframe/struct.NativeOptions.html)
- [Issue #3109 — request_repaint_after ignored when called each frame](https://github.com/emilk/egui/issues/3109)
- [Issue #3982 — Windows: High CPU when minimized](https://github.com/emilk/egui/issues/3982)
- [Issue #4527 — High CPU when accesskit enabled](https://github.com/emilk/egui/issues/4527)
- [Discussion #1261 — Reduce CPU Usage](https://github.com/emilk/egui/discussions/1261)
- [Discussion #4062 — Expected CPU load of empty window](https://github.com/emilk/egui/discussions/4062)
- [Discussion #4956 — FPS affected by mouse movement](https://github.com/emilk/egui/discussions/4956)
- [Discussion #5284 — Why repaint when mouse not hovering?](https://github.com/emilk/egui/discussions/5284)
- [notify-debouncer-mini docs](https://docs.rs/notify-debouncer-mini/latest/notify_debouncer_mini/)
- [puffin profiler GitHub](https://github.com/EmbarkStudios/puffin)
- [eframe 0.33.3 on crates.io](https://crates.io/crates/eframe)
