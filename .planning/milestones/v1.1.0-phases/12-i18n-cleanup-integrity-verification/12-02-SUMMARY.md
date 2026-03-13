---
phase: 12-i18n-cleanup-integrity-verification
plan: 02
subsystem: i18n, build
tags: [rust, i18n, fluent, compile-warnings, cleanup]

# Dependency graph
requires:
  - phase: 11-file-operations-watcher-guard-removal
    provides: sandbox removal from code (exec_in_sandbox renamed, watcher filter updated)
provides:
  - "Zero compile warnings in cargo build"
  - "57 passing tests with 0 failures"
  - "Zero sandbox references in src/ and locales/"
  - "v1.1.0 milestone integrity verified"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: []

key-files:
  created: []
  modified:
    - src/app/ui/workspace/modal_dialogs.rs
    - src/app/ui/workspace/modal_dialogs/settings.rs
    - src/app/ui/background.rs
    - src/app/ui/workspace/mod.rs
    - locales/cs/ui.ftl
    - locales/de/ui.ftl
    - locales/en/ui.ftl
    - locales/ru/ui.ftl
    - locales/sk/ui.ftl
    - locales/cs/ai.ftl
    - locales/de/ai.ftl
    - locales/en/ai.ftl
    - locales/sk/ai.ftl
    - locales/cs/errors.ftl
    - locales/de/errors.ftl
    - locales/en/errors.ftl
    - locales/ru/errors.ftl
    - locales/sk/errors.ftl

key-decisions:
  - "Removed unused re-export instead of keeping for potential future use"
  - "Removed unused eframe::egui import from background.rs after egui_ctx parameter removal"
  - "Cleaned sandbox i18n keys from non-EN locales as blocking fix for test parity"
  - "Removed orphaned settings-safe-mode* and error-safe-mode-blocked keys (unused in code)"

patterns-established: []

requirements-completed: [INT-01, INT-02, INT-03]

# Metrics
duration: 6min
completed: 2026-03-05
---

# Phase 12 Plan 02: Integrity Verification Summary

**Fixed 3 compile warnings (unused import, parameters), removed 310+ sandbox i18n keys/references from all 5 locales, verified 57 tests pass with zero warnings**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-05T22:54:46Z
- **Completed:** 2026-03-05T23:00:41Z
- **Tasks:** 2
- **Files modified:** 18

## Accomplishments
- Removed unused `restore_runtime_settings_from_snapshot` re-export from modal_dialogs.rs
- Removed unused `id_salt` parameter from settings::show() and `egui_ctx` parameter from process_background_events()
- Cleaned 43 sandbox-only i18n keys from CS/DE/RU/SK locales and updated sandbox-referencing values
- Verified: cargo build 0 warnings, cargo test 57 pass/0 fail, 0 sandbox references in src/ and locales/

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix compile warnings** - `5cb59a7` (fix)
2. **Task 1+2: Remove sandbox i18n keys** - `8ffd61b` (fix, deviation Rule 3)

**Plan metadata:** pending (docs: complete plan)

## Files Created/Modified
- `src/app/ui/workspace/modal_dialogs.rs` - Removed unused re-export, updated settings::show() call
- `src/app/ui/workspace/modal_dialogs/settings.rs` - Removed unused id_salt parameter
- `src/app/ui/background.rs` - Removed unused egui_ctx parameter and egui import
- `src/app/ui/workspace/mod.rs` - Updated process_background_events() call
- `locales/*/ui.ftl` - Removed 43 sandbox-only keys, updated sandbox-referencing values
- `locales/*/ai.ftl` - Updated gemini-default-prompt (sandbox -> project)
- `locales/*/errors.ftl` - Removed unused error-safe-mode-blocked key

## Decisions Made
- Removed unused `eframe::egui` import from background.rs after removing the only egui::Context parameter
- Cleaned sandbox i18n keys as part of Task 2 verification fix (deviation Rule 3) since the test `all_lang_keys_match_english` was failing due to key parity mismatch
- Removed orphaned settings-safe-mode* keys (not referenced in any src/ code)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed sandbox i18n keys from non-EN locales**
- **Found during:** Task 2 (Verification)
- **Issue:** Test `all_lang_keys_match_english` failed because CS/DE/RU/SK had 43 sandbox keys not present in EN (EN was already cleaned by prior work). Plan 12-01 (i18n cleanup) was not yet executed.
- **Fix:** Removed 43 sandbox-only keys from all non-EN locales, updated sandbox-referencing values in remaining keys, removed orphaned unused keys
- **Files modified:** All 10 locale .ftl files (ui.ftl, ai.ftl, errors.ftl across 5 languages)
- **Verification:** `cargo test all_lang_keys_match_english` passes, `grep -r "sandbox" locales/` returns 0 results
- **Committed in:** 8ffd61b

**2. [Rule 1 - Bug] Removed unused eframe::egui import**
- **Found during:** Task 1 (Warning fixes)
- **Issue:** After removing `egui_ctx: &egui::Context` parameter, the `use eframe::egui;` import became unused
- **Fix:** Removed the import line
- **Files modified:** src/app/ui/background.rs
- **Verification:** cargo check passes with 0 warnings
- **Committed in:** 5cb59a7

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Locale cleanup was necessary for test verification to pass. The scope expanded to include i18n key cleanup that was originally planned for 12-01.

## Issues Encountered
None beyond the deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- v1.1.0 Sandbox Removal milestone is complete
- Editor compiles cleanly, all tests pass, no sandbox remnants in code or locales
- Plan 12-01 (i18n cleanup) work was absorbed into this plan's execution

---
*Phase: 12-i18n-cleanup-integrity-verification*
*Completed: 2026-03-05*
