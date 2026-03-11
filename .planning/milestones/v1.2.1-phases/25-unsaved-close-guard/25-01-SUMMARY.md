---
phase: 25-unsaved-close-guard
plan: 01
subsystem: ui
tags: [workspace, editor, unsaved-guard]

# Dependency graph
requires:
  - phase: 24-save-mode-foundation
    provides: SaveMode persistence and manual save routing
provides:
  - Workspace-level pending close flow model for unsaved guard
  - Deterministic dirty tab queue builder for close guard orchestration
affects: [unsaved-close-guard, workspace, editor]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Guard-aware workspace close API", "Deterministic dirty tab queue over PathBuf"]

key-files:
  created: []
  modified:
    - src/app/ui/workspace/state/mod.rs
    - src/app/ui/workspace/state/init.rs
    - src/app/ui/terminal/ai_chat/slash.rs
    - src/app/ui/workspace/mod.rs

key-decisions:
  - "Unsaved close guard uses workspace-level PendingCloseFlow with explicit mode and queue index"
  - "Dirty tab queue is deterministic and always starts with the active tab when dirty"

patterns-established:
  - "PendingCloseFlow centralizes unsaved close guard state on WorkspaceState"
  - "request_close_active_tab provides single entry point for guard-aware close requests"

requirements-completed: [GUARD-01, GUARD-03]

# Metrics
duration: 5min
completed: 2026-03-10
---

# Phase 25: Plan 01 Summary

**Workspace-level PendingCloseFlow model and deterministic dirty tab queue for unsaved close guard**

## Performance

- **Duration:** 5 min (approx)
- **Started:** 2026-03-10T15:41:41Z
- **Completed:** 2026-03-10T15:46:41Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added PendingCloseFlow model and PendingCloseMode enum on WorkspaceState
- Implemented deterministic dirty tab queue builder over editor tabs with unit test
- Introduced request_close_active_tab guard-aware API that ignores re-entrant close requests and shortcuts when nothing is dirty

## Task Commits

Each task was committed atomically:

1. **Task 1: PendingCloseFlow model + queue builder** - `9b684bd` (feat)
2. **Task 2: Workspace guard-aware close request API** - `fc27b0e` (feat)

**Plan metadata:** *(will be captured in docs commit at phase level)*

## Files Created/Modified
- `src/app/ui/workspace/state/mod.rs` - Adds PendingCloseFlow model, queue builder, and targeted unit test
- `src/app/ui/workspace/state/init.rs` - Initializes pending_close_flow on WorkspaceState
- `src/app/ui/terminal/ai_chat/slash.rs` - Updates test helper WorkspaceState initializer for new field
- `src/app/ui/workspace/mod.rs` - Adds request_close_active_tab guard-aware close API wired to PendingCloseFlow

## Decisions Made
- Unsaved close guard state lives on WorkspaceState to keep orchestration close to workspace UI logic
- Dirty tab queue building is a pure helper over (PathBuf, modified) pairs for easy unit testing and reuse

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated WorkspaceState test initializer for new field**
- **Found during:** Task 1 (PendingCloseFlow model introduction)
- **Issue:** Existing test helper WorkspaceState initializer in slash.rs missed the new pending_close_flow field, causing compilation failure
- **Fix:** Extended the initializer with pending_close_flow: None to satisfy the new struct shape
- **Files modified:** src/app/ui/terminal/ai_chat/slash.rs
- **Verification:** `cargo test unsaved_close_guard_queue -- --nocapture` and `cargo check` both pass
- **Committed in:** 9b684bd (part of Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Auto-fix was required to keep tests compiling after extending WorkspaceState; no scope creep.

## Issues Encountered
- None beyond the blocking initializer mismatch documented above.

## User Setup Required

None - no external services or configuration required.

## Next Phase Readiness
- Workspace now exposes a centralized PendingCloseFlow model and guard-aware close entry point ready for wiring into UI triggers and dialogs in later plans (25-02, 25-03).
- No known blockers for implementing the actual guard dialog and global close scenarios.

---
*Phase: 25-unsaved-close-guard*
*Completed: 2026-03-10*

## Self-Check: PASSED
