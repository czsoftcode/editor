---
phase: 03-light-varianty-settings-ui
plan: 04
subsystem: ui
tags: [egui, terminal, git, light-mode, variant, tonality, color-palette]

# Dependency graph
requires:
  - phase: 03-01-light-varianty-settings-ui
    provides: LightVariant enum a Settings::to_egui_visuals() s WarmIvory/CoolGray/Sepia větvením
  - phase: 02-terminal-git-barvy
    provides: terminal_theme_for_visuals(), git_color_for_mode(), resolve_file_tree_git_color() z Phase 2
provides:
  - Variant-aware terminal light paleta — panel_fill blending podle aktivní light varianty
  - Variant-aware git status barvy v light mode — tonální adaptace pro M/A/D/?? přes git_color_for_visuals()
  - Regresní TDD testy pokrývající WarmIvory/CoolGray/Sepia odlišnost a dark mode stabilitu
affects: [budoucí light-mode rozšíření, terminal theme pipeline, file tree git render]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Tonální blending: mix_color(base, visuals.panel_fill, t) pro jemnou variantní adaptaci"
    - "TDD RED→GREEN cyklus: nejdříve failing testy, pak implementace bez přerušení pipeline"
    - "Variant-aware resolver: git_color_for_visuals(status, visuals) odvozuje tón z panel_fill + faint_bg_color"

key-files:
  created: []
  modified:
    - src/app/ui/terminal/instance/theme.rs
    - src/app/ui/git_status.rs
    - src/app/ui/file_tree/render.rs

key-decisions:
  - "Terminal variant-aware tón: blending panel_fill do celé ColorPalette (0.06–0.42 dle složky) bez restartu PTY backendu"
  - "Git light variant tón: mix_color s panel_fill (0.16–0.22) + faint_bg_color (0.06) pro dvojitý tonální posun"
  - "Dark mode beze změn: obě větve explicitně vracejí legacy barvy bez doteku dark path"
  - "Kontrast zachován: žlutá a cyan >=2.2, fg/bg >=4.5 v light mode po tonálním posunu"

patterns-established:
  - "Variant-aware resolver pattern: všechny light-mode barvy se odvozují z ui.visuals(), nikoliv z pevných RGB"
  - "TDD terminálních barev: testy assertují luminanci, kontrast ratio a HashSet odlišnost variant"

requirements-completed: [LITE-01, LITE-02, LITE-03, LITE-04]

# Metrics
duration: 15min
completed: 2026-03-05
---

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
