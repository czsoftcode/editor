---
id: M006
provides:
  - Inline project search panel v TopBottomPanel::bottom pod editorem (ne modální dialog)
  - Klik na výsledek → otevření souboru s fokusem na řádku, panel zůstane otevřený
  - Persistentní stav panelu (query, výsledky, togglery přežijí close/reopen)
  - Replace flow z inline panelu (toggle + input + Replace All → existující preview dialog)
  - Panel resize přes resizable(true) s default 250px, min 100px, max 60% výšky
  - In-file search (Ctrl+F) s regex/case-sensitive/whole-word togglery sdílející build_regex() engine
  - Smazání mrtvých modálních dialogů (render_project_search_dialog, poll_and_render_project_search_results)
  - i18n pro všechny nové UI prvky ve všech 5 jazycích
key_decisions:
  - TopBottomPanel::bottom("search_panel") PŘED CentralPanel v layout pořadí — egui vyžaduje bottom panel deklaraci před central panelem
  - Dual-index pattern (last_selected_index pro persistentní highlight + pending_jump_index pro consumable navigaci) — borrow checker neumožňuje open_file_in_ws uvnitř panel closure
  - show_panel nahrazuje show_input jako primární visibility flag — jednodušší stav, toggle/close nezmaže data
  - Mrtvé modální funkce smazány kompletně (ne podmíněně) — čistý kód bez mrtvých cest
  - Sdílení build_regex() engine mezi project search a in-file search — žádná duplikace logiky
  - Dedikované i18n klíče (search-*-toggle) pro in-file search místo sdílení project-search-* klíčů — odlišné tooltipy
patterns_established:
  - TopBottomPanel::bottom("search_panel") před CentralPanel v layout pořadí
  - Dual-index pattern pro panel navigaci (persistentní highlight + consumable jump)
  - Poll loop v panel rendereru (try_recv loop na začátku render funkce)
  - Toggle button pattern (selectable_label + on_hover_text) sjednocen mezi project search a in-file search
observability_surfaces:
  - ws.project_search.show_panel — runtime viditelnost panelu
  - ws.project_search.searching — poll loop aktivní (spinner indikátor)
  - ws.project_search.regex_error — inline chybová hláška v panelu
  - ws.project_search.pending_jump_index — transitní navigační signál, None po zpracování
  - editor.search_regex_error — červený text v search baru při nevalidním regex patternu
requirement_outcomes:
  - id: R026
    from_status: active
    to_status: validated
    proof: TopBottomPanel::bottom("search_panel") renderuje pod editorem s query inputem a togglery. cargo check + ./check.sh čisté, 229 testů pass.
  - id: R027
    from_status: active
    to_status: validated
    proof: pending_jump_index.take() → open_file_in_ws() + jump_to_location() + FocusedPanel::Editor. Panel zůstává otevřený.
  - id: R028
    from_status: active
    to_status: validated
    proof: show_panel=false nezmaže query, results, togglery. Reopen přes Ctrl+Shift+F zobrazí zachovaný stav.
  - id: R029
    from_status: active
    to_status: validated
    proof: CommandId::ProjectSearch dispatch v workspace/mod.rs toggles show_panel. Menu action nastavuje show_panel=true.
  - id: R030
    from_status: active
    to_status: validated
    proof: build_regex() + regex.find_iter() v update_search(). 3 toggle buttons v search_bar(). i18n 3 klíče × 5 jazyků. ./check.sh pass.
  - id: R031
    from_status: active
    to_status: validated
    proof: TopBottomPanel::bottom().resizable(true).default_height(250.0).min_height(100.0).max_height(60% screen).
  - id: R032
    from_status: active
    to_status: validated
    proof: Replace toggle + input + Replace All → compute_replace_previews() → show_replace_preview=true → render_replace_preview_dialog().
  - id: R033
    from_status: active
    to_status: validated
    proof: project-search-panel-title ve všech 5 jazycích + search-regex/case/word-toggle 6 klíčů × 5 jazyků.
  - id: R034
    from_status: active
    to_status: validated
    proof: searching flag + spinner + ctx.request_repaint() během poll loop. SearchBatch::Done ukončí spinner.
  - id: R035
    from_status: active
    to_status: validated
    proof: Výsledky seskupeny per-file s filename hlavičkou v ScrollArea.
  - id: R036
    from_status: active
    to_status: validated
    proof: build_match_layout_job() a build_context_layout_job() znovupoužity v panelu.
duration: ~62min (S01 42min + S02 20min)
verification_result: passed
completed_at: 2026-03-13
---

# M006: Inline Search Panel + Vylepšení In-file Search

