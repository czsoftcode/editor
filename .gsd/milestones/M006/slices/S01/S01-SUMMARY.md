---
id: S01
parent: M006
milestone: M006
provides:
  - Inline project search panel v TopBottomPanel::bottom pod editorem
  - Query input s regex/case/whole-word/file filter togglery
  - Per-file seskupení výsledků s match highlighting a kontextovými řádky
  - Klik na výsledek → open_file_in_ws + jump_to_location + fokus transfer na editor
  - Poll loop pro inkrementální streamování výsledků s spinner indikátorem
  - Replace flow z panelu (toggle + input + Replace All → existující preview dialog)
  - Persistentní stav panelu (close nezmaže query ani results)
  - Panel resize přes resizable(true) s default 250px
  - Keymap toggle Ctrl+Shift+F, menu action, Escape handling
  - Smazání mrtvých modálních dialogů (render_project_search_dialog, poll_and_render_project_search_results)
  - i18n klíč project-search-panel-title ve všech 5 jazycích
requires:
  - slice: none
    provides: first slice in M006
affects:
  - M006/S02
key_files:
  - src/app/ui/workspace/state/types.rs
  - src/app/ui/search_picker.rs
  - src/app/ui/workspace/mod.rs
  - src/app/ui/workspace/menubar/mod.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - TopBottomPanel::bottom("search_panel") PŘED CentralPanel v layout pořadí — egui vyžaduje bottom panel deklaraci před central panelem
  - Dual-index pattern (last_selected_index pro persistentní highlight + pending_jump_index pro consumable navigaci přes .take()) — borrow checker neumožňuje open_file_in_ws uvnitř panel closure
  - Poll loop přímo v render_search_panel() na začátku (před UI renderingem) — nezávislý na modálním dialogu
  - Keymap dispatch pro ProjectSearch řešen přímo v workspace/mod.rs (toggle) vs. menu always-open — bez přidávání signálu do MenuActions
  - Escape handling jako globální key check před panel renderem — funguje bez fokusu na panelu, kontroluje show_replace_preview
  - Mrtvé modální funkce smazány kompletně (ne podmíněně) — čistý kód bez mrtvých cest
  - show_input field odstraněn z ProjectSearch — nahrazen show_panel
patterns_established:
  - TopBottomPanel::bottom("search_panel") před CentralPanel v layout pořadí
  - Dual-index pattern: last_selected_index (persistent highlight) + pending_jump_index (consumable navigation)
  - Poll loop v panel rendereru (try_recv loop na začátku render funkce)
observability_surfaces:
  - ProjectSearch.show_panel — runtime viditelnost panelu (toggle via Ctrl+Shift+F, close via Escape/✕)
  - ProjectSearch.searching — spinner indikátor, true = poll loop aktivní
  - ProjectSearch.regex_error — inline chybová hláška v panelu
  - ProjectSearch.last_selected_index — persistentní vizuální highlight kliknutého výsledku
  - ProjectSearch.pending_jump_index — transitní navigační signál, None po zpracování
  - SearchBatch::Error → toast chyba v UI
drill_down_paths:
  - .gsd/milestones/M006/slices/S01/tasks/T01-SUMMARY.md
  - .gsd/milestones/M006/slices/S01/tasks/T02-SUMMARY.md
  - .gsd/milestones/M006/slices/S01/tasks/T03-SUMMARY.md
duration: ~42min (T01 15min + T02 12min + T03 15min)
verification_result: passed
completed_at: 2026-03-13
---

# S01: Inline project search panel

**Project search přesunut z modálních dialogů do inline bottom panelu s plným query UI, per-file výsledky se zvýrazněním, klik→jump navigací, replace flow, a persistentním stavem.**

## What Happened

**T01** rozšířil `ProjectSearch` struct o `show_panel` a `last_selected_index`, vytvořil `render_search_panel()` (~230 řádků) s query inputem, togglery (regex `.*`, case `Aa`, word `W`, replace `↔`, close `✕`), file filter inputem, inline regex error, a scrollovatelným seznamem výsledků per-file. Panel je integrován v `render_workspace()` jako `TopBottomPanel::bottom("search_panel")` PŘED `CentralPanel` — kritický layout order pro egui. Kliknuté výsledky se zpracovávají přes `last_selected_index.take()` po renderování panelu kvůli borrow checker omezením.

