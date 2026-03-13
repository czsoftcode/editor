# S01: Editovatelný panel se syntax highlighting, diff a sync scrollem

**Goal:** Levý panel history view je editovatelný TextEdit se syntax highlighting a diff zvýrazněním. Pravý panel je read-only se syntax highlighting a diff barvami. Scroll je synchronizovaný. Editace se propsávají do tab bufferu. Výchozí stav panelů odpovídá počtu verzí. Diff cache se invaliduje při editaci.

**Demo:** Otevřít historii souboru → levý panel je editovatelný s barevnou syntaxí a diff pozadím → pravý panel je read-only se syntaxí a diff barvami → scroll jedním panelem pohne druhým → editace v levém panelu označí tab jako modified (●) → soubor s 1 verzí má prázdný pravý panel.

## Must-Haves

- Levý panel je `TextEdit::multiline` s layouter callbackem (syntax + diff background) — R001, R002, R009
- Pravý panel je `Label` s `LayoutJob` (syntax + diff background) — R002, R009
- Diff mapy (per-řádek ChangeTag) se počítají z `current_content` vs historická verze a invalidují při editaci — R009
- Proportionální scroll sync mezi panely — R003
- Editace v levém panelu průběžně aktualizuje `tab.content` a nastaví `tab.modified = true` — R006
- Výchozí `selected_index`: `None` pro 1 verzi, `Some(0)` pro >1 verzi — R007
- `render_history_split_view()` přijímá `&Highlighter`, `theme_name`, `ext`, `fname` — boundary contract pro S02
- Existující testy pro diff logiku zachovány a projdou — regression
- `cargo check` + `./check.sh` prochází

## Proof Level

- This slice proves: integration (TextEdit+layouter+diff+highlight v jednom widgetu, dosud nekombinováno)
- Real runtime required: yes (vizuální ověření diff+syntax kombinace v běžícím editoru)
- Human/UAT required: yes (čitelnost syntax barev, diff pozadí kontrast, scroll sync UX)

## Verification

- `cargo check` — kompilace bez chyb
- `./check.sh` — clippy + testy (včetně existujících diff testů)
- `cargo test -p polycredo_editor -- history` — unit testy pro diff→panel logiku a apply_diff_backgrounds
- UAT: vizuální kontrola v běžícím editoru (syntax barvy + diff pozadí + scroll sync + editovatelnost)

## Observability / Diagnostics

- Runtime signals: `tab.modified` flag se nastaví při editaci v history view; diff cache invalidace přes `content_hash` změnu
- Inspection surfaces: `HistoryViewState` fieldy (`content_hash`, `scroll_source`, `left_diff_map.len()`, `right_diff_map.len()`) inspektovatelné přes debugger
- Failure visibility: pokud diff recompute selže (prázdné mapy), oba panely se zobrazí bez diff pozadí (graceful degradation); pokud syntax highlighting selže, text se zobrazí monochrome (existující highlighter fallback)
- Redaction constraints: none

## Integration Closure

- Upstream surfaces consumed: `Highlighter::highlight()` z `src/highlighter.rs`, `LocalHistory::get_snapshot_content()` z `src/app/local_history.rs`, `Tab` struct z `src/app/ui/editor/mod.rs`
- New wiring introduced: `render_history_split_view()` rozšířená signatura (highlighter, theme, ext, fname) + nový return type `HistorySplitResult`; borrow-checker řešení v `workspace/mod.rs` (extrakce highlighter ref + tab metadata před mutable borrow)
- What remains before milestone is truly usable end-to-end: S02 (Obnovit tlačítko + potvrzovací dialog + i18n)

## Tasks

