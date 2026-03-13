---
phase: 01-zaklad
plan: 03
subsystem: ui
tags: [egui, terminal, theming, light-mode]
requires: []
provides:
  - Theme-aware floating terminal frame fill v StandardTerminalWindow
  - Odstranění hardcoded tmavé výplně rámu v light mode
affects: [terminal-ui, theme-switching, uat]
tech-stack:
  added: []
  patterns:
    - "Floating window frame fill odvozovat z ctx.style().visuals místo fixních RGB hodnot"
key-files:
  created: []
  modified:
    - src/app/ui/terminal/window.rs
key-decisions:
  - "Použit `ctx.style().visuals.panel_fill` jako jednotný theme-aware fill pro floating terminálový frame."
  - "Fix držet centralizovaný v `StandardTerminalWindow`, aby se změna automaticky propsala do AI i build floating terminálu."
patterns-established:
  - "Theme-aware styling v runtime: vizuální hodnoty vždy číst z aktuálních `egui::Visuals`."
requirements-completed: [UI-01]
duration: 1 min
completed: 2026-03-04
---

# Phase 01 Plan 03: Floating Terminal Frame Theme Summary

**Floating frame standardního terminálového okna nyní používá runtime theme-aware fill z `egui::Visuals`, takže light mode už nezůstává čistě černý.**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-04T21:18:16Z
- **Completed:** 2026-03-04T21:19:14Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Odstraněn hardcoded `Color32::from_rgb(20, 20, 25)` z `StandardTerminalWindow::show()`.
- Frame fill navázán na `ctx.style().visuals.panel_fill` pro konzistentní dark/light chování.
- Doplněn komentář, že stejnou logiku sdílí AI i build floating terminál.

## Task Commits

Each task was committed atomically:

1. **Task 1: Nahradit hardcoded frame fill theme-aware variantou** - `561a8bb` (fix)
2. **Task 2: Ověřit wiring pro oba floating terminály** - `5635702` (chore)

## Files Created/Modified
- `src/app/ui/terminal/window.rs` - Theme-aware frame fill pro `StandardTerminalWindow`.

## Decisions Made
- `panel_fill` z aktivních visuals je vhodný základ pro floating window frame bez ruční dark hardcode.
- Fix je záměrně lokalizovaný jen do `StandardTerminalWindow`, aby nevznikla duplikace mezi AI/build terminálem.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `cargo check` blokovaný `sccache` v sandboxu**
- **Found during:** Task 1 (Nahradit hardcoded frame fill theme-aware variantou)
- **Issue:** Verifikační `cargo check` padal na `sccache: Operation not permitted (os error 1)`.
- **Fix:** Pro verifikaci použit `RUSTC_WRAPPER=` (obejití sandbox omezení `sccache` wrapperu).
- **Files modified:** none
- **Verification:** `RUSTC_WRAPPER= cargo check` doběhl úspěšně.
- **Committed in:** N/A (runtime verification workaround)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Bez scope creep; odchylka byla pouze verifikační workaround prostředí.

## Issues Encountered
- Sandbox nepovolil `sccache` wrapper pro `cargo check`; kompilace ověřena přes `RUSTC_WRAPPER=`.  

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plan 01-03 je dokončen a ověřen.
- Fáze 01-zaklad má po tomto kroku připravený přechod na další plán (`01-04-PLAN.md`).

---
*Phase: 01-zaklad*
*Completed: 2026-03-04*

## Self-Check: PASSED

- Found `.planning/phases/01-zaklad/01-03-SUMMARY.md`
- Found task commit `561a8bb`
- Found task commit `5635702`
