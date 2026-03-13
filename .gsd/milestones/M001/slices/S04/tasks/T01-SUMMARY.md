---
id: T01
parent: S04
milestone: M001
provides:
  - terminal theme resolver with explicit light/dark palettes
  - runtime TerminalView.set_theme(...) application in shared Terminal::ui(...)
  - theme-aware terminal scrollbar colors with hover/drag contrast behavior
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 5min
verification_result: passed
completed_at: 2026-03-04
blocker_discovered: false
---
# T01: 02-terminal-git-barvy 01

**# Phase 2 Plan 1: Terminal Theme Runtime + Scrollbar Summary**

## What Happened

# Phase 2 Plan 1: Terminal Theme Runtime + Scrollbar Summary

**Shared terminal wrapper now applies explicit light/dark egui_term palette at runtime and renders theme-aware scrollbar/overlay colors without PTY restart.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-04T20:16:50Z
- **Completed:** 2026-03-04T20:20:59Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- Přidán `theme.rs` resolver (`terminal_palette`, `terminal_theme_for_visuals`) s light-safe ANSI paletou a kontrastními unit testy.
- `Terminal::ui(...)` nyní vždy volá `.set_theme(terminal_theme_for_visuals(ui.visuals()))`, takže dark/light switch se projeví za běhu bez restartu procesu.
- Scrollbar v `render.rs` používá helpery `scrollbar_track_color`/`scrollbar_thumb_color` odvozené z aktivního tématu; hardcoded tmavé konstanty pro scrollbar path byly odstraněny.
- Unfocused terminal overlay/cursor v `mod.rs` už nepoužívá fixní dark barvy, ale visuals-driven varianty čitelné i v light mode.

## Task Commits

Each task was committed atomically:

1. **Task 1: Wave 0 — terminal theme resolver + unit testy**
   - `cdc99fa` (test): failing RED testy pro kontrast a light background
   - `93f1499` (feat): implementace palety + runtime `set_theme(...)`
2. **Task 2: Theme-aware scrollbar + helper testy (TERM-03)**
   - `cb9b70a` (test): failing RED testy pro light/dark scrollbar track
   - `6cd07f6` (feat): theme-aware scrollbar helpery + napojení do renderu
3. **Task 3: Integrace + regresní kontrola obou terminálů (TERM-01, TERM-02, TERM-04)**
   - `6d6cab6` (fix): theme-aware unfocused overlay, bez backend restart flow změn

## Files Created/Modified
- `src/app/ui/terminal/instance/theme.rs` - nový resolver terminal palety a unit testy kontrastu pro light/dark.
- `src/app/ui/terminal/instance/mod.rs` - runtime `set_theme(...)` aplikace a visuals-aware unfocused overlay.
- `src/app/ui/terminal/instance/render.rs` - scrollbar color helpery + testy track/thumb chování.

## Decisions Made
- Runtime theming se drží čistě na `TerminalView` builderu (render path), aby běžící PTY procesy zůstaly nepřerušené.
- Light paleta používá tmavý foreground a upravené yellow/cyan tóny kvůli čitelnosti na světlém backgroundu.
- Scrollbar aktivní stav zvyšuje kontrast pouze barvou (ne šířkou), aby zůstal stabilní layout.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Terminály (Claude i Build) sdílí stejný theme-aware render path a jsou připravené pro další fázi git color vyladění.
- Manuální smoke pro runtime switch (dlouhý proces + přepnutí dark/light) je připraven podle plánu, ale nebyl automatizovatelný v CLI.

## Self-Check: PASSED

- FOUND: `.planning/phases/02-terminal-git-barvy/02-01-SUMMARY.md`
- FOUND commit: `cdc99fa`
- FOUND commit: `93f1499`
- FOUND commit: `cb9b70a`
- FOUND commit: `6cd07f6`
- FOUND commit: `6d6cab6`

---
*Phase: 02-terminal-git-barvy*
*Completed: 2026-03-04*
