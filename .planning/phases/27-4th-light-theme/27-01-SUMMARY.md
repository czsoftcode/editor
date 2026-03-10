---
phase: 27-4th-light-theme
plan: 01
subsystem: ui
tags: [theme, warmtan, light-variant, i18n]

# Dependency graph
requires:
  - phase: 24-26
    provides: Settings modal UI, Save modes
provides:
  - WarmTan light theme variant (4th light theme)
  - i18n translations in 5 languages
affects: [theme system, settings UI]

# Tech tracking
tech-stack:
  added: []
  patterns: [LightVariant enum pattern, per-variant UI picker]

key-files:
  created: []
  modified:
    - src/settings.rs
    - src/app/ui/workspace/modal_dialogs/settings.rs
    - locales/cs/ui.ftl
    - locales/en/ui.ftl
    - locales/sk/ui.ftl
    - locales/de/ui.ftl
    - locales/ru/ui.ftl
    - src/app/ui/git_status.rs
    - src/app/ui/terminal/instance/theme.rs

key-decisions:
  - "WarmTan selected as 4th light theme name (between Sepia and Brown)"
  - "RGB colors: panel_fill (215,200,185), window_fill (205,190,175), faint_bg_color (195,180,165)"

patterns-established:
  - "Per-variant swatch in settings picker"
  - "i18n key format: settings-light-variant-{name}"

requirements-completed: [THEME-01, THEME-02, THEME-03, THEME-04]

# Metrics
duration: 3min
completed: 2026-03-10
---

# Phase 27 Plan 1: 4th Light Theme Summary

**WarmTan light theme added as 4th variant with i18n support in all 5 languages**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-10T21:09:25Z
- **Completed:** 2026-03-10T21:12:58Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments
- Added WarmTan variant to LightVariant enum in settings.rs
- Implemented to_egui_visuals() match arm with warm tan colors
- Added WarmTan to settings UI picker (label key, swatch, iteration loop)
- Added i18n translations for WarmTan in all 5 languages (cs, en, sk, de, ru)
- Updated test arrays in git_status.rs and terminal/theme.rs

## Task Commits

Each task was committed atomically:

1. **Task 1: Add WarmTan to LightVariant enum** - `d4c6068` (feat)
2. **Task 2: Add WarmTan to settings UI picker** - `3da8d54` (feat)
3. **Task 3: Add i18n translations** - `3da8d54` (feat)

**Plan metadata:** `3da8d54` (docs: complete plan)

## Files Created/Modified
- `src/settings.rs` - Added WarmTan to enum and to_egui_visuals()
- `src/app/ui/workspace/modal_dialogs/settings.rs` - Added label key, swatch, and iteration
- `locales/cs/ui.ftl` - Added "Teplý tan" translation
- `locales/en/ui.ftl` - Added "Warm Tan" translation
- `locales/sk/ui.ftl` - Added "Teplý tan" translation
- `locales/de/ui.ftl` - Added "Warme Tan" translation
- `locales/ru/ui.ftl` - Added "Тёплый тан" translation
- `src/app/ui/git_status.rs` - Updated test array (4 variants)
- `src/app/ui/terminal/instance/theme.rs` - Updated test array (4 variants)

## Decisions Made
- Used "WarmTan" as the variant name (per context decisions)
- Applied RGB colors from context: panel_fill (215,200,185), window_fill (205,190,175), faint_bg_color (195,180,165)
- Used consistent i18n key format: settings-light-variant-warm-tan

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- WarmTan theme is fully implemented and persisted in settings
- Ready for any additional theme work or related features

---
*Phase: 27-4th-light-theme*
*Completed: 2026-03-10*
