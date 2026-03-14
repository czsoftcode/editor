# S01: Inline project search panel

**Goal:** Přesunout project search z modálních dialogů do inline `TopBottomPanel::bottom` pod editorem. Panel zobrazuje query input s togglery, per-file výsledky se zvýrazněním, klik na výsledek otevírá soubor s fokusem. Replace flow se spouští z panelu. Stav persistuje přes close/reopen. Panel je resizable.
**Demo:** Ctrl+Shift+F → panel se otevře pod editorem → query → výsledky streamují do panelu per-file se zvýrazněním → klik na výsledek → editor jumpne na řádek s fokusem → panel zůstane otevřený → Escape → Ctrl+Shift+F → výsledky stále zachovány. Replace: toggle → replace text → Replace All → preview dialog → potvrzení → soubory modifikovány.

## Must-Haves

- `TopBottomPanel::bottom("search_panel")` vložen PŘED `CentralPanel` v `render_workspace()` layout pořadí
- Query input s regex/case/whole-word/file filter togglery v panelu nahoře
- Per-file seskupení výsledků s filename hlavičkou a match highlighting přes LayoutJob
- Kontextové řádky s dim barvou
- Klik na výsledek → `open_and_jump()` + `request_editor_focus()` + `FocusedPanel::Editor`
- Poll loop pro inkrementální streamování výsledků (try_recv + request_repaint)
- Spinner + "Hledám..." indikátor během search
- Replace toggle + replace input + Replace All button → spuštění existujícího preview dialogu
- `show_panel` jako toggle (ne show_input) — close nezmaže query ani results
- Panel resize přes `resizable(true)` s `default_height(250.0)`
- Keymap dispatch: Ctrl+Shift+F → toggle show_panel
- Menubar: přesměrování project_search action na show_panel toggle
- Smazání/podmínění mrtvých modálních dialogů (render_project_search_dialog, poll_and_render_project_search_results)
- i18n klíče pro nové UI prvky ve všech 5 jazycích

## Proof Level

- This slice proves: integration — celý project search workflow funguje v inline panelu end-to-end
- Real runtime required: yes (desktop GUI)
- Human/UAT required: yes (vizuální ověření panelu, fokus, resize)

## Verification

- `cargo check` — čistá kompilace bez warningů
- `./check.sh` — fmt, clippy, všechny testy pass
- `grep -c 'show_panel' src/app/ui/workspace/state/types.rs` → ≥1
- `grep -c 'search_panel' src/app/ui/workspace/mod.rs` → ≥1
- `grep -c 'render_project_search_dialog' src/app/ui/workspace/mod.rs` → 0 (modální dialog smazán/podmíněn)
- Existující 20 unit testů v search_picker::tests stále pass (engine funkce nezměněny)

## Observability / Diagnostics

- Runtime signals: `ProjectSearch.show_panel: bool` — viditelnost panelu, `ProjectSearch.searching: bool` — indikátor běhu
- Inspection surfaces: panel se zobrazí/skryje přes Ctrl+Shift+F toggle, výsledky v UI
- Failure visibility: `ProjectSearch.regex_error` — inline chyba v panelu, `SearchBatch::Error` → toast
- Redaction constraints: none

## Tasks

- [x] **T01: State rozšíření + panel skeleton + layout integrace** `est:45m`
  - Why: Dokáže že TopBottomPanel se zobrazí na správném místě v layout pořadí, query input funguje, search se spouští. Bez tohoto nelze ověřit layout risk.
  - Files: `src/app/ui/workspace/state/types.rs`, `src/app/ui/search_picker.rs`, `src/app/ui/workspace/mod.rs`
  - Do: 1) Rozšířit `ProjectSearch` o `show_panel: bool` (default false), `panel_height: f32` (default 250.0), `last_selected_index: Option<usize>`. 2) Vytvořit `render_search_panel()` funkci v search_picker.rs — `TopBottomPanel::bottom("search_panel").resizable(true).default_height(250.0).min_height(100.0).max_height(ctx.screen_rect().height() * 0.6)` s query input, togglery (regex/case/whole-word/file filter), a Search/Close tlačítky. 3) V `render_workspace()` vložit volání `render_search_panel()` PŘED `CentralPanel::default().show()` (podmíněné na `ws.project_search.show_panel`). 4) Zapojit search spouštění — Enter v query inputu nebo Search button spustí `run_project_search()` ve stejném threadu patternu jako stávající dialog. 5) Přidat inline regex error (červeně pod inputem) — přenést z render_project_search_dialog(). 6) Replace toggle button (↔) + replace input v panelu — zatím jen UI, Replace All button bude v T02.
  - Verify: `cargo check` čistá kompilace. `grep 'search_panel' src/app/ui/workspace/mod.rs` najde panel rendering.
  - Done when: Panel se zobrazí pod editorem při `show_panel = true`, query input funguje, search se spouští a výsledky začínají streamovat do `ProjectSearch.results`.

