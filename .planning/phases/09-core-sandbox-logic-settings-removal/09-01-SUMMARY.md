---
phase: 09-core-sandbox-logic-settings-removal
plan: 01
subsystem: settings
tags: [serde, toml, json, migration, sandbox-removal]

# Dependency graph
requires: []
provides:
  - Settings struct without sandbox_mode field
  - migrate_remove_sandbox_fields() migration function for TOML and JSON
  - All settings.sandbox_mode references replaced with hardcoded false
affects: [09-02, 09-03, 10-sandbox-ui-cleanup]

# Tech tracking
tech-stack:
  added: []
  patterns: [settings-field-migration-on-load]

key-files:
  created: []
  modified:
    - src/settings.rs
    - src/app/mod.rs
    - src/app/ui/workspace/state/init.rs
    - src/app/ui/workspace/mod.rs
    - src/app/ui/workspace/modal_dialogs/settings.rs

key-decisions:
  - "Kept sandbox.rs — removing it breaks too many dependent modules; Plan 02 handles full removal"
  - "Replaced all settings.sandbox_mode references with hardcoded false to maintain compilation"
  - "Removed sandbox_mode tests from modal_dialogs/settings.rs as they test removed functionality"

patterns-established:
  - "Settings field migration: migrate_remove_sandbox_fields() strips legacy fields on load"

requirements-completed: [CORE-01, SET-01, SET-02]

# Metrics
duration: 5min
completed: 2026-03-05
---

# Phase 9 Plan 01: Core Sandbox Logic & Settings Removal Summary

**Removed sandbox_mode from Settings struct with TOML/JSON migration, replaced all references with hardcoded false**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-05T20:48:52Z
- **Completed:** 2026-03-05T20:54:11Z
- **Tasks:** 1
- **Files modified:** 5

## Accomplishments
- Removed sandbox_mode field and project_read_only serde alias from Settings struct
- Added migrate_remove_sandbox_fields() function that strips sandbox_mode and project_read_only from existing TOML and JSON settings files during load
- Replaced all settings.sandbox_mode references across the codebase with hardcoded false
- All 17 settings tests pass, including 2 new migration tests

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove sandbox_mode from Settings + add migration** - `89785c7` (feat)

**Plan metadata:** (pending)

## Files Created/Modified
- `src/settings.rs` - Removed sandbox_mode field, added migrate_remove_sandbox_fields()
- `src/app/mod.rs` - Removed sandbox_mode change detection blocks from settings apply
- `src/app/ui/workspace/state/init.rs` - Replaced settings.sandbox_mode with false
- `src/app/ui/workspace/mod.rs` - Replaced settings.sandbox_mode with false
- `src/app/ui/workspace/modal_dialogs/settings.rs` - Stubbed sandbox_mode checkbox, removed sandbox tests

## Decisions Made
- Kept sandbox.rs and its mod declaration: removing the file breaks too many dependent modules (Sandbox struct used in init.rs, state/mod.rs, modal_dialogs/sandbox.rs, workspace/mod.rs). Plan 02 handles full sandbox module removal.
- Replaced settings.sandbox_mode references with hardcoded `false` instead of leaving broken code: ensures project compiles and tests pass.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Kept sandbox.rs to maintain compilation**
- **Found during:** Task 1 (sandbox.rs deletion)
- **Issue:** Deleting sandbox.rs breaks 6+ files that reference crate::app::sandbox::Sandbox, preventing compilation and test execution
- **Fix:** Restored sandbox.rs and its mod declaration; only removed sandbox_mode from Settings
- **Files modified:** src/app/sandbox.rs (restored), src/app/mod.rs (mod declaration kept)
- **Verification:** cargo test settings::tests passes (17 tests)
- **Committed in:** 89785c7

**2. [Rule 3 - Blocking] Replaced settings.sandbox_mode references with false**
- **Found during:** Task 1 (removing sandbox_mode from Settings)
- **Issue:** 12 references to settings.sandbox_mode across 4 files caused compilation errors
- **Fix:** Replaced with hardcoded false and removed sandbox_mode-dependent tests
- **Files modified:** src/app/mod.rs, src/app/ui/workspace/state/init.rs, src/app/ui/workspace/mod.rs, src/app/ui/workspace/modal_dialogs/settings.rs
- **Verification:** cargo test settings::tests passes (17 tests)
- **Committed in:** 89785c7

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** sandbox.rs kept to prevent widespread breakage. Core deliverables (Settings cleanup + migration) achieved. Plan 02 can now remove sandbox.rs and remaining references.

## Issues Encountered
- Plan contradicted itself: expected project not to compile after task but also expected cargo test to pass. Resolution: fixed all blocking references to maintain compilation.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Settings struct is clean of sandbox_mode — Plan 02 can proceed with removing sandbox.rs and dependent structures
- Migration function ensures existing user settings files are automatically cleaned
- All sandbox_mode UI references are stubbed with false, ready for complete removal

---
*Phase: 09-core-sandbox-logic-settings-removal*
*Completed: 2026-03-05*
