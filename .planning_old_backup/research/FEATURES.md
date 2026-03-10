# Feature Landscape — egui/eframe CPU/Memory Optimization

**Domain:** Performance optimization for immediate-mode Rust GUI editor
**Researched:** 2026-03-04
**Confidence:** HIGH (core egui patterns), MEDIUM (component-specific), LOW (flagged)

---

## Context

PolyCredo Editor runs on Rust/eframe/egui and exhibits excessive idle CPU load causing
laptop heating. The editor has these known CPU contributors:

- egui render loop (default: fires on every mouse move / at high rate)
- Git status polling every 5 s (on which thread is unclear)
- Autosave timer checking every 500 ms
- FileWatcher/ProjectWatcher via `notify` crate
- `egui_term` terminal emulator (potential busy-loop waiting for output)
- Multi-window viewports (`show_viewport_deferred`)

---

## Table Stakes

Features/changes users need for optimization to be meaningful. Missing any one of
these leaves the problem unsolved or leaves a major heat source running.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Conditional repaint — `request_repaint_after`** | egui default fires on every event; the most impactful single change | Low | Replace any bare `request_repaint()` in idle paths; use `request_repaint_after(Duration)` everywhere a timer or background job triggers a UI wake-up |
| **Baseline profiling before any change** | Prevents fixing wrong hotspot; validates every subsequent optimization | Low-Med | Use `cargo flamegraph` (Linux `perf`) or `samply`; optionally integrate `puffin_egui` for in-app frame profiling |
| **Git polling moved to background thread** | Currently on main thread or triggered every 5 s; blocks/wakes egui render loop | Medium | Spawn dedicated thread; send result via `std::sync::mpsc`; only trigger `request_repaint()` when result differs from previous state |
| **Autosave timer converted to event-driven** | 500 ms busy-check wakes render loop; should only act when document is dirty | Low | Track `last_edit_time: Instant`; replace polling with dirty flag; check only inside normal key-event frame, not on its own timer |
| **Skip drawing for non-focused / minimized viewports** | egui issue #3982 shows 17% CPU when minimized on some systems | Low | Check `ViewportInfo.focused` and `ViewportInfo.minimized` at top of each viewport's update callback; skip all draw calls when both are false |
| **FileWatcher using event-based wake-up, not polling** | `notify` supports both `inotify` (event-based) and poll backends; poll backend spins | Low | Verify `notify` is configured with `RecommendedWatcher` (inotify on Linux); do NOT use `PollWatcher`; on event, call `ctx.request_repaint()` once |
| **egui_term: repaint only on PTY output** | Terminal emulators traditionally busy-loop on PTY; must be wired to repaint-on-data | Medium | egui_term should call `ctx.request_repaint()` only when it reads bytes from PTY; if current integration polls, replace with `mio`/`epoll` wakeup |

---

## Differentiators

Advanced optimizations that go beyond the immediate fix — worth doing in a later pass.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **`very_lazy` eframe feature flag** | Prevents cursor movement over non-hoverable elements from triggering repaint; egui PR #4880 | Low | Add `very_lazy` to `eframe` features in `Cargo.toml`; experimental but merged; works native + web |
| **Puffin in-app profiler (dev-only feature)** | Frame-level CPU visibility without external tooling; pinpoints slow `update()` sections | Medium | `puffin_egui` + `profiling` crates; gate behind `#[cfg(feature = "profiling")]`; call `puffin::GlobalProfiler::lock().new_frame()` each frame |
| **Git status debouncing after file save** | Currently refreshes every 5 s unconditionally; smarter: refresh after Ctrl+S + once on 5 s timer | Low | Combine event-driven refresh (post-save) with longer background interval (30 s); reduces unnecessary `git status` subprocess launches |
| **Syntax highlighter cache invalidation** | `syntect` HighlightLines is expensive; re-highlight only changed lines, not full document | High | Requires per-line dirty tracking; skip unchanged cached spans; major complexity for large files |
| **Scroll-area virtualization** | egui lays out all rows even off-screen; large files cost proportionally | High | Use `ui.add(egui::Label)` only for visible range; compute `first_visible_line` / `last_visible_line` from scroll position |
| **Viewport-count-aware repaint budgeting** | Multi-window: each secondary viewport gets its own repaint budget; idle windows cost nothing | Medium | Per-workspace `last_activity: Instant`; only call `request_repaint()` for that viewport if interaction happened recently |
| **Single background tokio/rayon runtime for all heavy tasks** | Currently git, autosave, watcher, and async file dialog each spawn ad-hoc threads | Medium | Consolidate into one shared `tokio` runtime (or `rayon` pool); prevents thread explosion on large projects |
| **FPS cap for active-but-not-interactive frames** | During typing, 60 fps is excessive; 30 fps is imperceptible for text editing | Low | `ctx.request_repaint_after(Duration::from_millis(33))` after each keystroke instead of `request_repaint()` |
| **Memory: limit syntax highlight cache size** | Unbounded cache grows with open files; LRU eviction after N entries | Low-Med | Use `lru` crate or simple `VecDeque` + `HashMap` combo; evict when `cache.len() > MAX_CACHED_FILES` |

