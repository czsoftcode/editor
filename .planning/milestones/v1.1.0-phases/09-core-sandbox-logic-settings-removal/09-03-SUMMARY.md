---
phase: 09-core-sandbox-logic-settings-removal
plan: 03
subsystem: core
tags: [sandbox, refactoring, cleanup, rust]

# Dependency graph
requires:
  - phase: 09-02
    provides: sandbox structures and logic removal from types.rs and state/mod.rs
provides:
  - sandbox.rs deleted entirely
  - WorkspaceState without sandbox, file_tree_in_sandbox, pending_agent_id fields
  - All ws.sandbox.* references replaced with ws.root_path
  - Sandbox sync/promotion/staged dialogs removed
affects: [10-sandbox-ui-removal]

# Tech tracking
tech-stack:
  added: []
  patterns: [direct root_path usage instead of sandbox indirection]

key-files:
  created: []
  modified:
    - src/app/mod.rs
    - src/app/ui/workspace/state/mod.rs
    - src/app/ui/workspace/state/init.rs
    - src/app/ui/panels.rs
    - src/app/ui/ai_panel.rs
    - src/app/ui/terminal/ai_chat/render.rs
    - src/app/ui/workspace/modal_dialogs/ai.rs

key-decisions:
  - "AI agent now starts directly without sandbox sync plan check"
  - "Terminal tabs use root_path as CWD instead of sandbox.root"

patterns-established:
  - "root_path is the single source of truth for project directory"

requirements-completed: [CORE-01, CORE-02]

# Metrics
duration: 4min
completed: 2026-03-05
---

# Phase 9 Plan 3: Sandbox Module and References Removal Summary

**Deleted sandbox.rs, removed all sandbox fields from WorkspaceState, and replaced every ws.sandbox.* reference with ws.root_path across 7 files**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-05T21:26:06Z
- **Completed:** 2026-03-05T21:29:40Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Deleted src/app/sandbox.rs (305 lines including Sandbox struct, SyncPlan, sync/promote logic, tests)
- Removed sandbox, file_tree_in_sandbox, pending_agent_id fields from WorkspaceState
- Simplified AI agent start flow (no more sync plan check)
- Removed 3 sandbox-related modal dialogs (promotion_success, sync_confirmation, sandbox_staged)

## Task Commits

Each task was committed atomically:

1. **Task 1: Delete sandbox.rs, remove sandbox fields from WorkspaceState and init** - `b269ca9` (feat)
2. **Task 2: Fix all remaining sandbox references and ensure compilation** - `44434b9` (feat)

## Files Created/Modified
- `src/app/sandbox.rs` - DELETED (Sandbox struct, SyncPlan, sync/promote logic)
- `src/app/mod.rs` - Removed `pub mod sandbox;` declaration
- `src/app/ui/workspace/state/mod.rs` - Removed sandbox, file_tree_in_sandbox, pending_agent_id fields
- `src/app/ui/workspace/state/init.rs` - Removed Sandbox::new, sandbox watcher, sandbox field init
- `src/app/ui/panels.rs` - Simplified file tree heading, removed sandbox toggle UI
- `src/app/ui/ai_panel.rs` - Simplified agent start (no sync plan), terminal uses root_path
- `src/app/ui/terminal/ai_chat/render.rs` - Info bar shows root_path instead of sandbox.root
- `src/app/ui/workspace/modal_dialogs/ai.rs` - Removed promotion_success, sync_confirmation, sandbox_staged dialogs

## Decisions Made
- AI agent now starts directly without sandbox sync plan check - sync was only relevant in sandbox mode
- Terminal tabs use root_path as CWD instead of sandbox.root - all work happens in project directory

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- CORE-01 and CORE-02 gaps from verification are fully closed
- Phase 9 sandbox removal is complete
- Only sandbox.rs reference remaining is in modal_dialogs/sandbox.rs behind #[cfg(never)] - Phase 10 will remove it
- 61 tests pass, cargo check clean (warnings only)

---
*Phase: 09-core-sandbox-logic-settings-removal*
*Completed: 2026-03-05*
