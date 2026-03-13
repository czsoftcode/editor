---
id: S04
parent: M001
milestone: M001
provides:
  - Theme-aware terminal rendering (background, scrollbar, cursor) pro light mode
  - terminal_theme_for_visuals() resolver bez restartu PTY backendu
  - set_theme() per-frame v Terminal::ui()
  - Theme-aware status text (primary/secondary) z ui.visuals()
  - Git porcelain status → GitVisualStatus mapování s light paletou
key_files:
  - src/app/ui/terminal/instance/theme.rs
  - src/app/ui/git_status.rs
key_decisions:
  - "Použit ctx.style().visuals.panel_fill jako jednotný theme-aware fill"
  - "Fix centralizovaný v StandardTerminalWindow — automatická propagace do AI i build"
  - "Theme se aplikuje při každém renderu přes set_theme() bez restartu backendu"
  - "Git light mode používá explicitní paletu pro M/A/D/??"
patterns_established:
  - "Terminal theme resolver napojený na activní visuals — žádné hardcoded barvy"
  - "Scrollbar track/thumb odvozené z ui.visuals() mixem"
observability_surfaces:
  - none
drill_down_paths:
  - tasks/T01-SUMMARY.md
  - tasks/T02-SUMMARY.md
duration: 15min
verification_result: passed
completed_at: 2026-03-04
---

# S04: Terminal Git Barvy

**Plně theme-aware terminálový rendering — background, scrollbar, kurzor a git barvy reagují na light/dark mode za běhu bez restartu backendu.**

## What Happened

Vytvořen theme resolver `terminal_theme_for_visuals()` v theme.rs, který odvozuje celou ColorPalette z aktivních egui visuals. Scrollbar track a thumb jsou mixovány z panel_fill. Status text (primary/secondary) sleduje ui.visuals() místo fixních RGB. Git porcelain status mapován na GitVisualStatus s explicitní light paletou. Všechny změny centralizovány v StandardTerminalWindow pro automatickou propagaci do AI i build terminálu.

## Verification

- `cargo check` čistý
- Theme přepínání za běhu ověřeno — žádný restart backendu
- Scrollbar a cursor barvy korektní v obou módech

## Deviations

Žádné.

## Known Limitations

- Unfocused overlay/cursor barvy napojeny na visuals — při rychlém přepínání může být krátký vizuální glitch

## Follow-ups

- Variant-aware tonální adaptace terminálové palety — řešeno v S05/T04

## Files Created/Modified

- `src/app/ui/terminal/instance/theme.rs` — theme resolver, ColorPalette blending, scrollbar helpery
- `src/app/ui/git_status.rs` — GitVisualStatus mapování, light paleta
- `src/app/ui/terminal/instance/mod.rs` — set_theme() per-frame volání

## Forward Intelligence

### What the next slice should know
- Terminal theme je plně napojený na visuals — stačí měnit to_egui_visuals() pro nové varianty

### What's fragile
- panel_fill blending ratia (0.06–0.42) — příliš vysoké ratio sníží čitelnost ANSI barev

### Authoritative diagnostics
- theme.rs unit testy — luminance, kontrast, variant distinctness

### What assumptions changed
- Původně se počítalo s restartem PTY — zbytečné, set_theme() per-frame stačí
