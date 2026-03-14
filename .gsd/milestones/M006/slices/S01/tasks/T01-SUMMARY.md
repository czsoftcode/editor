---
id: T01
parent: S01
milestone: M006
provides:
  - ProjectSearch.show_panel + last_selected_index state fields
  - render_search_panel() inline panel s query inputem, togglery, a výsledky
  - Layout integrace panelu PŘED CentralPanel v render_workspace()
  - Keymap dispatch otvírá panel při Ctrl+Shift+F
key_files:
  - src/app/ui/workspace/state/types.rs
  - src/app/ui/search_picker.rs
  - src/app/ui/workspace/mod.rs
  - src/app/ui/workspace/menubar/mod.rs
key_decisions:
  - Panel používá last_selected_index + .take() pattern místo přímého returnu Option pro navigaci — kvůli borrow checker limitacím (panel drží &mut ws během renderování výsledků)
  - I18n klíče sdíleny s existujícím modálem (project-search-hint, project-search-regex-toggle atd.) — žádné nové lokalizační klíče
  - show_input ponechán v dispatch — T03 ho odstraní po přepojení keymapu
patterns_established:
  - TopBottomPanel::bottom("search_panel") před CentralPanel v layout pořadí
  - Kliknuté výsledky z panelu se zpracovávají po render_search_panel() voláním přes last_selected_index.take()
observability_surfaces:
  - ProjectSearch.show_panel — runtime viditelnost panelu
  - ProjectSearch.last_selected_index — transitní signál kliknutého výsledku (resetuje se na None po zpracování)
  - ProjectSearch.regex_error — inline chybová hláška v panelu
  - ProjectSearch.searching — spinner indikátor v panelu
duration: 15min
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: State rozšíření + panel skeleton + layout integrace

**Inline search panel s plným query UI, togglery, výsledky s navigací, a layout integrací před CentralPanel.**

## What Happened

1. Rozšířen `ProjectSearch` struct o `show_panel: bool` (default false) a `last_selected_index: Option<usize>` (default None). Pole `show_input` ponecháno — T03 ho odstraní.

2. Vytvořena `render_search_panel()` v `search_picker.rs` (~230 řádků):
   - Early return na `!show_panel`
   - `TopBottomPanel::bottom("search_panel")` — resizable, default 250px, min 100, max 60% výšky
   - Horní řádek: query TextEdit + togglery (regex `.*`, case `Aa`, word `W`, replace `↔`) + close button `✕`
   - Podmíněný replace input
   - File filter input
   - Inline regex error (červeně)
   - Automatický search při změně togglerů
   - Scrollovatelný seznam výsledků seskupených per-soubor s match highlighting a kontextem
   - Klikatelné výsledky → nastaví `last_selected_index`

3. V `workspace/mod.rs` vloženo volání `render_search_panel()` v sekci 4b PŘED CentralPanel. Po renderování se zpracuje `last_selected_index` → `open_file_in_ws()` + `jump_to_location()`.

4. V `menubar/mod.rs` dispatch pro `project_search` přidáno `show_panel = true` (vedle existujícího `show_input = true`).

## Verification

- `cargo check` — čistá kompilace, žádné chyby ani warningy
- `cargo test` — 192+ testů pass, 0 failures
- Všech 20 search_picker::tests pass (build_regex 10× + search_file 3× + file_filter 2× + replace 5×)
- `grep -c 'show_panel' src/app/ui/workspace/state/types.rs` → 2 ✅
- `grep -c 'search_panel' src/app/ui/workspace/mod.rs` → 2 ✅
- `grep -c 'render_search_panel' src/app/ui/search_picker.rs` → 1 ✅

### Slice-level verification (partial — T01 intermediate):
- ✅ `cargo check` čistá
- ⬜ `./check.sh` — neběželo (nepovinné pro intermediate task)
- ✅ `show_panel` v types.rs ≥1
- ✅ `search_panel` v mod.rs ≥1
- ⬜ `render_project_search_dialog` v mod.rs → 0 — zatím 2 (modální dialog smazán/podmíněn v T03)
- ✅ 20 unit testů pass

## Diagnostics

- Panel viditelnost: `ws.project_search.show_panel` — `true` = zobrazen, `false` = skryt
- Search stav: `ws.project_search.searching` — `true` = spinner v panelu
- Regex chyba: `ws.project_search.regex_error` — `Some("...")` = červená hláška v panelu
- Kliknutý výsledek: `ws.project_search.last_selected_index` — `Some(idx)` transitně, po zpracování `None`

## Deviations

- Task plan psal `t: &Translator` ale kódová báze používá `i18n: &I18n` — použit reálný typ.
- Panel renderuje i výsledky přímo (ne jen skeleton) — protože engine funkce (`start_project_search`, polling přes `rx`) jsou sdílené a výsledky se přirozeně akumulují v `ws.project_search.results`. Oddělený rendering výsledků by byl zbytečný krok navíc.

## Known Issues

- Modální dialog (`render_project_search_dialog`) stále aktivní paralelně s panelem — T03 ho podmíní/smaže po přepojení keymapu.
- Panel polling výsledků z `rx` kanálu probíhá v `poll_and_render_project_search_results()` (modální), ne v panelu — panel zobrazuje výsledky co se do `results` vektoru dostanou tímto polováním. Pokud se modální dialog neotevře, polling stále probíhá v `poll_and_render_project_search_results()`. Toto je akceptovatelné pro T01 a bude refaktorováno.

## Files Created/Modified

- `src/app/ui/workspace/state/types.rs` — přidáno `show_panel: bool` a `last_selected_index: Option<usize>` do ProjectSearch
- `src/app/ui/search_picker.rs` — nová `render_search_panel()` (~230 řádků) s plným query UI a výsledky
- `src/app/ui/workspace/mod.rs` — import render_search_panel, volání v layout pořadí, zpracování kliknutého výsledku
- `src/app/ui/workspace/menubar/mod.rs` — dispatch přidává `show_panel = true`
- `.gsd/milestones/M006/slices/S01/tasks/T01-PLAN.md` — přidána Observability Impact sekce
