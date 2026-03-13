# S02: Zaklad

**Goal:** Rozšíření Settings struct o LightVariant enum a theme-aware metody.
**Demo:** Rozšíření Settings struct o LightVariant enum a theme-aware metody.

## Must-Haves


## Tasks

- [x] **T01: 01-zaklad 01** `est:5min`
  - Rozšíření Settings struct o LightVariant enum a theme-aware metody. Základ pro celý dark/light milestone — bez těchto změn nemohou fungovat ani Highlighter, ani startup apply, ani Phase 3 light varianty.

Purpose: Centralizovat theme logiku do Settings, eliminovat přímý if/else v apply(), připravit datový model pro Phase 3 light varianty.
Output: `src/settings.rs` s LightVariant enum, `syntect_theme_name()`, `to_egui_visuals()`, upravenou `apply()` a unit testy pokrývajícími THEME-01, THEME-02, THEME-04, SETT-04.
- [x] **T02: 01-zaklad 02** `est:3min`
  - Highlighter parametrizace a startup theme apply. Rozšíření Highlighter o theme_name parametr (EDIT-01, EDIT-02, EDIT-03), cache invalidaci pouze při změně tématu (EDIT-04), a aplikaci tématu v EditorApp::new() (THEME-03).

Purpose: Dokončit Phase 1 — žádný startup flash, funkční light/dark syntax highlighting.
Output: `src/highlighter.rs` s parametrickým themingem, `src/app/mod.rs` s startup apply.
- [x] **T03: 01-zaklad 03** `est:1 min`
  - Uzavřít UAT gap #2: floating terminal rám zůstává čistě černý v light mode.

Purpose: Odstranit hardcoded tmavý frame u standardního terminálového okna a navázat barvu rámu na aktivní téma.
Output: `src/app/ui/terminal/window.rs` s theme-aware frame fill, který funguje pro oba floating terminály.
- [x] **T04: 01-zaklad 04** `est:4 min`
  - Uzavřít UAT gap #6: status bar text je v light mode příliš světlý a špatně čitelný.

Purpose: Nahradit fixní status bar text paletu za theme-aware barvy, aby byl kontrast stabilní v dark i light režimu.
Output: `src/app/ui/editor/ui.rs` s theme-aware status bar barvami bez hardcoded světle-modrých konstant.

## Files Likely Touched

- `src/settings.rs`
- `src/highlighter.rs`
- `src/app/mod.rs`
- `src/app/ui/terminal/window.rs`
- `src/app/ui/editor/ui.rs`
