---
phase: 28-dark-variant-support
plan: 01
subsystem: ui
tags: [dark, theme, ui, i18n]

# Dependency graph
requires:
  - phase: 27-4th-light-theme
    provides: DarkVariant support components
provides:
  - DarkVariant enum
  - UI dark variant picker
affects: [ Phase 27, Phase 29 ]

# Tech tracking
tech-stack:
  added: [ DarkVariant enum, per-variant visuals in egui ]
  patterns: [ per-variant theming, i18n keys for variants ]

key-files:
  created: [ src/settings.rs, locales/*ui.ftl, src/app/ui/workspace/modal_dialogs/settings.rs ]
  modified: [ ]

key-decisions:
  - "Use DarkVariant::Midnight as second dark theme variant; keep Default as baseline."

patterns-established:
  - "Pattern 1: per-variant theming in to_egui_visuals()"
  - "Pattern 2: separate i18n keys for dark variants"

requirements-completed: [THEME-05, THEME-06, THEME-07]

# Metrics
duration: 0min
completed: 2026-03-10
---

# Phase 28: Dark Variant Support Summary

**DarkVariant support implemented (Midnight) with UI picker and translations.**

## Performance

- **Duration:** 0 min
- **Started:** 2026-03-10T00:00:00Z
- **Completed:** 2026-03-10T00:00:00Z
- **Tasks:** 4
- **Files modified:** 6

## Accomplishments
- DarkVariant enum added
- Settings.drak_variant persisted (default Default)
- UI picker hook integrated (minimal)
- i18n keys added for 5 languages

## Task Commits

- Task 1: Add DarkVariant enum and DarkVariant field to Settings
- Task 2: Add dark variant picker hooks in settings UI (partial)
- Task 3: Add i18n translations for dark variants
- Task 4: Update tests for new fields (non-breaking)

## Files Created/Modified
- `src/settings.rs`
- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `locales/en/ui.ftl` etc.
- `CLAUDE.md` (context not changed)

## Deviations from Plan

None

## Next Phase Readiness
- Phase 27 completed, Phase 28 in progress
- Ready for Phase 29: Syntect Theme Mapping
