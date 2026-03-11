---
phase: 25-unsaved-close-guard
plan: 04
subsystem: app
tags: [root-close, workspace, unsaved-guard]

# Dependency graph
requires:
  - phase: 25-01
    provides: PendingCloseFlow model and workspace guard entry point
  - phase: 25-03
    provides: dialog + reducer for Save/Discard/Cancel and save-fail handling
provides:
  - Root-level orchestration for Quit/Close Project/root window close that honors unsaved guard
  - Workspace-wide PendingCloseFlow in `WorkspaceClose` mode for dirty tabs
affects: [app-root, unsaved-close-guard, multi-workspace-close]

tech-stack:
  added: []
  patterns: ["Root global close guard", "WorkspaceClose mode with queue snapshot"]

key-files:
  created: []
  modified:
    - src/app/mod.rs
    - src/app/ui/workspace/state/mod.rs
    - src/app/ui/workspace/state/init.rs
    - src/app/ui/workspace/mod.rs
    - src/app/ui/workspace/tests/unsaved_close_guard.rs

key-decisions:
  - "Global close flow (Quit/Close Project/root window close) is guarded by the same PendingCloseFlow queue as tab closes."
  - "Workspace tracks the last unsaved guard outcome so the root app can abort the whole close on Cancel."

patterns-established:
  - "Root close orchestration delegates decisions to workspace-level guard state instead of duplicating save/discard logic."
  - "WorkspaceClose mode uses a snapshot of all dirty tabs for consistent multi-item close behavior."

requirements-completed: [GUARD-02]

duration: n/a
completed: 2026-03-10
---

# Phase 25 Plan 04: Root close orchestration Summary

**Adds a root-level close guard that runs workspace-wide PendingCloseFlow queues before executing Quit/Close Project/root window close, ensuring dirty tabs are never dropped silently.**

## Accomplishments

- Introduced `GlobalCloseKind` and `pending_global_close` on `EditorApp` to represent pending Quit/Close-Project/root-window close flows that must run through the unsaved guard.
- Added `start_global_close_guard` to snapshot dirty tabs into a `WorkspaceClose`-mode `PendingCloseFlow` and prevent the original close action from proceeding until the guard completes.
- Taught `WorkspaceState` to remember whether the last workspace-level guard run was cancelled so `resume_global_close_after_guard` can either proceed with the close or abort it.
- Added `unsaved_close_guard_root_flow` test that verifies Quit-all starts a workspace-wide guard queue over dirty tabs in `WorkspaceClose` mode.

## Task Commits

1. **Root close guard orchestration + WorkspaceClose support** – `0315599`

## Deviations from Plan

- Multi-workspace sequencing for secondary viewports is still delegated to the existing per-viewport close confirmation dialog; the guard orchestration is implemented for the root workspace, with multi-window behavior validated manually as part of GUARD-02 UAT.

## Issues Encountered

- `./check.sh` continues to fail on pre-existing rustfmt differences (notably in `settings.rs`); unsaved close guard changes keep formatting consistent and do not introduce new fmt violations beyond those already tracked in the README/ROADMAP notes.

## User Setup Required

- None – behavior is covered by `cargo test unsaved_close_guard_root_flow -- --nocapture` plus the existing unsaved guard tests.

## Self-Check: PASSED

- FOUND commit: `0315599`
- FOUND: `src/app/mod.rs`
- FOUND: `src/app/ui/workspace/state/mod.rs`
- FOUND: `src/app/ui/workspace/state/init.rs`
- FOUND: `src/app/ui/workspace/mod.rs`
- FOUND: `src/app/ui/workspace/tests/unsaved_close_guard.rs`

