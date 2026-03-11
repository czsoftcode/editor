---
phase: 25-unsaved-close-guard
plan: 02
subsystem: ui
tags: [workspace, editor, unsaved-guard, tab-triggers]

# Dependency graph
requires:
  - phase: 25-01
    provides: PendingCloseFlow model and guard-aware close API
provides:
  - Unified guard-aware routing for all tab-close triggers
  - Integration-lite coverage for Save/Discard/Cancel decisions over the guard queue
affects: [unsaved-close-guard, workspace, editor]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Single entry point for tab close guard", "Reducer-based guard outcome tests"]

key-files:
  created: []
  modified:
    - src/app/ui/workspace/mod.rs
    - src/app/ui/workspace/menubar/mod.rs
    - src/app/ui/editor/ui.rs
    - src/app/ui/editor/tabs.rs
    - src/app/ui/workspace/tests/unsaved_close_guard.rs

key-decisions:
  - "All tab-close triggers (Ctrl+W, menu Close Tab, tab bar close) are funneled through request_close_active_tab and the PendingCloseFlow guard."
  - "Low-level Editor::close_tab and Editor::clear remain internal helpers; user flows never call them directly for dirty tabs."

patterns-established:
  - "Guard-aware tab-close routing is centralized in the workspace layer instead of scattered per trigger."
  - "Unsaved close behavior is tested via reducer outcomes (Save/Discard/Cancel) for a stable queue of pending items."

requirements-completed: [GUARD-01, GUARD-03]

duration: n/a
completed: 2026-03-10
---

# Phase 25 Plan 02: Tab-close trigger routing Summary

**Unifies all tab-close triggers onto the unsaved close guard flow and adds reducer-focused tests for Save/Discard/Cancel decisions over the close queue.**

## Accomplishments

- Wired `Ctrl+W`, menu `Close Tab` and `TabBarAction::Close` to call `request_close_active_tab`, so all user-facing tab-close actions go through the same guard-aware entry point.
- Ensured that clean tabs still close immediately without opening the guard dialog, preserving the fast path for already-saved files.
- Added `unsaved_close_guard_tab_triggers` test to exercise the reducer behavior for Save/Discard/Cancel over a multi-item queue, confirming that Save/Discard advance or finish the flow while Cancel stops it without side effects.

## Task Commits

The plan is covered by the following commits:

1. **Guard-aware workspace close API wiring (shared with Plan 01)** – `fc27b0e`
2. **Tab-close trigger reducer coverage for Save/Discard/Cancel** – `329e7e9`

## Deviations from Plan

- No additional deviations were needed beyond the reducer-level tests; existing guard wiring from Plan 01 already provided a single entry point for tab-close triggers, so this plan focused on tightening test coverage rather than adding new UI plumbing.

## Issues Encountered

- None within this plan; existing unrelated warnings in the codebase were left as-is per project guidelines.

## User Setup Required

- None – all behavior is exercised via `cargo test unsaved_close_guard_tab_triggers -- --nocapture`.

## Self-Check: PASSED

- FOUND: `src/app/ui/workspace/tests/unsaved_close_guard.rs`
- FOUND commit: `fc27b0e`
- FOUND commit: `329e7e9`