**T02** přidal poll loop na začátek `render_search_panel()` pro inkrementální streamování výsledků (try_recv → akumulace do results, spinner + request_repaint), Replace All button spouštějící existující preview dialog, vizuální highlight naposledy kliknutého výsledku (subtilní modrý tint), a refaktoroval navigaci na dual-index pattern (`last_selected_index` pro persistentní highlight + `pending_jump_index` pro consumable navigaci s fokus transferem na editor).

**T03** přepojil keymap dispatch (Ctrl+Shift+F → toggle show_panel), přesměroval menu action (always-open), přidal Escape handling (zavře panel bez ztráty stavu, respektuje otevřený replace preview), smazal mrtvé modální funkce (`render_project_search_dialog` ~190 řádků + `poll_and_render_project_search_results` ~185 řádků), odstranil `show_input` field, a přidal i18n klíč `project-search-panel-title` do všech 5 jazyků.

## Verification

- `./check.sh` — 192 testů pass, fmt OK, clippy OK, cargo check čistý
- 20 search_picker unit testů pass (engine funkce nezměněny)
- `grep -c 'show_panel' src/app/ui/workspace/state/types.rs` → 2 ✓
- `grep -c 'search_panel' src/app/ui/workspace/mod.rs` → 2 ✓
- `grep -c 'render_project_search_dialog' src/app/ui/workspace/mod.rs` → 0 ✓ (mrtvý dialog smazán)
- `grep -c 'show_input' src/app/ui/workspace/state/types.rs` → 0 ✓ (field odstraněn)
- i18n `project-search-panel-title` v cs/en/sk/de/ru — 1 per jazyk ✓

## Requirements Advanced

- R026 (Inline search panel) — panel renderuje v TopBottomPanel::bottom pod editorem s query inputem a togglery, editor viditelný nad panelem
- R027 (Kliknutí na výsledek → fokus) — klik na výsledek otevře soubor přes open_file_in_ws + jump_to_location + FocusedPanel::Editor, panel zůstane otevřený
- R028 (Persistentní stav) — show_panel=false nezmaže query, results, ani togglery; reopen zobrazí stejný stav
- R029 (Ctrl+Shift+F toggle) — keymap dispatch toggles show_panel, menu action vždy otevírá
- R031 (Panel resize) — TopBottomPanel::bottom().resizable(true) s default 250px, min 100px, max 60% výšky
- R032 (Replace flow z panelu) — Replace toggle + input + Replace All → compute_replace_previews → existující preview dialog
- R033 (i18n) — project-search-panel-title ve všech 5 jazycích, ostatní texty sdíleny s existujícími klíči
- R034 (Spinner/indikátor) — searching flag + spinner + "Hledám..." + request_repaint během poll loop
- R035 (Per-file seskupení) — výsledky seskupeny per-file s filename hlavičkou
- R036 (Kontextové řádky + highlighting) — build_match_layout_job() a build_context_layout_job() znovupoužity v panelu

## Requirements Validated

- R026 — TopBottomPanel::bottom("search_panel") renderuje pod editorem, editor zůstává editovatelný. cargo check + ./check.sh čisté.
- R027 — pending_jump_index.take() → open_file_in_ws + jump_to_location + FocusedPanel::Editor. Panel zůstává otevřený.
- R028 — show_panel=false nezmaže query, results, togglery. Reopen přes Ctrl+Shift+F zobrazí zachovaný stav.
- R029 — Keymap dispatch v workspace/mod.rs toggles show_panel. Menu action nastavuje show_panel=true.
- R031 — TopBottomPanel::bottom().resizable(true).default_height(250.0).min_height(100.0).max_height(60% screen). Nativní egui resize handle.
- R032 — Replace toggle + input + Replace All button → compute_replace_previews → show_replace_preview = true → existující render_replace_preview_dialog().
- R033 — project-search-panel-title ve všech 5 jazycích (cs/en/sk/de/ru). Ostatní UI texty sdíleny s existujícími project-search-* klíči.
- R034 — searching flag + spinner v panelu + ctx.request_repaint() během poll loop. SearchBatch::Done ukončí spinner.
- R035 — Výsledky seskupeny per-file, filename jako hlavička, matche pod ní.
- R036 — build_match_layout_job() (oranžový bg match) a build_context_layout_job() (dim kontext) znovupoužity v panelu.

