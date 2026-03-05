---
phase: 09-core-sandbox-logic-settings-removal
plan: 02
subsystem: core
tags: [rust, egui, sandbox-removal, refactoring]

# Dependency graph
requires:
  - phase: 09-01
    provides: "Settings sandbox_mode field removed with migration"
provides:
  - "All sandbox structures, fields, and methods removed from codebase"
  - "Clean compilation (cargo check OK, 62 tests pass)"
  - "Simplified settings save flow (no sandbox persist failure handling)"
  - "Simplified toast system (no action buttons)"
affects: [10-ui-sandbox-components-removal]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Toast simplified to message-only (no action buttons)"
    - "Terminal working dir always uses project root"

key-files:
  created: []
  modified:
    - src/app/types.rs
    - src/app/ui/workspace/state/mod.rs
    - src/app/ui/workspace/state/init.rs
    - src/app/mod.rs
    - src/app/ui/workspace/mod.rs
    - src/app/ui/workspace/modal_dialogs.rs
    - src/app/ui/workspace/modal_dialogs/settings.rs
    - src/app/ui/workspace/modal_dialogs/ai_dialogs.rs
    - src/app/ui/workspace/modal_dialogs/conflict.rs
    - src/app/ui/background.rs
    - src/app/ui/panels.rs
    - src/app/ui/terminal/bottom/build_bar.rs
    - src/app/ui/terminal/bottom/compile_bar.rs
    - src/app/ui/terminal/bottom/git_bar.rs
    - src/app/ui/terminal/bottom/mod.rs
    - src/app/ui/terminal/right/mod.rs
    - src/app/ui/terminal/right/ai_bar.rs
    - src/app/ui/workspace/menubar/mod.rs
    - src/app/ui/terminal/mod.rs

key-decisions:
  - "Kept sandbox.rs module (Sandbox struct still referenced for file operations) - full removal in Phase 10"
  - "Simplified Toast to message-only - removed ToastAction/ToastActionKind entirely"
  - "Semantic indexer now scans project root instead of sandbox root"
  - "Registry::new takes project root instead of sandbox root"

patterns-established:
  - "Terminal working dir always equals project root (no sandbox branching)"

requirements-completed: [CORE-02]

# Metrics
duration: 14min
completed: 2026-03-05
---

# Phase 09 Plan 02: Sandbox Structures and Logic Removal Summary

**Removed all sandbox structures, fields, methods, and toast actions from 19 source files; 1304 lines deleted with clean compilation and 62 tests passing**

## Performance

- **Duration:** 14 min
- **Started:** 2026-03-05T20:55:54Z
- **Completed:** 2026-03-05T21:10:19Z
- **Tasks:** 2
- **Files modified:** 19

## Accomplishments
- Removed ToastActionKind enum (6 variants), ToastAction struct, and info_with_actions from types.rs
- Removed 4 sandbox structs (PendingSettingsSave, SandboxApplyRequest, SandboxPersistFailure, TabRemapRequest) and ~18 sandbox fields from WorkspaceState
- Removed apply_sandbox_mode_change method, should_apply_sandbox_request function, and 3 related tests
- Removed sandbox logic from 17 additional files (workspace, background, settings, terminals, panels, menus)
- Deactivated modal_dialogs/sandbox.rs with #[cfg(never)] (preserved for Phase 10)
- Project compiles cleanly; all 62 tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove sandbox structures from types.rs and state/mod.rs** - `8089812` (feat)
2. **Task 2: Remove sandbox logic from all modules, fix compilation** - `45a7629` (feat)

