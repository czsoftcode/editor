---
phase: 03-light-varianty-settings-ui
plan: 02
subsystem: ui
tags: [egui, settings, i18n, live-preview, theming]
requires:
  - phase: 03-01
    provides: light variant model and visuals mapping in Settings::to_egui_visuals
provides:
  - localized light variant labels in all supported locales
  - conditional card picker for LightVariant in Settings editor category
  - guarded runtime live preview with theme fingerprint checks
  - workspace snapshot slot for upcoming cancel-revert lifecycle
affects: [03-03, settings modal lifecycle, runtime theme propagation]
tech-stack:
  added: []
  patterns:
    - changed()-guarded live preview updates for UI controls
    - explicit theme fingerprint tuple (dark_theme, light_variant)
key-files:
  created: []
  modified:
    - locales/cs/ui.ftl
    - locales/en/ui.ftl
    - locales/de/ui.ftl
    - locales/sk/ui.ftl
    - locales/ru/ui.ftl
    - src/app/ui/workspace/modal_dialogs/settings.rs
    - src/app/ui/workspace/state/mod.rs
    - src/app/ui/workspace/state/init.rs
key-decisions:
  - "Light variant picker is rendered as clickable cards with swatch + label + clear selected state, and is hidden in dark mode."
  - "Live preview runs only on theme control changed() events and only when theme fingerprint actually changes."
  - "WorkspaceState now carries settings_original snapshot metadata for next plan's cancel-revert semantics."
patterns-established:
  - "Settings preview propagation: update shared.settings + bump settings_version to sync root and deferred viewports."
  - "Theme controls use local fingerprint comparison to prevent version bump storms."
requirements-completed: [SETT-01, SETT-02]
duration: 6min
completed: 2026-03-04
---

# Phase 3 Plan 2: Settings Light Variant Picker and Guarded Live Preview Summary

**Settings modal now exposes a light-only 3-card variant picker and applies dark/light + variant changes immediately across viewports with a fingerprint guard to avoid redundant settings_version churn.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-04T22:57:55Z
- **Completed:** 2026-03-04T23:04:02Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments
- Added `settings-light-variant*` translation keys to `cs/en/de/sk/ru` locale files without fallback gaps.
- Implemented conditional card picker for `WarmIvory`, `CoolGray`, `Sepia` including swatch, localized title, and selected border/check state.
- Implemented live preview helpers in Settings modal with `.changed()` + fingerprint guard and runtime `settings_version` propagation.
- Added `settings_original` field to workspace state and initialized snapshot metadata on settings modal open.

## Task Commits

Each task was committed atomically:

1. **Task 1: Rozšířit i18n o labely light variant pickeru** - `8969dfd` (feat)
2. **Task 2: Implementovat conditional kartový picker pouze pro light mode** - `9835ec2` (feat)
3. **Task 3: Live preview s fingerprint guardem** - `bd67dec` (feat)

**Plan metadata:** `(pending docs commit)`

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## Files Created/Modified
- `locales/cs/ui.ftl` - Added Czech labels for light variant picker.
- `locales/en/ui.ftl` - Added English labels for light variant picker.
- `locales/de/ui.ftl` - Added German labels for light variant picker.
- `locales/sk/ui.ftl` - Added Slovak labels for light variant picker.
- `locales/ru/ui.ftl` - Added Russian labels for light variant picker.
- `src/app/ui/workspace/modal_dialogs/settings.rs` - Added light variant card UI and guarded live preview helper flow.
- `src/app/ui/workspace/state/mod.rs` - Added `settings_original` snapshot field for settings lifecycle.
- `src/app/ui/workspace/state/init.rs` - Initialized `settings_original` to `None`.

## Decisions Made
- Live preview uses `theme_fingerprint(draft)` before/after control changes to avoid per-frame version bumping.
- Light variant selection stays in `draft.light_variant` while dark mode is active, so `Light -> Dark -> Light` preserves prior variant.
- Snapshot field was introduced now to keep 03-03 cancel-revert work incremental without refactoring modal ownership.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `cargo check` reports a pre-existing warning (`unused import: eframe::egui` in `state/mod.rs`); functionality unaffected.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Settings modal now has runtime preview hooks and snapshot state required for Save/Cancel revert semantics in 03-03.
- Theme changes propagate across root and deferred viewports through existing `settings_version` path.

## Self-Check: PASSED
- FOUND: `.planning/phases/03-light-varianty-settings-ui/03-02-SUMMARY.md`
- FOUND: `8969dfd`
- FOUND: `9835ec2`
- FOUND: `bd67dec`

---
*Phase: 03-light-varianty-settings-ui*
*Completed: 2026-03-04*
