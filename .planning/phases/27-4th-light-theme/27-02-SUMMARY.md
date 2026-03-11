---
phase: 27-4th-light-theme
plan: 02
subsystem: ui
tags: [theme, settings, i18n, tests]

# Dependency graph
requires:
  - phase: 27-01
    provides: WarmTan variant, swatch, a lokalizace pro light varianty
provides:
  - Light variant picker je viditelný i v dark režimu (disabled), WarmTan nezmizí
  - Okamžitý preview při přepínání dark/light díky rozšířenému fingerprintu
  - Ověřená lokalizace a persistence WarmTan přes regresní testy
affects: [settings, theme picker, localization]

# Tech tracking
tech-stack:
  added: []
  patterns: [theme_fingerprint zahrnuje dark_theme, disabled light picker v dark režimu]

key-files:
  created: []
  modified:
    - src/app/ui/workspace/modal_dialogs/settings.rs

key-decisions:
  - "Light variant picker zůstává vždy viditelný; v dark režimu je pouze disabled."

patterns-established:
  - "Theme preview porovnává i dark_theme, aby se přepnutí aplikovalo okamžitě."

requirements-completed: [THEME-01, THEME-02, THEME-03, THEME-04]

# Metrics
duration: 9 min
completed: 2026-03-11
---

# Phase 27 Plan 02: 4th Light Theme Summary

**Light variant picker je viditelný i v dark režimu (disabled) a přepnutí dark/light teď spouští okamžitý preview, WarmTan zůstává lokalizovaný a persistovaný.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-11T00:00:20Z
- **Completed:** 2026-03-11T00:09:11Z
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments
- Light varianty jsou vždy viditelné v Settings (v dark režimu disabled), takže WarmTan nezmizí z pickeru.
- Theme preview nově bere v úvahu i `dark_theme`, takže přepnutí dark/light se projeví okamžitě.
- Regresní testy pro viditelnost, přepnutí, persistence a lokalizaci WarmTan zůstávají zelené.

## Task Commits

Each task was committed atomically:

1. **Task 27-02-01/02: Viditelnost pickeru + okamžitý preview při přepnutí** - `d5f9191` (fix)
2. **Task 27-02-03: Lokalizace WarmTan** - bez změn kódu (ověřeno testy)

## Files Created/Modified
- `src/app/ui/workspace/modal_dialogs/settings.rs` - light picker vždy viditelný, fingerprint zahrnuje dark_theme

## Decisions Made
- Light variant picker zůstává viditelný i v dark režimu, ale je disabled, aby byl seznam variant stále patrný.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `./check.sh` padá na repo-wide clippy warnings mimo rozsah tohoto plánu (např. unused variables a collapsible if). Kód z této změny je bez nových lintů.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
Phase 27 zůstává uzavřená; regresní guardy pro WarmTan jsou platné. Žádné nové blokery.

---
*Phase: 27-4th-light-theme*
*Completed: 2026-03-11*
