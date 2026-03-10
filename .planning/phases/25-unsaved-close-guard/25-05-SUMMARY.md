---
phase: 25-unsaved-close-guard
plan: 05
subsystem: app+ui
tags: [tests, root-close, unsaved-guard]

# Dependency graph
requires:
  - phase: 25-03
    provides: reducer + save-fail handling for guard dialog
  - phase: 25-04
    provides: root close guard orchestration (Quit/Close Project/root window close)
provides:
  - Regression tests for root close guard behavior and save-fail branches
affects: [unsaved-close-guard, regression, root-flow]

tech-stack:
  added: []
  patterns: ["Guard-focused regression tests", "Reducer behavior coverage for Save/Discard/Cancel"]

key-files:
  created: []
  modified:
    - src/app/mod.rs
    - src/app/ui/workspace/tests/unsaved_close_guard.rs

key-decisions:
  - "Root close guard tests focus on the presence of a WorkspaceClose queue and leave full GUI click-flow to manual UAT."
  - "Reducer-based tests are used to validate Save/Discard/Cancel behavior and save-fail queue semantics without spinning up full egui contexts."

patterns-established:
  - "Guard regression coverage is centralized in `unsaved_close_guard_*` tests rather than scattered across unrelated modules."

requirements-completed: [GUARD-01, GUARD-02, GUARD-03, GUARD-04]

duration: n/a
completed: 2026-03-10
---

# Phase 25 Plan 05: Root guard regression tests Summary

**Adds regression coverage for root close guard orchestration and reinforces reducer behavior for Save/Discard/Cancel and save-fail branches over the unsaved close queue.**

## Accomplishments

- Extended the unsaved guard test suite with `unsaved_close_guard_tab_triggers`, which exercises reducer semantics for Save/Discard/Cancel over multi-item queues, ensuring the queue advances or stops as expected.
- Added `unsaved_close_guard_root_flow` test in `app::tests` to assert that a Quit-all request starts a workspace-wide `PendingCloseFlow` in `WorkspaceClose` mode with the expected dirty queue.
- Re-ran `cargo test unsaved_close_guard -- --nocapture` to validate that queue ordering, modal actions, save-fail behavior, and root flow wiring all remain green together.

## Task Commits

1. **Reducer + root-flow regression coverage for guard** – `329e7e9`, `0315599`

## Deviations from Plan

- Instead of end-to-end GUI automation, tests focus on reducer and orchestration primitives; multi-window and multi-workspace sequences remain in the manual Nyquist validation checklist for Phase 25.

## Issues Encountered

- None within the scope of this plan; existing warnings and fmt differences are tracked separately and unchanged by these tests.

## User Setup Required

- None – tests are part of the regular Rust test suite and run via `cargo test unsaved_close_guard -- --nocapture`.

## Self-Check: PASSED

- FOUND commit: `329e7e9`
- FOUND commit: `0315599`
- FOUND: `src/app/ui/workspace/tests/unsaved_close_guard.rs`
- FOUND: `src/app/mod.rs`

