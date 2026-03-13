---
phase: 25-unsaved-close-guard
plan: 10
subsystem: ui
tags: [egui, unsaved-guard, close-flow, tab-targeting]
requires:
  - phase: 25-unsaved-close-guard
    provides: "Ctrl+W consume + guard input lock from plan 25-08"
provides:
  - "SingleTab queue builder mode targets only one explicit tab"
  - "TabBar close action maps idx to explicit path without active_tab coupling"
  - "SingleTab regressions cover X-click target and Ctrl+W non-iterating behavior"
affects: [workspace, editor-tabs, unsaved-close-guard]
tech-stack:
  added: []
  patterns:
    - "Queue builder explicitně rozlišuje SingleTab vs WorkspaceClose režim"
    - "Close flow používá explicitní target path namísto nepřímého active_tab přepínání"
key-files:
  created: []
  modified:
    - src/app/ui/workspace/state/mod.rs
    - src/app/ui/workspace/mod.rs
    - src/app/ui/workspace/tests/unsaved_close_guard.rs
key-decisions:
  - "`DirtyCloseQueueMode::SingleTab(target)` vrací max. jednu položku pouze pro dirty target tab."
  - "`TabBarAction::Close(idx)` řeší target přes snapshot path a při race (idx mimo rozsah) je bezpečný no-op."
patterns-established:
  - "SingleTab close flow nikdy nesmí používat workspace-wide dirty queue."
requirements-completed: [GUARD-01, GUARD-02, GUARD-03]
duration: 3min
completed: 2026-03-10
---

# Phase 25 Plan 10: SingleTab Close Target Guard Summary

**Unsaved close guard nyní pro SingleTab řeší jen explicitně cílený tab (X i Ctrl+W), zatímco workspace close dál iteruje všechny dirty taby deterministicky.**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-10T18:28:22Z
- **Completed:** 2026-03-10T18:31:33Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Zaveden queue builder mode split (`SingleTab(target)` vs `WorkspaceClose(active-first/all-dirty)`).
- Přepojen close flow v workspace na explicitní target path místo nepřímého `active_tab` přepínání.
- Přepsány regresní testy tak, aby SingleTab flow negarantoval iteraci přes více dirty tabů.

## Task Commits

1. **Task 1: Rozdělení queue builderu podle close mode**
2. `eb6243c` (test, RED)
3. `ea75878` (feat, GREEN)
4. **Task 2: Explicitní target tab pro klik na X i Ctrl+W**
5. `e0e7d3d` (test, RED)
6. `d2770d1` (feat, GREEN)
7. **Task 3: Úprava a doplnění regresních testů pro SingleTab cílení**
8. `843835d` (test, RED)
9. `d2661a3` (test, GREEN)

## Files Created/Modified

- `src/app/ui/workspace/state/mod.rs` - `DirtyCloseQueueMode` + mode-aware queue builder s kompatibilním workspace wrapperem.
- `src/app/ui/workspace/mod.rs` - explicitní target close flow, path resolver pro `TabBarAction::Close(idx)`, safe race no-op.
- `src/app/ui/workspace/tests/unsaved_close_guard.rs` - cílené regresní testy pro tabbar target a SingleTab chování bez multi-item iterace.

## Decisions Made

- SingleTab queue se staví výhradně z explicitního target path (`build_dirty_close_queue_for_mode(SingleTab)`), nikoli ze všech dirty tabů.
- U kliknutí na X je target path odvozen přímo z indexu tabu a při invalid indexu se akce bezpečně ignoruje bez paniky.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `./check.sh` skončil na `cargo fmt --check` kvůli preexistujícím out-of-scope formátovacím odchylkám v jiných souborech. Zapsáno do `.planning/phases/25-unsaved-close-guard/deferred-items.md`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- SingleTab targetování je deterministické a testově pokryté.
- Workspace/root close flow může dál bezpečně používat workspace mode queue bez kolize se SingleTab scénáři.

## Self-Check: PASSED

- Verified summary and key implementation files exist.
- Verified all task commits (`eb6243c`, `ea75878`, `e0e7d3d`, `d2770d1`, `843835d`, `d2661a3`) exist in git history.

---
*Phase: 25-unsaved-close-guard*
*Completed: 2026-03-10*