---

## Anti-Features

Things to deliberately NOT do — each causes the problem to be worse or introduces new instability.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Bare `request_repaint()` in timer callbacks** | Forces immediate repaint every time the callback fires; defeats lazy rendering entirely | Use `request_repaint_after(Duration)` with the minimum interval that still feels responsive |
| **Disabling VSync (`NativeOptions { vsync: false }`)** | Removes the only built-in frame cap; app renders at GPU max rate (100+ fps); CPU/GPU both spike | Keep VSync enabled; use `request_repaint_after` to limit logical frame rate instead |
| **Busy-loop in background threads with `thread::sleep(0)` or `yield_now()`** | Spins a core waiting for work; invisible in flamegraph but shows as kernel time | Use blocking channels (`recv()`) or `condvar.wait()` to park threads until work arrives |
| **Calling `git status` (subprocess) on UI thread** | Blocks egui update() until process exits; causes frame drops; hides in flamegraph as "wait" | Always spawn subprocess in background thread; post result via channel |
| **Using `PollWatcher` from `notify` crate** | Polls filesystem on a fixed interval; adds constant CPU background regardless of file activity | Use `RecommendedWatcher` which uses inotify/FSEvents/ReadDirectoryChangesW |
| **Re-highlighting the entire document on each keypress** | `syntect` HighlightLines is O(n) in document size; on large files causes visible lag | Invalidate and re-highlight only from the edited line downward |
| **Spawning a new thread per autosave / git check** | Thread creation is expensive; thread pool starvation on many open windows | Use a persistent background thread or tokio task per concern, not per invocation |
| **Adding heavy new dependencies "just for performance"** | Binary size grows; compilation time grows; may introduce their own overhead | Use std primitives (`mpsc`, `Mutex`, `Condvar`, `Instant`) first; reach for crates only when stdlib is insufficient |

---

## Feature Dependencies

```
Baseline profiling → all other optimizations
  (measure first, fix second)

Conditional repaint (request_repaint_after) ← required by:
  └── Git polling background thread      (must call request_repaint_after on result)
  └── Autosave event-driven              (must call request_repaint_after after save)
  └── FileWatcher event-based wake-up   (must call request_repaint_after on FS event)
  └── egui_term PTY wake-up             (must call request_repaint_after on PTY data)

Skip minimized viewport draw → depends on:
  └── ViewportInfo.focused / minimized fields (egui 0.27+)

very_lazy feature flag → independent, additive

Syntax highlighter cache → depends on:
  └── Per-line dirty tracking infrastructure

Scroll-area virtualization → depends on:
  └── Knowing visible line range from scroll state
  └── Stable line height (fixed-height rows)

Single background runtime → depends on:
  └── Refactoring git, autosave, watcher to use shared channel/runtime
```

---

## MVP Recommendation

Prioritize in this order for maximum CPU reduction with minimum risk:

1. **Baseline profiling** — `cargo flamegraph` run during idle; identify actual hotspots
2. **Conditional repaint** — replace all `request_repaint()` with `request_repaint_after()` in timer/background paths
3. **Skip minimized/unfocused viewports** — three-line change per viewport update callback
4. **Git polling to background thread** — spawn once at workspace open; post via channel; repaint only on change
5. **Autosave dirty-flag only** — remove 500 ms timer; check dirty flag inside keystroke frame
6. **Verify FileWatcher backend** — confirm `RecommendedWatcher`, not `PollWatcher`
7. **egui_term PTY wake-up** — ensure repaint is triggered only on PTY output bytes

Defer:
- Syntax highlighter per-line cache — high complexity, only relevant on very large files
- Scroll virtualization — high complexity, only relevant for 10,000+ line files
- Puffin in-app profiler — useful for ongoing perf work, not urgent for first pass

---

## Sources

- [egui: Reduce CPU Usage Discussion #1261](https://github.com/emilk/egui/discussions/1261) — MEDIUM confidence
- [egui: Expected CPU load of empty window Discussion #4062](https://github.com/emilk/egui/discussions/4062) — MEDIUM confidence
- [egui: Very Lazy Mode PR #4880](https://github.com/emilk/egui/pull/4880) — HIGH confidence (merged)
- [egui: Windows High CPU when minimized Issue #3982](https://github.com/emilk/egui/issues/3982) — HIGH confidence
- [egui Context docs: request_repaint_after](https://docs.rs/egui/latest/egui/struct.Context.html) — HIGH confidence
- [eframe NativeOptions docs](https://docs.rs/eframe/latest/eframe/struct.NativeOptions.html) — HIGH confidence
- [puffin_egui profiler](https://github.com/emilk/puffin_egui) — HIGH confidence
- [egui: 100% CPU wgpu 0.20 Issue #5092](https://github.com/emilk/egui/issues/5092) — MEDIUM confidence
- [egui_term GitHub](https://github.com/Harzu/egui_term) — MEDIUM confidence (implementation details not confirmed)
- [egui: Continuous message processing Discussion #995](https://github.com/emilk/egui/discussions/995) — MEDIUM confidence
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) — HIGH confidence
