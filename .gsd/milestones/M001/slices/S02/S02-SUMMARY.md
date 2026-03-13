---
id: S02
parent: M001
milestone: M001
provides:
  - LightVariant enum (WarmIvory, CoolGray, Sepia) v Settings
  - syntect_theme_name() pro dark/light téma
  - to_egui_visuals() s delegací v apply()
  - Settings startup apply flow
key_files:
  - src/settings.rs
key_decisions:
  - "LightVariant enum with 3 variants for Phase 3"
  - "syntect_theme_name returns base16-ocean.dark for dark, Solarized (light) for light"
  - "apply() delegates to to_egui_visuals()"
patterns_established:
  - "Centralizovaná theme logika v Settings — if/else eliminován z apply()"
observability_surfaces:
  - none
drill_down_paths:
  - tasks/T01-SUMMARY.md
  - tasks/T02-SUMMARY.md
  - tasks/T03-SUMMARY.md
  - tasks/T04-SUMMARY.md
duration: 13min
verification_result: passed
completed_at: 2026-03-04
---

# S02: Zaklad

**Datový model LightVariant enum, centralizovaná theme logika v Settings a unit testy pokrývající THEME-01/02/04, SETT-04.**

## What Happened

Rozšířen Settings struct o LightVariant enum se třemi variantami (WarmIvory, CoolGray, Sepia). Přidány metody `syntect_theme_name()` a `to_egui_visuals()`, přičemž `apply()` deleguje na `to_egui_visuals()`. Highlighter integrace přes `set_theme()` na místech settings apply. Základní unit testy ověřují korektní mapování.

## Verification

- `cargo check` čistý
- Unit testy pro theme mapování, syntect name, visuals — vše zelené
- Highlighter integration ověřena v apply flow pro root i deferred viewport

## Deviations

Tasky T01–T02 byly již implementované v aktuálním HEAD; verifikační task commity vytvořeny atomicky.

## Known Limitations

- Dark branch zůstává Visuals::dark() beze změny — záměrné pro kompatibilitu

## Follow-ups

- Phase 3 light varianty (mapování do konkrétních palet) — řešeno v S05

## Files Created/Modified

- `src/settings.rs` — LightVariant enum, syntect_theme_name(), to_egui_visuals(), apply() delegace
- `src/highlighter.rs` — set_theme() integrace

## Forward Intelligence

### What the next slice should know
- Theme logika je centralizovaná — všechny barvy jdou přes to_egui_visuals()

### What's fragile
- Dark mode path nesmí být modifikován — jakákoliv změna vizuálů riskuje regrese

### Authoritative diagnostics
- Unit testy v settings.rs — pokrývají theme varianty a syntect mapování

### What assumptions changed
- Původně se předpokládal if/else v apply() — nahrazeno delegací na to_egui_visuals()
