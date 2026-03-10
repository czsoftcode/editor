---
phase: 11-file-operations-watcher-guard-removal
plan: 01
subsystem: editor
tags: [rust, egui, file-operations, watcher, settings]

# Dependency graph
requires:
  - phase: 10-ui-state-cleanup
    provides: "Sandbox UI state removed from workspace"
provides:
  - "Editor save/autosave without sandbox path checks or read_only parameter"
  - "Editor UI without sandbox_mode_enabled parameter"
  - "Terminal without sandbox mode functions"
  - "Watcher ignoring entire .polycredo/ directory"
  - "Settings loading without sandbox field migration"
affects: [12-final-cleanup]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Direct file operations without sandbox path filtering"]

key-files:
  created: []
  modified:
    - src/app/ui/editor/files.rs
    - src/app/ui/editor/ui.rs
    - src/app/ui/terminal/mod.rs
    - src/app/ui/workspace/mod.rs
    - src/watcher.rs
    - src/settings.rs
    - src/app/local_history.rs
    - src/app/ui/background.rs
    - src/app/ui/workspace/menubar/mod.rs
    - src/app/ui/workspace/modal_dialogs/conflict.rs

key-decisions:
  - "Removed read_only parameter entirely from save/autosave/save_path - all callers passed false"
  - "Replaced sandbox-dependent is_readonly block with constant false (preserving downstream interface)"
  - "Watcher now skips entire .polycredo/ directory without sandbox exception"

patterns-established:
  - "File save operations have no path-based access control"

requirements-completed: [FILE-01, FILE-02, FILE-03, WATCH-01, WATCH-02]

# Metrics
duration: 5min
completed: 2026-03-05
---

# Phase 11 Plan 01: File Operations & Watcher Guard Removal Summary

**Removed sandbox path checks from editor save/autosave, sandbox_mode_enabled from UI, sandbox terminal functions, watcher sandbox filter, and settings migration**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-05T22:27:23Z
- **Completed:** 2026-03-05T22:32:00Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments
- Editor saves files directly without sandbox path validation or read_only guard
- Editor UI no longer receives sandbox_mode_enabled parameter; is_readonly set to constant false
- Terminal sandbox functions (terminal_mode_label, terminal_mode_label_for_workdir, terminal_working_dir) and their tests removed
- Watcher simplified to skip entire .polycredo/ directory instead of allowing sandbox subdirectory
- Settings migration function for sandbox fields removed along with its tests
- All 57 tests pass, zero sandbox references in target files

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove sandbox logic from editor files, editor UI, and terminal** - `0395037` (feat)
2. **Task 2: Remove sandbox filter from watcher, settings migration, and comments** - `07a3af3` (feat)

## Files Created/Modified
- `src/app/ui/editor/files.rs` - Removed sandbox path checks and read_only parameter from save/autosave/save_path
- `src/app/ui/editor/ui.rs` - Removed sandbox_mode_enabled parameter, simplified is_readonly to constant false
- `src/app/ui/terminal/mod.rs` - Removed 3 sandbox functions and 2 tests
- `src/app/ui/workspace/mod.rs` - Updated editor.ui() call and save() call to match new signatures
- `src/app/ui/background.rs` - Updated try_autosave() call to match new signature
- `src/app/ui/workspace/menubar/mod.rs` - Updated save() call to match new signature
- `src/app/ui/workspace/modal_dialogs/conflict.rs` - Updated save_path() call to match new signature
- `src/watcher.rs` - Simplified to skip entire .polycredo/ directory
- `src/settings.rs` - Removed migrate_remove_sandbox_fields() and 2 migration tests
- `src/app/local_history.rs` - Updated comment to remove sandbox reference

## Decisions Made
- Removed read_only parameter entirely from save/autosave/save_path since all callers passed false - no sandbox mode means no read-only guard needed
- Kept is_readonly as `false` constant in editor UI rather than removing the variable entirely, to minimize downstream changes in ui_markdown_split and ui_normal which accept it as parameter
- Watcher now treats entire .polycredo/ as ignored - no need for sandbox exception since sandbox no longer exists

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated additional callers not listed in plan**
- **Found during:** Task 1
- **Issue:** Plan mentioned workspace/mod.rs caller but not background.rs, menubar/mod.rs, and conflict.rs callers which also passed read_only/false to save functions
- **Fix:** Updated all 4 callers to match new signatures without read_only parameter
- **Files modified:** src/app/ui/background.rs, src/app/ui/workspace/menubar/mod.rs, src/app/ui/workspace/modal_dialogs/conflict.rs
- **Verification:** cargo check passes
- **Committed in:** 0395037 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary to maintain compilation. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All file operation sandbox guards removed
- Ready for Phase 12 final cleanup if applicable
- Zero sandbox references remain in target files

---
*Phase: 11-file-operations-watcher-guard-removal*
*Completed: 2026-03-05*
