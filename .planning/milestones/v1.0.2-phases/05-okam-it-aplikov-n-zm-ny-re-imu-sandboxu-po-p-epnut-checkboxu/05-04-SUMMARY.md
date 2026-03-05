---
phase: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu
plan: 04
subsystem: ui
tags: [sandbox, panels, toast, remap, cleanup, rust, egui]

# Dependency graph
requires:
  - phase: 05-03
    provides: pending_tab_remap + SandboxRemapTabs toast mechanismus

provides:
  - Automatický cleanup pending_tab_remap po expiraci toastu bez interakce
  - Dokumentační komentář vysvětlující intentional label-timing v apply_sandbox_mode_change

affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Cleanup navázaný na retain(): po odstraňování expired toastů zkontrolovat osiřelá pending_ pole"

key-files:
  created: []
  modified:
    - src/app/ui/panels.rs
    - src/app/ui/workspace/state/mod.rs

key-decisions:
  - "pending_tab_remap se vymaže ihned po retain(), pokud žádný SandboxRemapTabs toast neexistuje — zombie request není možný"
  - "Label terminálu se mění okamžitě při vytvoření nové instance (ne až po exitu starého PTY) — záměrné chování potvrzeno verifikací 05"

patterns-established:
  - "Retain-and-cleanup pattern: po ws.toasts.retain() zkontrolovat stav polí závislých na konkrétních toast akcích"

requirements-completed: [SANDBOX-03]

# Metrics
duration: 5min
completed: 2026-03-05
---

# Phase 05 Plan 04: Gap-closure SANDBOX-03 — remap cleanup a label-timing dokumentace Summary

**Automatický cleanup pending_tab_remap po expiraci SandboxRemapTabs toastu a dokumentace intentional label-timing v apply_sandbox_mode_change()**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-03-05T07:01:00Z
- **Completed:** 2026-03-05T07:06:31Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Cleanup blok v render_toasts() zabraňuje zombie TabRemapRequest — pending_tab_remap se automaticky nulluje pokud toast expiroval bez kliknutí
- Dokumentační komentář v apply_sandbox_mode_change() explicitně popisuje záměrné okamžité nahrazení labelu a poznamenává odchylku od původního locked decision
- Analogický komentář přidán i před blok build_terminal pro konzistenci

## Task Commits

1. **Task 1: Cleanup pending_tab_remap po expiraci toastu** - `f3004ad` (fix)
2. **Task 2: Dokumentační komentář pro label-timing** - `b395c9b` (docs)

## Files Created/Modified

- `src/app/ui/panels.rs` — cleanup pending_tab_remap po retain() v render_toasts()
- `src/app/ui/workspace/state/mod.rs` — dokumentační komentáře v apply_sandbox_mode_change()

## Decisions Made

- Cleanup je umístěn ihned za retain() — dříve než return pro prázdné toasty, aby cleanup proběhl i když jsou toasty prázdné (všechny expiraly najednou)
- Komentář u build_terminal bloku záměrně zkrácen (stačí odkaz na claude_tabs logiku výše)

## Deviations from Plan

None — plán byl proveden přesně podle specifikace.

## Issues Encountered

None.

## User Setup Required

None — žádná externe konfigurace není potřeba.

## Next Phase Readiness

SANDBOX-03 je plně uzavřen. Obě nalezené mezery (zombie remap request + undokumentovaný label-timing) jsou opraveny. Fáze 05 je kompletní.

---
*Phase: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu*
*Completed: 2026-03-05*
