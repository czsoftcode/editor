---
id: T04
parent: S05
milestone: M001
provides:
  - Variant-aware terminal light paleta — panel_fill blending podle aktivní light varianty
  - Variant-aware git status barvy v light mode — tonální adaptace pro M/A/D/?? přes git_color_for_visuals()
  - Regresní TDD testy pokrývající WarmIvory/CoolGray/Sepia odlišnost a dark mode stabilitu
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 15min
verification_result: passed
completed_at: 2026-03-05
blocker_discovered: false
---
# T04: 03-light-varianty-settings-ui 04

**# Phase 3 Plan 04: Light Variant Terminal + Git Tone Summary**

## What Happened

# Phase 3 Plan 04: Light Variant Terminal + Git Tone Summary

**Variant-aware tonální adaptace terminálové palety a git statusů pro tři light varianty (WarmIvory/CoolGray/Sepia) s regresními TDD testy a dark mode beze změn.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-03-05T00:00:00Z
- **Completed:** 2026-03-05T00:15:00Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Terminal `ColorPalette` se jemně mění s každou light variantou díky `panel_fill` blendingu (background ~42%, ANSI barvy ~18%)
- Git statusy M/A/D/?? v light mode jsou tonálně přizpůsobeny aktivní variantě přes `git_color_for_visuals(status, visuals)`
- `resolve_file_tree_git_color` v render.rs volá variant-aware resolver — file tree automaticky odráží zvolenou variantu
- 23 regresních testů pokrývá: luminanci pozadí, kontrast fg/bg, odlišnost 3 variant, dark mode stabilitu

## Task Commits

Každý task byl commitován atomicky:

1. **Task 1: Terminal light paleta (RED)** - `d22adee` (test)
2. **Task 1: Terminal light paleta (GREEN)** - `c3a7f93` (feat)
3. **Task 2: Git status light paleta (RED)** - `92eb27c` (test)
4. **Task 2: Git status light paleta (GREEN)** - `68aa46b` (feat)
5. **Task 3: End-to-end smoke testy** - `0ad2453` (test)

_TDD tasky mají dvojité commity (test RED → feat GREEN)._

## Files Created/Modified

- `src/app/ui/terminal/instance/theme.rs` — `tone_light_palette()` s panel_fill blendingem; 8 TDD testů
- `src/app/ui/git_status.rs` — `git_color_for_visuals()` s dvojitým tonálním posuvem; 10 testů
- `src/app/ui/file_tree/render.rs` — `resolve_file_tree_git_color()` volá variant-aware resolver; 5 testů

## Decisions Made

- Terminal blending: background dostane nejvyšší podíl (0.42) pro viditelný tonální rozdíl; ANSI barvy jen 0.18 pro zachování sémantiky
- Git varianty: Modified 0.20, Added 0.18, Deleted 0.16, Untracked 0.22 — vyšší blend pro méně sytě zabarvené statusy
- Faint blending: sekundární `mix_color(..., faint_bg_color, 0.06)` zajišťuje druhý dimension odlišnosti pro Sepia
- Dark mode: explicitní early return bez dotyku `mix_color` logiky — nulové riziko regresionu

## Deviations from Plan

Žádné — plán byl implementován přesně TDD cyklem RED→GREEN s automatickými commity.

Poznámka: Implementace probíhala jako kontinuace předchozí session; SUMMARY.md vytváříme po ověření, že všechny testy prochází a `cargo check` je zelený.

## Issues Encountered

Žádné — `cargo check` čistý (1 unused import warning v nesouvisejícím modulu `workspace/state/mod.rs`, mimo scope tohoto plánu).

## User Setup Required

Žádné — žádná externa konfigurace není potřeba.

## Next Phase Readiness

- Phase 03 je kompletní: light varianty jsou plně funkční ve všech částech UI (syntax highlighting, terminal, git statusy, file tree)
- Projekt je připraven pro volitelné pokročilé funkce (LSP, minimap) nebo vydání
- Manuální GUI smoke test (přepínání variant v živém editoru) doporučen při UAT

---
*Phase: 03-light-varianty-settings-ui*
*Completed: 2026-03-05*
