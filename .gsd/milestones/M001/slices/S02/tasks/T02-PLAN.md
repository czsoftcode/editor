# T02: 01-zaklad 02

**Slice:** S02 — **Milestone:** M001

## Description

Highlighter parametrizace a startup theme apply. Rozšíření Highlighter o theme_name parametr (EDIT-01, EDIT-02, EDIT-03), cache invalidaci pouze při změně tématu (EDIT-04), a aplikaci tématu v EditorApp::new() (THEME-03).

Purpose: Dokončit Phase 1 — žádný startup flash, funkční light/dark syntax highlighting.
Output: `src/highlighter.rs` s parametrickým themingem, `src/app/mod.rs` s startup apply.

## Must-Haves

- [ ] "Highlighter::highlight() přijímá theme_name jako parametr — žádný hardcoded \"base16-ocean.dark\""
- [ ] "Highlighter používá \"Solarized (light)\" pro light mode a \"base16-ocean.dark\" pro dark mode"
- [ ] "Highlighter cache se invaliduje pouze při skutečné změně tématu (EDIT-04), ne každý frame"
- [ ] "Téma je aplikováno v EditorApp::new() — první frame je v správném tématu bez flash (THEME-03)"

## Files

- `src/highlighter.rs`
- `src/app/mod.rs`