**Project search přesunut z modálních dialogů do inline bottom panelu s plným query UI, per-file výsledky, klik→jump navigací a persistentním stavem; in-file search sjednocen s regex/case/whole-word engine z M005.**

## What Happened

**S01 (high-risk)** provedl kompletní přesun project search z modálních dialogů do inline `TopBottomPanel::bottom("search_panel")` pod editorem. Kritický layout order (bottom panel PŘED CentralPanel) byl správně implementován. Panel obsahuje query input s togglery (regex `.*`, case `Aa`, word `W`, replace `↔`, close `✕`), file filter, inline regex error, a scrollovatelný seznam per-file výsledků se zvýrazněním matchů a kontextovými řádky. Kliknutí na výsledek naviguje přes dual-index pattern (`last_selected_index` pro persistentní highlight + `pending_jump_index` pro consumable navigaci) — borrow checker vyžadoval zprostředkování přes index s handlerem po renderování panelu. Poll loop pro inkrementální streamování výsledků běží přímo v `render_search_panel()`. Replace flow funguje z panelu (toggle + input + Replace All → existující preview dialog). Stav panelu (query, výsledky, togglery) přežije close/reopen. Keymap dispatch (Ctrl+Shift+F → toggle), menu action (always-open), a Escape handling (zavře panel bez ztráty stavu) byly přepojeny. Mrtvé modální funkce (`render_project_search_dialog` ~190 řádků + `poll_and_render_project_search_results` ~185 řádků) kompletně smazány.

**S02 (low-risk)** rozšířil in-file search bar (Ctrl+F) o regex/case-sensitive/whole-word togglery. Do `Editor` structu přidány 4 nové fieldy (`search_use_regex`, `search_case_sensitive`, `search_whole_word`, `search_regex_error`). `update_search()` kompletně přepsán z primitivního `char_indices` + `eq_ignore_ascii_case` loop na `build_regex()` + `regex.find_iter()` — sdílí identický engine s project search. Toggle buttons vizuálně sjednoceny s project search panelem. i18n klíče `search-regex-toggle`, `search-case-toggle`, `search-word-toggle` přidány do všech 5 locale souborů.

## Cross-Slice Verification

### Success Criteria

1. **Ctrl+Shift+F → inline panel pod editorem** ✓ — `TopBottomPanel::bottom("search_panel")` na řádku 728 v workspace/mod.rs, PŘED `CentralPanel` na řádku 743. Panel obsahuje query input, togglery, a výsledky.

2. **Kliknutí na výsledek → editor otevře soubor s fokusem, panel zůstane otevřený** ✓ — `pending_jump_index.take()` → `open_file_in_ws()` + `jump_to_location()` + `FocusedPanel::Editor` (workspace/mod.rs:731-738). `show_panel` se nemění.

3. **Zavření a znovuotevření → zachované výsledky** ✓ — `show_panel=false` nezmaže `query`, `results`, `replace_text` ani togglery. Stav žije v `ProjectSearch` struct.

4. **Replace z inline panelu** ✓ — Replace toggle + TextEdit + Replace All → `compute_replace_previews()` → `show_replace_preview=true` → existující `render_replace_preview_dialog()`.

5. **Panel resize** ✓ — `resizable(true).default_height(250.0).min_height(100.0).max_height(ctx.screen_rect().height() * 0.6)`.

6. **In-file search s regex/case/whole-word togglery** ✓ — `build_regex()` importován v search.rs, `update_search()` přepsán na `regex.find_iter()`. 3 `selectable_label` toggle buttons za search inputem.

7. **cargo check + ./check.sh** ✓ — "Quality Gate: All checks passed successfully!" — 229 testů (192 unit + 37 integration), fmt OK, clippy OK.

### Definition of Done

- [x] Inline search panel renderuje výsledky v TopBottomPanel::bottom pod editorem
- [x] Kliknutí na výsledek otevře soubor s fokusem na řádku bez ztráty stavu panelu
- [x] Panel stav (výsledky, query, pozice) přežije close/reopen
- [x] Replace flow funguje z inline panelu (preview dialog zůstává modální)
- [x] In-file search (Ctrl+F) používá build_regex() s regex/case/whole-word togglery
- [x] i18n klíče pro nové UI prvky ve všech 5 jazycích (project-search-panel-title + 3 search-*-toggle klíče × 5 jazyků)
- [x] cargo check + ./check.sh projde čistě
- [x] Mrtvé modální dialogy smazány (grep → 0 výskytů render_project_search_dialog/poll_and_render_project_search_results)

Všechna kritéria splněna. Oba slicey [x] v roadmapě, oba slice summaries existují.

## Requirement Changes

