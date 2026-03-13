---
phase: quick
plan: 4
subsystem: ui/terminal/bottom
tags: [ui, refactor, build-bar]
dependency_graph:
  requires: []
  provides: [combined-build-compile-bar]
  affects: [bottom-panel-layout]
tech_stack:
  added: []
  patterns: [platform-conditional-compilation]
key_files:
  created: []
  modified:
    - src/app/ui/terminal/bottom/build_bar.rs
    - src/app/ui/terminal/bottom/mod.rs
  deleted:
    - src/app/ui/terminal/bottom/compile_bar.rs
decisions: []
metrics:
  duration: 48s
  completed: "2026-03-06T00:25:36Z"
  tasks_completed: 1
  tasks_total: 1
---

# Quick Task 4: Move Compile Bar Next to Build Bar Summary

Consolidated compile platform buttons into build_bar horizontal row after profile dropdown, eliminating separate compile_bar.rs module.

## What Was Done

### Task 1: Move compile buttons into build_bar and remove compile_bar
**Commit:** `9c4b211`

- Inserted platform-conditional compile buttons (`#[cfg(target_os)]`) into `build_bar.rs` after the profile ComboBox, separated by `ui.separator()`
- Linux: "Create .deb" button that spawns build-deb.sh terminal
- Windows/macOS: WIP labels for MSI Installer and DMG Bundle
- Removed `pub mod compile_bar;` from mod.rs
- Removed both `compile_bar::render_compile_bar()` calls (floating window and inline content)
- Deleted `compile_bar.rs` entirely

## Deviations from Plan

None - plan executed exactly as written.

## Verification Results

- `cargo check` passes cleanly
- `grep -r "compile_bar" src/` returns no results
- `compile_bar.rs` confirmed deleted
- `btn-create-deb` found in `build_bar.rs`