- [x] **T01: Datový model, diff→panel logika a helper funkce** `est:1h`
  - Why: Stavební bloky pro rendering — rozšíření HistoryViewState, funkce pro sestavení panel textů + diff map z diff výstupu, funkce pro aplikaci diff background na LayoutJob sections. Bez nich nelze renderovat.
  - Files: `src/app/ui/workspace/history/mod.rs`
  - Do: (1) Přidat `ScrollSource` enum a nové fieldy do `HistoryViewState` (content_hash, left/right_scroll_y, scroll_source, right_panel_text, left/right_diff_map). (2) Napsat `build_panel_texts()` — z `Vec<DiffLine>` sestaví left text (Equal+Insert), right text (Equal+Delete), left_diff_map a right_diff_map. (3) Napsat `apply_diff_backgrounds()` — projde LayoutJob sections, pro každou section zjistí řádek z byte offset, nastaví `TextFormat.background` podle diff mapy. (4) Napsat `compute_line_offsets()` helper pro mapování byte offset → řádek. (5) Přidat `HistorySplitResult` struct (close: bool, content_changed: bool). (6) Rozšířit inicializaci HistoryViewState v build_panel_texts s výchozím stavem. (7) Přidat unit testy pro build_panel_texts a apply_diff_backgrounds.
  - Verify: `cargo test -p polycredo_editor -- history` — nové testy projdou, existující testy nezlomeny
  - Done when: Všechny nové funkce existují, jsou unit-testované, `cargo check` prochází

- [x] **T02: Přepis renderingu, scroll sync a napojení na volajícího** `est:1.5h`
  - Why: Složení stavebních bloků z T01 do funkčního UI — přepis `render_history_split_view()` na TextEdit+layouter (levý) a Label+LayoutJob se syntax (pravý), scroll sync, tab sync, a úprava volajícího v workspace/mod.rs pro novou signaturu.
  - Files: `src/app/ui/workspace/history/mod.rs`, `src/app/ui/workspace/mod.rs`
  - Do: (1) Přepsat levý panel z `Label`+`LayoutJob` na `TextEdit::multiline` s layouter callbackem — layouter volá `highlighter.highlight()`, klonuje job, aplikuje `apply_diff_backgrounds()`. (2) Přepsat pravý panel z monochrome `LayoutJob` na `Label` s `LayoutJob` kde syntax highlighting + diff background jsou složeny dohromady. (3) Implementovat scroll sync — po renderování obou panelů detekovat kdo scrolloval (porovnání offset s uloženým), proportionálně přepočítat offset druhého panelu, epsilon tolerance proti feedback loop. (4) Implementovat tab sync — po `TextEdit::changed()` propsát `current_content` do tab.content, nastavit modified+last_edit+save_status, invalidovat diff cache přes content_hash. (5) Upravit `render_history_split_view()` signaturu — přidat `&Highlighter`, `theme_name`, `ext`, `fname`, vrátit `HistorySplitResult`. (6) V `workspace/mod.rs` extrahovat highlighter ref, theme_name, ext, fname z `ws.editor` před mutable borrow na `ws.history_view`, předat do render funkce, zpracovat `HistorySplitResult`. (7) Upravit inicializaci `HistoryViewState` — `selected_index: if entries.len() > 1 { Some(0) } else { None }`, inicializovat nové fieldy. (8) Invalidovat diff cache nejen při změně selected_index ale i při změně content_hash.
  - Verify: `cargo check` + `./check.sh` prochází; vizuální kontrola v běžícím editoru (levý panel editovatelný se syntax + diff, pravý read-only se syntax + diff, scroll sync funguje, editace → tab modified)
  - Done when: Oba panely mají syntax highlighting + diff zvýraznění, levý je editovatelný, scroll je synchronizovaný, editace se propsávají do tabu, výchozí stav odpovídá počtu verzí, cargo check + ./check.sh prochází

## Files Likely Touched

- `src/app/ui/workspace/history/mod.rs` — hlavní přepis (rozšíření stavu, nové funkce, nový rendering)
- `src/app/ui/workspace/mod.rs` — rozšíření parametrů při volání render funkce, borrow-checker řešení, inicializace nových fieldů
