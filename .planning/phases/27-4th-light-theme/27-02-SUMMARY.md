---
phase: 27-4th-light-theme
plan: 02
subsystem: ui
tags: [theme, settings, tests, i18n]

# Dependency graph
requires:
  - phase: 27-01
    provides: WarmTan variant, swatches, and i18n keys
provides:
  - Deterministic WarmTan picker iteration plus swatch/label coverage
  - Runtime & persistence verification for WarmTan plus localization guard
affects: [settings, theme picker, localization]

# Tech tracking
tech-stack:
  added: []
  patterns: [deterministic selection constants, localized regression tests]

key-files:
  created: []
  modified:
    - src/app/ui/workspace/modal_dialogs/settings.rs
    - src/settings.rs

key-decisions:
  - "Use LIGHT_VARIANT_OPTIONS so the light picker iteration cannot silently drop WarmTan."
  - "Guard WarmTan visuals, persistence, and localization with targeted regression tests."

patterns-established:
  - "Test the UI-visible variant list to prevent disappearance of a built-in theme."
  - "Manual checklist that verifies visibility, runtime apply, persistence, and localized label before closing the plan."

requirements-completed: [THEME-01, THEME-02, THEME-03, THEME-04]

# Metrics
duration: 40 min
completed: 2026-03-11
---

# Phase 27 Plan 2: 4th Light Theme Summary

**WarmTan is again selectable in Settings with deterministic UI coverage, immediate visuals, persistence, and a localized label regression guard.**

## Performance
- **Duration:** 40 min
- **Started:** 2026-03-11T00:20:00Z
- **Completed:** 2026-03-11T00:58:42Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- Light variant rendering now walks the shared `LIGHT_VARIANT_OPTIONS` array, and `settings_light_variant_picker_includes_warmtan` catches regression if WarmTan disappears.
- Added `settings_light_variant_switch_to_warmtan` and `settings_light_variant_warmtan_roundtrip_persistence` so the warm tan visuals and settings.toml round-trip stay aligned with runtime previews.
- `settings_light_variant_label_warmtan_localized` exercises cs/en/de/ru/sk bundles, and manually verifying Settings in light mode confirmed WarmTan is visible, selectable, persisted, and shows the localized label.
- Automated verification: `cargo test -q settings_light_variant_picker_includes_warmtan`, `cargo test -q settings_light_variant_switch_to_warmtan`, `cargo test -q settings_light_variant_warmtan_roundtrip_persistence`, `cargo test -q settings_light_variant_label_warmtan_localized`, and `cargo check` all pass.

## Task Commits
Each task was committed atomically:

1. **Task 27-02-01: Reprodukovat a zafixovat viditelnost WarmTan v Settings pickeru** - `0f78ecc` (feat)
2. **Task 27-02-02: Opravit přepnutí a persistence WarmTan end-to-end** - `c53fb47` (test)
3. **Task 27-02-03: Uzavřít i18n coverage a anti-regression gate pro THEME-04** - `ab5df96` (test)

**Plan metadata:** `ab5df96` (docs: complete plan)

## Files Created/Modified
- `src/app/ui/workspace/modal_dialogs/settings.rs` - Deterministic light variant loop plus picker tests.
- `src/settings.rs` - Visuals/persistence regression tests anchored to WarmTan.

## Decisions Made
- `LIGHT_VARIANT_OPTIONS` keeps WarmTan part of the picker without relying on in-place array literals.
- Targeted regression tests for WarmTan visuals, persistence, and label localization reduce the risk of silent breakage.

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
- `./check.sh` still fails because repo-wide `cargo fmt --check` and `cargo clippy` throw pre-existing warnings (e.g., unused variables, collapsible `if` blocks, manual `is_multiple_of` logic) that are outside this plan’s scope.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
Phase 27 is now closed; Phase 28 (Dark Variant Support) can continue with the regression guard for light themes in place and no outstanding blockers.

---
*Phase: 27-4th-light-theme*
*Completed: 2026-03-11*
