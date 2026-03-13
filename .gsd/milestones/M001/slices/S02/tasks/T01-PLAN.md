# T01: 01-zaklad 01

**Slice:** S02 — **Milestone:** M001

## Description

Rozšíření Settings struct o LightVariant enum a theme-aware metody. Základ pro celý dark/light milestone — bez těchto změn nemohou fungovat ani Highlighter, ani startup apply, ani Phase 3 light varianty.

Purpose: Centralizovat theme logiku do Settings, eliminovat přímý if/else v apply(), připravit datový model pro Phase 3 light varianty.
Output: `src/settings.rs` s LightVariant enum, `syntect_theme_name()`, `to_egui_visuals()`, upravenou `apply()` a unit testy pokrývajícími THEME-01, THEME-02, THEME-04, SETT-04.

## Must-Haves

- [ ] "Settings struct obsahuje `light_variant: LightVariant` pole s `#[serde(default)]` — staré TOML bez tohoto pole se načte bez paniku"
- [ ] "`syntect_theme_name()` vrátí `\"base16-ocean.dark\"` pro dark mode a `\"Solarized (light)\"` pro light mode"
- [ ] "`to_egui_visuals()` vrátí `Visuals` s `dark_mode: true` pro dark a `dark_mode: false` pro light"
- [ ] "`Settings::apply()` volá `ctx.set_visuals(self.to_egui_visuals())` místo přímého if/else"

## Files

- `src/settings.rs`
