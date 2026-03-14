---
estimated_steps: 9
estimated_files: 3
---

# T02: Výsledky + klik → jump + poll loop + replace flow

**Slice:** S01 — Inline project search panel
**Milestone:** M006

## Description

Hlavní funkční jádro search panelu — rendering výsledků per-file se zvýrazněním, poll loop pro inkrementální streamování, klik na výsledek s fokus transferem do editoru, a replace flow spuštění z panelu. Po tomto tasku je search panel plně funkční (bez keymap toggle a i18n).

## Steps

1. V `render_search_panel()` přidat ScrollArea pod input řádkem pro výsledky:
   - `ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| { ... })`
   - Celá plocha panelu pod inputem = prostor pro výsledky

2. Poll loop pro SearchBatch:
   - `if let Some(rx) = &ws.project_search.search_rx { loop { match rx.try_recv() { ... } } }`
   - `SearchBatch::Results(results)` → akumulace do `ws.project_search.results`
   - `SearchBatch::Done` → `ws.project_search.searching = false`, drop rx
   - `SearchBatch::Error(msg)` → toast, `ws.project_search.searching = false`
   - Pokud `searching == true` → `ctx.request_repaint()` pro další poll

3. Spinner + indikátor:
   - `if ws.project_search.searching { ui.spinner(); ui.label(t.t("project-search-searching")); }`
   - Po dokončení: `ui.label(format!("{} {}", results.len(), t.t("project-search-results-title")))` — nebo přesnější počet matchů

4. Per-file seskupení výsledků:
   - Seskupit `ws.project_search.results` dle `result.path`
   - Pro každý soubor: filename hlavička jako bold label (nebo CollapsingHeader pro collapsible)
   - Relativní cesta k souboru, počet matchů v souboru

5. Rendering matchů v každém souboru:
   - Pro každý SearchResult v daném souboru:
     - Match řádky: `build_match_layout_job()` — existující funkce, oranžový bg na match ranges
     - Kontextové řádky: `build_context_layout_job()` — existující funkce, dim barva
     - Separátor `···` mezi nesouvisejícími bloky (přenést z modálního dialogu)
   - `Label::new(layout_job).sense(Sense::click())` pro klikatelnost

6. Klik na výsledek handler:
   - Při kliknutí: zavolat helper funkci v workspace/mod.rs nebo přímo:
     - `open_and_jump(ws, &result.path, result.line)` (import z state/actions.rs)
     - `ws.focused_panel = FocusedPanel::Editor`
     - `ws.project_search.last_selected_index = Some(global_index)`
   - POZOR: `open_and_jump` potřebuje absolutní cestu — výsledky mají relativní, přepočítat přes `ws.project_root`

7. Vizuální indikace naposledy navštíveného výsledku:
   - `last_selected_index` → highlight pozadí řádku (subtle tint) pro vizuální feedback

8. Replace flow z panelu:
   - Replace All button v panelu (vedle replace inputu, podmíněný na `show_replace && !results.is_empty()`)
   - Při kliknutí: `compute_replace_previews(&results, &replace_query, &regex, &ws.project_root)`
   - Nastavit `ws.project_search.replace_previews = Some(previews)`
   - Nastavit `ws.project_search.show_replace_preview = true`
   - Existující `render_replace_preview_dialog()` se vykreslí normálně (už je v render_workspace)

9. Ověřit že replace flow funguje end-to-end: Replace All → preview dialog → confirm → pending_replace → workspace handler → snapshot + write + tab refresh + toast

## Must-Haves

- [ ] Výsledky se zobrazují per-file se zvýrazněním matchů (LayoutJob)
- [ ] Kontextové řádky s dim barvou a separátory
- [ ] Poll loop inkrementálně akumuluje výsledky s request_repaint()
- [ ] Spinner indikátor během searching
- [ ] Klik na výsledek → open_and_jump() + fokus na editor
- [ ] Replace All z panelu spouští existující preview dialog
- [ ] Existující 20 unit testů stále pass

## Verification

- `cargo check` — čistá kompilace
- `cargo test --lib app::ui::search_picker::tests` — 20 testů pass
- `grep 'open_and_jump\|open_file_in_ws' src/app/ui/search_picker.rs` → nalezeno (klik handler)
- `grep 'build_match_layout_job\|build_context_layout_job' src/app/ui/search_picker.rs` → multiple výskyty (rendering v panelu)

## Inputs

- T01 výstup: `render_search_panel()` skeleton s query inputem a togglery
- `src/app/ui/search_picker.rs` — engine funkce a LayoutJob buildery
- `src/app/ui/workspace/state/actions.rs` — `open_and_jump()` helper

## Observability Impact

- `ws.project_search.searching` — `true` indikuje běžící poll loop v panelu; přechod na `false` = search done/error
- `ws.project_search.results.len()` — počet výsledků akumulovaných přes poll loop; roste inkrementálně
- `ws.project_search.last_selected_index` — `Some(idx)` po kliknutí na výsledek, `None` po zpracování (open_and_jump)
- `ws.project_search.show_replace_preview` — `true` = replace preview dialog otevřen z panelu
- `ws.project_search.pending_replace` — `true` = replace potvrzeno, čeká na zpracování v workspace handler
- Toast chyby: `SearchBatch::Error(msg)` → toast v UI; I/O chyby replace → toast
- Spinner v panelu: viditelný runtime signál pro probíhající search
- `ctx.request_repaint()` — voláno pokud `searching == true` pro continuální poll loop

## Expected Output

- `src/app/ui/search_picker.rs` — kompletní render_search_panel() s výsledky, poll loop, klik handlery, replace flow
- `src/app/ui/workspace/mod.rs` — případné úpravy pro replace handler propojení