- R026: active → validated — TopBottomPanel::bottom("search_panel") PŘED CentralPanel, editor viditelný a editovatelný nad panelem
- R027: active → validated — pending_jump_index → open_file_in_ws + jump_to_location + FocusedPanel::Editor
- R028: active → validated — show_panel=false nezmaže query/results/togglery
- R029: active → validated — keymap toggle show_panel, menu always-open
- R030: active → validated — build_regex() + regex.find_iter() v update_search(), 3 togglery, i18n
- R031: active → validated — resizable(true) default 250px, min 100, max 60%
- R032: active → validated — Replace toggle + input + Replace All → preview dialog
- R033: active → validated — project-search-panel-title + search-*-toggle klíče ve všech 5 jazycích
- R034: active → validated — poll loop + searching flag + spinner + request_repaint
- R035: active → validated — per-file seskupení s filename hlavičkou
- R036: active → validated — build_match/context_layout_job() znovupoužity v panelu

## Forward Intelligence

### What the next milestone should know
- Engine funkce v `search_picker.rs` (`build_regex`, `search_file_with_context`, `compute_replace_previews`, `apply_replacements`) jsou stabilní a znovupoužitelné — žádné změny v M006, pouze nové UI napojení.
- `ProjectSearch` struct v `types.rs` je nyní kompletní stav panelu — `show_panel`, `pending_jump_index`, `last_selected_index`, togglery, results, replace data. Rozšíření je přímočaré.
- In-file search fieldy (`search_use_regex`, `search_case_sensitive`, `search_whole_word`) žijí v `Editor` struct a jsou nezávislé na project search kontextu.
- Všechny i18n klíče pro search jsou kompletní — `project-search-*` (35 klíčů) + `search-*-toggle` (3 klíče) × 5 jazyků.

### What's fragile
- **egui layout pořadí** — `TopBottomPanel::bottom("search_panel")` MUSÍ být před `CentralPanel::default().show()`. Přidání dalšího bottom panelu vyžaduje pozornost na pořadí volání v `render_workspace()`.
- **Dual-index pattern** (last_selected_index + pending_jump_index) — logika v workspace/mod.rs po `render_search_panel()` závisí na tom, že panel nastaví oba indexy atomicky při kliknutí. Změna jednoho bez druhého rozbije highlight nebo navigaci.
- **Regex byte offsets** z `regex.find_iter()` předpokládají validní UTF-8 String — přechod na rope-based buffer by vyžadoval přepis matchingu.

### Authoritative diagnostics
- `ws.project_search.show_panel` — definitivní runtime viditelnost panelu
- `ws.project_search.searching` — definitivní poll loop stav (nastavuje se přímo poll loop)
- `editor.search_regex_error` — inline regex error v search baru
- `grep 'render_project_search_dialog' src/` — 0 výsledků = mrtvý kód kompletně smazán

### What assumptions changed
- Plán předpokládal `panel_height: f32` field v ProjectSearch — egui TopBottomPanel.resizable(true) si výšku spravuje interně, explicitní field zbytečný
- Plán předpokládal `open_and_jump()` přímo v panelu — borrow checker vyžaduje zprostředkování přes pending_jump_index
- Plán předpokládal sdílení i18n klíčů mezi project search a in-file search togglery — dedikované klíče jsou lepší kvůli odlišným tooltipům

## Files Created/Modified

- `src/app/ui/workspace/state/types.rs` — ProjectSearch rozšířen o show_panel, last_selected_index, pending_jump_index; show_input odstraněn
- `src/app/ui/search_picker.rs` — nová render_search_panel() (~230 řádků) s poll loop, query UI, výsledky, replace; smazány render_project_search_dialog() (~190 řádků) a poll_and_render_project_search_results() (~185 řádků)
- `src/app/ui/workspace/mod.rs` — layout integrace panelu, pending_jump_index handler, keymap toggle dispatch, Escape handling, smazání mrtvých volání
- `src/app/ui/workspace/menubar/mod.rs` — menu action přesměrování na show_panel
- `src/app/ui/editor/mod.rs` — 4 nové fieldy v Editor struct (search_use_regex, search_case_sensitive, search_whole_word, search_regex_error)
- `src/app/ui/editor/search.rs` — přepsaný update_search() na build_regex() + regex.find_iter(), 3 toggle buttons v search_bar(), regex error zobrazení
- `locales/cs/ui.ftl` — project-search-panel-title + 3 search-*-toggle klíče
- `locales/en/ui.ftl` — project-search-panel-title + 3 search-*-toggle klíče
- `locales/sk/ui.ftl` — project-search-panel-title + 3 search-*-toggle klíče
- `locales/de/ui.ftl` — project-search-panel-title + 3 search-*-toggle klíče
- `locales/ru/ui.ftl` — project-search-panel-title + 3 search-*-toggle klíče