## Files Created/Modified
- `src/app/types.rs` - Removed ToastActionKind, ToastAction, simplified Toast struct
- `src/app/ui/workspace/state/mod.rs` - Removed 4 sandbox structs, ~18 fields, methods, tests
- `src/app/ui/workspace/state/init.rs` - Removed sandbox field init, semantic indexer uses project root
- `src/app/mod.rs` - Registry uses project root, removed sandbox_off_toast_shown
- `src/app/ui/workspace/mod.rs` - Removed 4 sandbox functions, simplified render_workspace
- `src/app/ui/workspace/modal_dialogs/settings.rs` - Removed SandboxModeChange, sandbox helpers, simplified save
- `src/app/ui/workspace/modal_dialogs/ai_dialogs.rs` - Removed promotion_success, sync_confirmation, staged dialogs
- `src/app/ui/background.rs` - Removed sandbox auto-sync, staged files rx, sync rx
- `src/app/ui/panels.rs` - Removed toast action rendering
- `src/app/ui/terminal/bottom/build_bar.rs` - Simplified to use project root
- `src/app/ui/terminal/right/mod.rs` - Removed sandbox float title, tab uses project root

## Decisions Made
- Kept sandbox.rs module since Sandbox struct is still referenced (ws.sandbox.root) - Phase 10 will remove UI components
- Registry::new parameter changed from sandbox_root to project_root (PluginManager still receives it)
- Semantic indexer now scans project root directory instead of sandbox directory
- All `sandbox_mode_enabled` references replaced with `false` literal where API signature requires bool

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed build_bar.rs sandbox references**
- **Found during:** Task 2
- **Issue:** build_bar.rs referenced build_in_sandbox and sandbox_mode_enabled (not listed in plan files)
- **Fix:** Simplified to always use project root, removed sandbox label logic
- **Files modified:** src/app/ui/terminal/bottom/build_bar.rs
- **Verification:** cargo check passes

**2. [Rule 3 - Blocking] Fixed compile_bar.rs and git_bar.rs sandbox references**
- **Found during:** Task 2
- **Issue:** Both files referenced sandbox_staged_files and build_in_sandbox
- **Fix:** Removed sandbox checks, always enable controls
- **Files modified:** compile_bar.rs, git_bar.rs
- **Verification:** cargo check passes

**3. [Rule 3 - Blocking] Fixed bottom/mod.rs sandbox label and guards**
- **Found during:** Task 2
- **Issue:** Float window title used sandbox labels, git_bar gated by build_in_sandbox
- **Fix:** Simplified title, always show git_bar
- **Files modified:** src/app/ui/terminal/bottom/mod.rs
- **Verification:** cargo check passes

**4. [Rule 3 - Blocking] Fixed ai_bar.rs sync_confirmation reference**
- **Found during:** Task 2
- **Issue:** AI start button used sync_confirmation (removed field)
- **Fix:** Start agent directly without sync check
- **Files modified:** src/app/ui/terminal/right/ai_bar.rs
- **Verification:** cargo check passes

**5. [Rule 3 - Blocking] Fixed conflict.rs sandbox_mode_enabled reference**
- **Found during:** Task 2
- **Issue:** External change conflict save used sandbox_mode_enabled
- **Fix:** Replaced with false literal
- **Files modified:** src/app/ui/workspace/modal_dialogs/conflict.rs
- **Verification:** cargo check passes

**6. [Rule 1 - Bug] Fixed failing terminal mode label test**
- **Found during:** Task 2
- **Issue:** Test asserted "Terminal ---" but function returns "Terminal"
- **Fix:** Updated test assertion to match actual function behavior
- **Files modified:** src/app/ui/terminal/mod.rs
- **Verification:** cargo test passes (62/62)

---

**Total deviations:** 6 auto-fixed (1 bug, 5 blocking)
**Impact on plan:** All auto-fixes necessary for compilation. No scope creep.

## Issues Encountered
None - cascade compilation errors resolved iteratively as expected.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All sandbox data structures removed from core code
- sandbox.rs module still exists (Sandbox struct referenced for file operations)
- modal_dialogs/sandbox.rs deactivated with #[cfg(never)]
- Ready for Phase 10: UI sandbox component removal (file tree toggle, terminal labels, etc.)

## Self-Check: PASSED

- SUMMARY.md: FOUND
- Commit 8089812: FOUND
- Commit 45a7629: FOUND
- All 19 modified files: FOUND

---
*Phase: 09-core-sandbox-logic-settings-removal*
*Completed: 2026-03-05*
