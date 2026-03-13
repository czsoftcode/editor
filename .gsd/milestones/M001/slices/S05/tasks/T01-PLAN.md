# T01: 03-light-varianty-settings-ui 01

**Slice:** S05 — **Milestone:** M001

## Description

Dodat konecne mapovani tri light variant do `Settings::to_egui_visuals()` tak, aby kazda varianta mela vlastni citelnou paletu a splnila LITE-01..LITE-04.

Purpose: Uzavrit modelovou cast Phase 03 pred UI integraci; vsechny vizualni konstanty drzet centralne v `settings.rs`.
Output: `src/settings.rs` s explicitnim mapovanim variant a unit testy overujicimi barvy + oddeleni panelu.

## Must-Haves

- [ ] "`Settings::to_egui_visuals()` mapuje vsechny tri light varianty na explicitni palety: WarmIvory (255,252,240), CoolGray (242,242,242), Sepia (240,230,210)."
- [ ] "Kazda light varianta nastavuje `panel_fill`, `window_fill` i `faint_bg_color`; `faint_bg_color` neni shodny s `panel_fill` a panely se vizualne oddeli."
- [ ] "Dark branch zustava funkcne kompatibilni (`Visuals::dark()`), bez regresi do jiz hotovych THEME/TERM/TREE casti."
- [ ] "Mapovani variant je testovatelne unit testy v `src/settings.rs` a kryje LITE-01..LITE-04 bez UI zavislosti."

## Files

- `src/settings.rs`
