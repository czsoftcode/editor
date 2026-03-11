---
phase: 25-unsaved-close-guard
plan: 06
subsystem: i18n
tags: [locales, unsaved-guard, save-fail]

# Dependency graph
requires:
  - phase: 25-03
    provides: guard dialog + save-fail branch prepared for i18n
provides:
  - Localized guard dialog labels and messages for all supported languages
  - Localized save-fail error message for unsaved close guard flow
affects: [i18n, unsaved-close-guard, save-errors]

tech-stack:
  added: []
  patterns: ["English-first key set with strict parity", "Guard-specific save-fail error key"]

key-files:
  created: []
  modified:
    - locales/en/ui.ftl
    - locales/cs/ui.ftl
    - locales/de/ui.ftl
    - locales/ru/ui.ftl
    - locales/sk/ui.ftl
    - locales/en/errors.ftl
    - locales/cs/errors.ftl
    - locales/de/errors.ftl
    - locales/ru/errors.ftl
    - locales/sk/errors.ftl
    - src/app/ui/workspace/mod.rs

key-decisions:
  - "Unsaved close guard uses dedicated keys (`unsaved-close-guard-*`) instead of overloading generic button labels."
  - "Save-fail branch during close flow uses a specific `unsaved_close_guard_save_failed` error message that can be localized independently of generic save errors."

patterns-established:
  - "All guard dialog buttons and messages are driven from FTL keys with enforced language parity (cs/en/de/ru/sk)."
  - "Workspace routes save-fail inline errors and toasts through a single localized message constructed with filename and underlying reason."

requirements-completed: [GUARD-03, GUARD-04]

duration: n/a
completed: 2026-03-10
---

# Phase 25 Plan 06: i18n for unsaved close guard Summary

**Adds fully localized strings for the unsaved close guard dialog and save-fail branch across all supported locales, and wires the close flow to use the guard-specific save-fail error message.**

## Accomplishments

- Introduced `unsaved-close-guard-title`, `unsaved-close-guard-message`, and button labels for Save/Discard/Cancel in `ui.ftl` for `en`, `cs`, `de`, `ru`, and `sk`, keeping the key set identical across languages.
- Added `unsaved_close_guard_save_failed` to `errors.ftl` in all locales, providing a guard-specific description that includes the filename and underlying error reason.
- Updated `process_unsaved_close_guard_dialog` to build a localized save-fail message via `unsaved_close_guard_save_failed` and use it consistently for both inline error display and deduped error toast.
- Ran `cargo check` and `cargo test unsaved_close_guard -- --nocapture` to confirm that the guard flow builds and tests pass with the new localization wiring.

## Task Commits

1. **Guard dialog and save-fail i18n wiring** – `7c7e56b`

## Deviations from Plan

- `./check.sh` still fails on pre-existing `cargo fmt --check` differences (notably in `settings.rs`); i18n keys and guard wiring do not introduce new parity failures, and language sets remain synchronized across all five locales.

## Issues Encountered

- None specific to localization; existing warnings in unrelated modules are left as-is per project guidelines.

## User Setup Required

- None – localization changes are bundled with the binary and require no configuration beyond choosing a language in Settings.

## Self-Check: PASSED

- FOUND commit: `7c7e56b`
- FOUND: `locales/en/ui.ftl`
- FOUND: `locales/cs/ui.ftl`
- FOUND: `locales/de/ui.ftl`
- FOUND: `locales/ru/ui.ftl`
- FOUND: `locales/sk/ui.ftl`
- FOUND: `locales/en/errors.ftl`
- FOUND: `locales/cs/errors.ftl`
- FOUND: `locales/de/errors.ftl`
- FOUND: `locales/ru/errors.ftl`
- FOUND: `locales/sk/errors.ftl`

