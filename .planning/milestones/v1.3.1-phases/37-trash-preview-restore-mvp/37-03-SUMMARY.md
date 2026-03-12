---
phase: 37-trash-preview-restore-mvp
plan: 03
subsystem: ui
tags: [trash, restore, conflict-policy, file-tree, tabs, tdd]
requires:
  - phase: 37-01
    provides: trash preview + restore modal flow skeleton
  - phase: 37-02
    provides: conflict modal wiring and restore async handoff base
provides:
  - Engine-level restore conflict policy with deterministic restore-as-copy resolver
  - Post-restore UI handoff from file tree to panel and editor tab sync without auto-open
  - Phase37 requirement tests for conflict policy and preview->restore UI roundtrip
affects: [phase-37, phase-38, restore-flow, trash-preview]
tech-stack:
  added: []
  patterns:
    - TDD red-green commits per task
    - Fail-closed restore conflict handling via explicit policy enum
    - Pending result handoff from file tree into panel-level orchestration
key-files:
  created:
    - tests/phase37_restore_ui_sync.rs
  modified:
    - src/app/trash.rs
    - src/app/ui/file_tree/dialogs.rs
    - src/app/ui/file_tree/mod.rs
    - src/app/ui/panels.rs
    - src/app/ui/editor/tabs.rs
    - tests/phase37_restore_engine.rs
key-decisions:
  - "Conflict restore policy stays engine-owned via RestoreConflictPolicy; UI no longer duplicates restore-as-copy filesystem logic."
  - "Restore success updates existing tabs only via sync_tabs_for_restored_path; no auto-open side effects."
  - "Include-based tests use #[allow(dead_code)] to stay compatible with strict clippy -D warnings in check.sh."
patterns-established:
  - "Restore policy enum in engine: explicit Cancel vs RestoreAsCopy behavior."
  - "FileTreeResult carries restored path for panel orchestration."
requirements-completed: [RESTORE-02, RESTORE-03, TRASHUI-01, RESTORE-01]
duration: 5min
completed: 2026-03-12
---

# Phase 37 Plan 03: Trash Preview Restore MVP Summary

**Restore conflict handling now resolves through deterministic restore-as-copy policy in engine and UI state stays synchronized after restore without opening new tabs.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-12T12:16:16Z
- **Completed:** 2026-03-12T12:20:01Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Added `RestoreConflictPolicy` with deterministic bounded copy-suffix resolver in `trash` engine.
- Replaced duplicate UI restore-as-copy filesystem logic with engine policy call path.
- Added explicit restored-path handoff to panel layer and tab synchronization helper that avoids auto-open.
- Added requirement-focused phase37 sync tests covering conflict-as-copy and preview->restore UI refresh contract.

## Task Commits

Each task was committed atomically:

1. **Task 1: Dokoncit conflict-safe restore as copy v engine** - `d939ac3` (test), `a1e323b` (feat)
2. **Task 2: UI konzistence po restore (reload + highlight + tabs sync)** - `952eb9a` (test), `7ef1232` (feat), `06e55c4` (fix)

## Files Created/Modified
- `tests/phase37_restore_ui_sync.rs` - New phase37 tests for conflict copy policy and UI sync contracts.
- `src/app/trash.rs` - Added restore conflict policy enum, copy destination resolver, and policy-aware restore API.
- `src/app/ui/file_tree/dialogs.rs` - Rewired restore-as-copy action to engine API.
- `src/app/ui/file_tree/mod.rs` - Added restored-path output handoff in `FileTreeResult`.
- `src/app/ui/panels.rs` - Added post-restore tab sync orchestration.
- `src/app/ui/editor/tabs.rs` - Added `sync_tabs_for_restored_path` helper ("restore tab sync") without auto-open behavior.
- `tests/phase37_restore_engine.rs` - Clippy compatibility annotation for include-based trash module.

## Decisions Made
- Engine is the single source of truth for conflict resolution, including restore-as-copy naming policy.
- Panel layer consumes `result.restored` and triggers tab state reconciliation only for already open tabs.
- Strict quality gate (`check.sh`) compatibility was preserved by eliminating dead-code clippy blockers in include-based tests.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] check.sh clippy gate failed on include-based test dead_code**
- **Found during:** Task 2 final verification
- **Issue:** `check.sh` runs clippy with `-D warnings`; include-based tests importing `src/app/trash.rs` triggered dead-code errors.
- **Fix:** Added `#[allow(dead_code)]` on include module declarations and kept formatted `trash.rs` output clean.
- **Files modified:** `tests/phase37_restore_engine.rs`, `tests/phase37_restore_ui_sync.rs`, `src/app/trash.rs`
- **Verification:** `RUSTC_WRAPPER= ./check.sh` PASS
- **Committed in:** `06e55c4`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Needed to satisfy mandatory quality gate; no scope creep.

## Issues Encountered
- Local `sccache` wrapper returned permission error; commands were executed with `RUSTC_WRAPPER=` to keep verification deterministic.

## Authentication Gates
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- RESTORE-02 and RESTORE-03 are now covered by engine and UI contracts with dedicated phase37 tests.
- Phase 38 can focus on watcher stability (`RELIAB-03`) without reopening restore conflict semantics.

## Self-Check
PASSED
- FOUND: `.planning/phases/37-trash-preview-restore-mvp/37-03-SUMMARY.md`
- FOUND commits: `d939ac3`, `a1e323b`, `952eb9a`, `7ef1232`, `06e55c4`

---
*Phase: 37-trash-preview-restore-mvp*
*Completed: 2026-03-12*