- [x] **T02: Výsledky + klik → jump + poll loop + replace flow** `est:60m`
  - Why: Hlavní funkční jádro — výsledky se zobrazují v panelu se zvýrazněním, klik otevře soubor, replace flow funguje. Toto je nejvíce kódu — rendering výsledků, poll loop, click handler, replace spuštění.
  - Files: `src/app/ui/search_picker.rs`, `src/app/ui/workspace/mod.rs`
  - Do: 1) V `render_search_panel()` přidat ScrollArea pod inputem pro výsledky. 2) Per-file seskupení: iterovat `ws.project_search.results` seskupené dle souboru, filename hlavička (collapsible CollapsingHeader nebo bold label). 3) Pro každý SearchResult renderovat match řádky přes `build_match_layout_job()` a kontextové řádky přes `build_context_layout_job()` — přesně stejné funkce jako v modálním dialogu. 4) Klik na výsledek: `Label::new(job).sense(Sense::click())`, při kliknutí zavolat `open_and_jump(ws, &result.path, result.line)` + `ws.focused_panel = FocusedPanel::Editor` + `ws.project_search.last_selected_index = Some(idx)`. 5) Poll loop: v render_search_panel() přidat try_recv() loop pro SearchBatch akumulaci — stejný pattern jako poll_and_render_project_search_results(). Spinner + "Hledám..." indikátor přes `searching` flag + `ctx.request_repaint()`. 6) Replace All button: při kliknutí spustit `compute_replace_previews()` a nastavit `ws.project_search.show_replace_preview = true` — existující `render_replace_preview_dialog()` se vykreslí normálně (je modální, nezávisí na search UI). 7) Separátor `···` mezi nesouvisejícími kontextovými bloky.
  - Verify: `cargo check`. Existujících 20 unit testů v search_picker::tests stále pass.
  - Done when: Výsledky se zobrazují v panelu per-file se zvýrazněním, klik na výsledek otevře soubor s fokusem, replace flow spustí preview dialog z panelu.

- [x] **T03: Keymap dispatch + smazání mrtvých modálů + i18n + cleanup** `est:45m`
  - Why: Uzavře slice — keymap toggle, menu přesměrování, smazání starých modálních dialogů, i18n, final quality gate.
  - Files: `src/app/keymap.rs`, `src/app/ui/workspace/menubar/mod.rs`, `src/app/ui/workspace/mod.rs`, `src/app/ui/search_picker.rs`, `locales/{cs,en,sk,de,ru}/ui.ftl`
  - Do: 1) V keymap dispatch: `CommandId::ProjectSearch` → `ws.project_search.show_panel = !ws.project_search.show_panel; if ws.project_search.show_panel { ws.project_search.focus_requested = true; }` místo `show_input = true`. 2) V menubar `process_menu_actions()`: přesměrovat `actions.project_search` na `show_panel` toggle. 3) Smazat volání `render_project_search_dialog()` a `poll_and_render_project_search_results()` z `render_workspace()` — celý modální flow nahrazen inline panelem. Ponechat `render_replace_preview_dialog()`. 4) Smazat mrtvé funkce `render_project_search_dialog()` a `poll_and_render_project_search_results()` ze search_picker.rs (nebo podmínit `#[cfg(never)]` pokud by smazání rozbilo jiné závislosti). 5) Smazat `show_input` field z ProjectSearch pokud už není používán. 6) i18n: přidat nové klíče do všech 5 jazyků — `project-search-panel-title`, `project-search-no-results`, `project-search-results-count`. Většina textů sdílena s existujícími klíči. 7) Escape handling v panelu: Escape zavře panel (show_panel = false), ale nekonsumuje Escape pokud je otevřený modální dialog (settings, replace preview). 8) Final cleanup: `cargo fmt`, `cargo clippy`, odstranit unused imports.
  - Verify: `cargo check` + `./check.sh` — všechny testy pass, clippy čistý. `grep -c 'show_input' src/app/ui/workspace/state/types.rs` → 0 (field smazán). `grep -c 'render_project_search_dialog' src/app/ui/workspace/mod.rs` → 0.
  - Done when: Ctrl+Shift+F toggles panel. Menu Project Search otevírá panel. Mrtvé modální dialogy smazány. i18n kompletní. `./check.sh` projde.

## Files Likely Touched

- `src/app/ui/workspace/state/types.rs` — rozšíření ProjectSearch structu
- `src/app/ui/search_picker.rs` — nový render_search_panel(), smazání modálních dialogů
- `src/app/ui/workspace/mod.rs` — layout integrace, panel rendering volání, replace handler
- `src/app/keymap.rs` — dispatch přesměrování na show_panel toggle
- `src/app/ui/workspace/menubar/mod.rs` — menu action přesměrování
- `locales/cs/ui.ftl` — nové i18n klíče
- `locales/en/ui.ftl` — nové i18n klíče
- `locales/sk/ui.ftl` — nové i18n klíče
- `locales/de/ui.ftl` — nové i18n klíče
- `locales/ru/ui.ftl` — nové i18n klíče
