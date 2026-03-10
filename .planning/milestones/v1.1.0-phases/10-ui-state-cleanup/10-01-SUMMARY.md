---
phase: 10-ui-state-cleanup
plan: 01
subsystem: ui
tags: [egui, sandbox-removal, file-tree, settings, cleanup]

# Dependency graph
requires:
  - phase: 09-core-sandbox-removal
    provides: "Settings sandbox_mode field removed, sandbox state fields removed"
provides:
  - "Clean UI without sandbox toggle, tooltip, hint in settings"
  - "File tree with global line count and large file highlighting"
  - "Build bar without Terminal mode label"
  - "Gitignore filter without sandbox keyword"
affects: [11-i18n-cleanup, 12-final-cleanup]

# Tech tracking
tech-stack:
  added: []
  patterns: ["line count as global feature instead of sandbox-only"]

key-files:
  created: []
  modified:
    - src/app/ui/workspace/modal_dialogs.rs
    - src/app/ui/workspace/modal_dialogs/settings.rs
    - src/app/ui/terminal/bottom/build_bar.rs
    - src/app/ui/file_tree/mod.rs
    - src/app/ui/file_tree/render.rs
    - src/app/ui/panels.rs
    - src/app/ui/workspace/state/init.rs

key-decisions:
  - "Line count and large file highlighting promoted from sandbox-only to global features"

patterns-established:
  - "File tree line count runs for all files, not conditionally"

requirements-completed: [UI-01, UI-02, UI-03, UI-04, UI-05, UI-06, STATE-01, STATE-02, STATE-03, STATE-04]

# Metrics
duration: 1min
completed: 2026-03-05
---

# Phase 10 Plan 01: UI State Cleanup Summary

**Removed all sandbox UI elements: settings dialog sandbox block, sandbox modal file, build bar Terminal label, file tree is_sandbox parameter, and gitignore sandbox filter**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-05T21:57:02Z
- **Completed:** 2026-03-05T21:58:20Z
- **Tasks:** 2
- **Files modified:** 7 (+ 1 deleted)

## Accomplishments
- Deleted modal_dialogs/sandbox.rs (80 lines of sandbox-only UI code)
- Removed sandbox checkbox/tooltip/hint block from settings dialog
- Removed Terminal mode label with sandbox hover from build bar
- Promoted file tree line count and large file highlighting to global features
- Removed sandbox keyword from gitignore filter in semantic indexer

## Task Commits

Each task was committed atomically:

1. **Task 1: Smazat sandbox modal a vycistit settings/build_bar/modal_dialogs** - `49cbabb` (feat)
2. **Task 2: Odstranit is_sandbox parametr z file tree** - `952ed91` (feat)

## Files Created/Modified
- `src/app/ui/workspace/modal_dialogs/sandbox.rs` - Deleted (sandbox modal dialog)
- `src/app/ui/workspace/modal_dialogs.rs` - Removed mod sandbox declaration and TODO comment
- `src/app/ui/workspace/modal_dialogs/settings.rs` - Removed sandbox checkbox/tooltip/hint block
- `src/app/ui/terminal/bottom/build_bar.rs` - Removed Terminal mode_label with sandbox hover
- `src/app/ui/workspace/state/init.rs` - Removed "sandbox" from gitignore filter
- `src/app/ui/file_tree/mod.rs` - Removed is_sandbox parameter from ui()
- `src/app/ui/file_tree/render.rs` - Removed is_sandbox parameter from show_node(), made line count global
- `src/app/ui/panels.rs` - Updated file_tree.ui() call to match new signature

## Decisions Made
- Line count calculation and large file highlighting (>=500 lines) promoted from sandbox-only to global features — these are useful for all project views

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All sandbox UI elements removed from the codebase
- Ready for Phase 11 (i18n cleanup) to remove orphaned sandbox i18n keys
- cargo check and cargo test (61 tests) pass cleanly

---
*Phase: 10-ui-state-cleanup*
*Completed: 2026-03-05*

## Self-Check: PASSED