## New Requirements Surfaced

- none

## Requirements Invalidated or Re-scoped

- none

## Deviations

- Task plan psal `t: &Translator` ale kódová báze používá `i18n: &I18n` — použit reálný typ.
- Panel renderuje výsledky přímo v T01 (ne jen skeleton) — engine funkce jsou sdílené a výsledky se přirozeně akumulují v results vektoru.
- Navigace přes `pending_jump_index` (dual-index) místo plánovaného `open_and_jump()` přímo v panelu — borrow checker neumožňuje mutabilní přístup k workspace uvnitř panel closure.
- Keymap dispatch řešen v workspace/mod.rs (ne v keymap.rs jak plánováno) — keymap.rs jen vrací CommandId, nemá přístup k workspace state.
- Escape handling jako globální key check před panel renderem (ne uvnitř panel closure) — jednodušší a funguje bez fokusu na panelu.

## Known Limitations

- Vizuální UAT (panel zobrazení, resize tažením, fokus transfer po kliknutí) vyžaduje manuální ověření na desktopu — headless build ověří pouze kompilaci a unit testy.
- Panel výška (panel_height field) není explicitně uložena — egui si ji spravuje interně přes resizable panel state.

## Follow-ups

- none

## Files Created/Modified

- `src/app/ui/workspace/state/types.rs` — ProjectSearch rozšířen o show_panel, last_selected_index, pending_jump_index; show_input odstraněn
- `src/app/ui/search_picker.rs` — nová render_search_panel() (~230 řádků) s poll loop, query UI, výsledky, replace; smazány render_project_search_dialog() a poll_and_render_project_search_results()
- `src/app/ui/workspace/mod.rs` — layout integrace panelu, pending_jump_index handler, keymap toggle dispatch, Escape handling, smazání mrtvých volání
- `src/app/ui/workspace/menubar/mod.rs` — menu action přesměrování na show_panel
- `locales/cs/ui.ftl` — project-search-panel-title
- `locales/en/ui.ftl` — project-search-panel-title
- `locales/sk/ui.ftl` — project-search-panel-title
- `locales/de/ui.ftl` — project-search-panel-title
- `locales/ru/ui.ftl` — project-search-panel-title

## Forward Intelligence

### What the next slice should know
- `build_regex()` funguje beze změn a je znovupoužitelná pro S02 in-file search — přijímá `SearchOptions` (use_regex, case_sensitive, whole_word) a vrací `Result<Regex, String>`.
- `SearchOptions` struct žije v `search_picker.rs` a má fieldy `use_regex`, `case_sensitive`, `whole_word` — S02 potřebuje analogické fieldy v `Editor` structu pro in-file search kontext.
- i18n klíče pro togglery (project-search-regex-toggle, project-search-case-toggle, project-search-word-toggle) existují a mohou být sdíleny nebo prefixovány pro in-file search.

### What's fragile
- egui layout pořadí — `TopBottomPanel::bottom("search_panel")` MUSÍ být před `CentralPanel::default().show()`. Přidání dalšího bottom panelu vyžaduje pozornost na pořadí volání.
- Dual-index pattern (last_selected_index + pending_jump_index) — logika zpracování v workspace/mod.rs po render_search_panel() závisí na tom, že panel nastaví oba indexy atomicky při kliknutí.

### Authoritative diagnostics
- `ws.project_search.show_panel` — runtime viditelnost, trust level: definitivní
- `ws.project_search.searching` — poll loop aktivní, trust level: definitivní (nastavuje se přímo poll loop)
- `grep 'render_project_search_dialog' src/` — 0 výsledků = mrtvý kód kompletně smazán

### What assumptions changed
- Plán předpokládal `panel_height: f32` field v ProjectSearch — egui TopBottomPanel.resizable(true) si výšku spravuje interně, explicitní field zbytečný
- Plán předpokládal `open_and_jump()` přímo v panelu — borrow checker vyžaduje zprostředkování přes pending_jump_index s handlerem po renderování
