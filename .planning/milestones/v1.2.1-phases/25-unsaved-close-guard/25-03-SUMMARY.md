---
phase: 25-unsaved-close-guard
plan: 03
subsystem: ui
tags: [unsaved-close-guard, dialog, save-flow, egui]
requires:
  - phase: 25-01
    provides: PendingCloseFlow model and queue builder
  - phase: 25-02
    provides: guard-aware close triggers for tabs
provides:
  - Unsaved close guard dialog with Save/Discard/Cancel decisions
  - Reducer-driven close queue with inline error state for save failures
  - Workspace integration that wires dialog decisions to editor close/save behavior
affects: [phase-25-04-root-close-flow, guard-ux, save-fail-behavior]
tech-stack:
  added: []
  patterns: [queue-based guard flow, reducer-driven close decisions, inline-error-with-toast]
key-files:
  created:
    - .planning/phases/25-unsaved-close-guard/25-03-SUMMARY.md
    - src/app/ui/workspace/tests/unsaved_close_guard.rs
  modified:
    - src/app/ui/workspace/mod.rs
    - src/app/ui/dialogs/confirm.rs
key-decisions:
  - "Unsaved close guard uses a reducer (`apply_unsaved_close_decision`) with an explicit queue and inline error field instead of ad-hoc branching."
  - "Closing via Esc, window `X` or backdrop in the guard modal is treated as `Cancel` to guarantee a safe default."
  - "Save failures during close flow keep the current tab open, surface an inline error plus toast, and allow retry or Discard/Cancel without leaving the flow."
patterns-established:
  - "All unsaved close decisions are funneled through `PendingCloseFlow` + reducer, not scattered ad-hoc branches."
  - "Guard dialogs are rendered from workspace using StandardModal and return a typed decision enum used by the reducer."
requirements-completed: [GUARD-03, GUARD-04]
duration: n/a
completed: 2026-03-10
---

# Phase 25 Plan 03: Guard dialog + save-fail handling Summary

**Unsaved close guard dialog with Save/Discard/Cancel decisions wired to a reducer-backed queue, including inline error + toast handling for save failures.**

## Accomplishments

- Added `UnsavedGuardDecision` enum and `show_unsaved_close_guard_dialog` using `StandardModal` with Save/Discard/Cancel actions and a safe default of `Cancel` (including Esc / window close).
- Implemented `UnsavedCloseOutcome` and `apply_unsaved_close_decision` on top of `PendingCloseFlow` to drive queue advancement, cancellation, and save-fail inline error state.
- Integrated the guard into `render_workspace` via `process_unsaved_close_guard_dialog`, which focuses the current tab, runs saves through the existing editor save API, pushes error toasts on failure, and closes tabs only after successful Save/Discard decisions.
- Added `unsaved_close_guard_modal_actions` and `unsaved_close_guard_save_fail` tests to cover reducer behavior for Discard/Cancel and save-fail branches.

## Task Commits

Each task was completed in a single feature commit for this plan:

1. **Task 1: Guard modal UI + rozhodovací větve** – `c510bcb`
2. **Task 2: Save fail handling v close flow** – `c510bcb`

## Deviations from Plan

### Auto-fixed / Clarified Behavior

**1. [Rule 2 - Missing behavior] Handle vanished tabs in pending close queue**
- **Found during:** wiring `process_unsaved_close_guard_dialog` into workspace.
- **Issue:** If a tab was closed externally while still present in `PendingCloseFlow.queue`, the reducer could have been left pointing at a non-existent tab.
- **Fix:** When the current queue path no longer exists in `editor.tabs`, the flow now treats it as discarded and advances the reducer without attempting a save.
- **Files modified:** `src/app/ui/workspace/mod.rs`
- **Verification:** `cargo test unsaved_close_guard -- --nocapture`

_No other deviations from the written plan; dialog layout and decision semantics follow the PLAN.md description._

## Issues Encountered

- None within the scope of this plan; existing unrelated warnings in the codebase were left unchanged per scope boundary.

## User Setup Required

- None – no external configuration or services required for the guard dialog or tests.

## Self-Check: PASSED

- FOUND: `.planning/phases/25-unsaved-close-guard/25-03-SUMMARY.md`
- FOUND commit: `c510bcb`

---
*Phase: 25-unsaved-close-guard*  
*Plan: 03*  
*Completed: 2026-03-10*

