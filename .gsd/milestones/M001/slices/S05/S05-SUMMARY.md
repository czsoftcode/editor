---
id: S05
parent: M001
milestone: M001
provides:
  - Explicitní mapování 3 light variant do to_egui_visuals() s vlastní paletou
  - Light variant picker UI (clickable cards se swatch + label)
  - Live preview při změně theme s fingerprint-based invalidací
  - Settings persistence (TOML canonical, JSON legacy migrace)
  - Save/Cancel/Discard flow se snapshot sémantikou
  - Variant-aware terminal ColorPalette blending (0.55 ratio)
  - Variant-aware git status barvy s tonálním posunem
  - warm_ivory_bg() helper pro detekci teplého panel_fill
key_files:
  - src/settings.rs
  - src/app/ui/workspace/modal_dialogs/settings.rs
  - src/app/ui/terminal/instance/theme.rs
  - src/app/ui/git_status.rs
key_decisions:
  - "Light varianty mapovány explicitním match self.light_variant v to_egui_visuals()"
  - "Light variant picker jako clickable cards, hidden v dark mode"
  - "Live preview jen na changed() events + fingerprint check"
  - "Save persistuje jen při reálné změně fingerprinu"
  - "Cancel restoruje ze snapshot + bumpne version jen pokud se fingerprint liší"
  - "Canonical storage settings.toml; settings.json legacy, smaže se po migraci"
  - "Background blend ratio 0.55 pro silnější variantní tón"
patterns_established:
  - "Theme fingerprint pro detekci reálné změny"
  - "Snapshot-based cancel/discard flow"
  - "TempConfigDir pro izolované persistence testy"
observability_surfaces:
  - none
drill_down_paths:
  - tasks/T01-SUMMARY.md
  - tasks/T02-SUMMARY.md
  - tasks/T03-SUMMARY.md
  - tasks/T04-SUMMARY.md
  - tasks/T05-SUMMARY.md
duration: 30min
verification_result: passed
completed_at: 2026-03-05
---

# S05: Light Varianty Settings Ui

**Tři light varianty (WarmIvory, CoolGray, Sepia) s vlastní paletou, picker UI, live preview, TOML persistence a variant-aware terminal/git barvami.**

## What Happened

Pět tasků pokrylo kompletní lifecycle light variant: T01 dodal explicitní mapování variant do to_egui_visuals() s unit testy barev a odlišení panelů. T02 přidal picker UI jako clickable karty se swatch a label, skrytý v dark mode. T03 zavedl persistence do settings.toml (s legacy JSON migrací), snapshot-based Save/Cancel flow a theme fingerprint pro detekci změn — 15/15 testů. T04 přidal variant-aware terminálovou paletu (panel_fill blending) a git status barvy s tonálním posunem — 23 regresních testů. T05 opravil dva UAT defekty: picker karty se zobrazovaly jen jedna místo tří (with_layout bug) a WarmIvory terminál měl studený tón místo teplého (warm_ivory_bg() helper, blend ratio 0.55).

## Verification

- `cargo check` čistý
- 55+ unit testů (varianty, persistence, terminal theme, git status)
- TDD cyklus RED→GREEN pro terminal/git barvy
- UAT gap closure pro picker a WarmIvory tón

## Deviations

Žádné — všech 5 tasků provedeno dle plánu.

## Known Limitations

- Warning text kontrast v light mode Settings modal (známý tech debt)
- Sepia warm_ivory_bg() detekce je na hraně (r-b <= 12, threshold 10)

## Follow-ups

- Manuální GUI smoke test doporučen při UAT

## Files Created/Modified

- `src/settings.rs` — TOML persistence, load/save, migrace, konstanty, testy
- `src/app/ui/workspace/modal_dialogs/settings.rs` — variant picker, fingerprint, snapshot, Save/Cancel
- `src/app/ui/workspace/modal_dialogs.rs` — global discard flow
- `src/app/ui/workspace/state/mod.rs` — settings_original snapshot field
- `src/app/ui/terminal/instance/theme.rs` — tone_light_palette(), warm_ivory_bg()
- `src/app/ui/git_status.rs` — git_color_for_visuals() s tonálním posunem
- `src/app/ui/file_tree/render.rs` — resolve_file_tree_git_color()

## Forward Intelligence

### What the next slice should know
- Theme lifecycle je kompletní: model → UI → persistence → terminal → git

### What's fragile
- warm_ivory_bg() threshold 10 (r-b) — Sepia je na r-b=32, ale budoucí varianty mohou být blíž

### Authoritative diagnostics
- settings.rs persistence testy — round-trip, migrace, fingerprint
- theme.rs testy — luminance, kontrast, distinctness

### What assumptions changed
- Původně se počítalo s jednoduchým radio buttonem — clickable karty jsou UX výrazně lepší
