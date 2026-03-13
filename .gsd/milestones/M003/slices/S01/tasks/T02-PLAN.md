---
estimated_steps: 8
estimated_files: 2
---

# T02: Přepis renderingu, scroll sync a napojení na volajícího

**Slice:** S01 — Editovatelný panel se syntax highlighting, diff a sync scrollem
**Milestone:** M003

## Description

Přepsat `render_history_split_view()` — levý panel z read-only `Label`+`LayoutJob` na editovatelný `TextEdit::multiline` s layouter callbackem (syntax highlighting + diff background), pravý panel z monochrome `LayoutJob` na `Label` s `LayoutJob` se syntax highlighting + diff background. Implementovat proportionální scroll sync, tab sync (editace → tab.content + modified), výchozí stav panelů podle počtu verzí. Upravit volajícího v `workspace/mod.rs` pro novou signaturu a borrow-checker kompatibilitu.

## Steps

1. **Upravit signaturu `render_history_split_view()`** — přidat parametry `highlighter: &Highlighter`, `theme_name: &str`, `ext: &str`, `fname: &str`, `font_size: f32`. Změnit return type z `bool` na `HistorySplitResult`. Přidat potřebné importy.

2. **Přepsat diff cache logiku** — invalidace nejen při změně `selected_index` ale i při změně `content_hash`. Po invalidaci zavolat `build_panel_texts()` a uložit výsledky do `HistoryViewState` (right_panel_text, left_diff_map, right_diff_map). `current_content` slouží přímo jako left panel text (editovatelný buffer).

3. **Přepsat levý panel na TextEdit + layouter** — `TextEdit::multiline(&mut history_view.current_content)` s layouter callbackem: (a) capture `&left_diff_map` a `&colors` do closure, (b) zavolat `highlighter.highlight()` pro syntax barvy, (c) klonovat LayoutJob, nastavit wrap_width, (d) zavolat `apply_diff_backgrounds()` s diff mapou, (e) `ui.fonts(|f| f.layout_job(job))`. TextEdit je `.code_editor()` s monospace fontem.

4. **Přepsat pravý panel na Label + LayoutJob se syntax** — (a) zavolat `highlighter.highlight(&right_panel_text, ...)` pro syntax barvy, (b) klonovat LayoutJob, (c) zavolat `apply_diff_backgrounds()` s right_diff_map, (d) `ui.add(Label::new(job))`. Pokud `selected_index` je None (1 verze), zobrazit prázdný panel.

5. **Implementovat scroll sync** — po renderování obou ScrollArea: (a) přečíst `left_output.state.offset.y` a `right_output.state.offset.y`, (b) porovnat s uloženým `left_scroll_y`/`right_scroll_y` s epsilon tolerancí (1.0px), (c) pokud se levý změnil a `scroll_source != Right` → sync pravý proportionálně, (d) analogicky pro pravý → levý, (e) aktualizovat uložené offsety. Proportionální: `target_y = (source_y / source_max) * target_max`. Nastavit přes `.vertical_scroll_offset()` na ScrollArea.

6. **Implementovat tab sync** — po TextEdit: pokud `response.changed()` → (a) aktualizovat `content_hash = xxh3_64(current_content.as_bytes())`, (b) nastavit `result.content_changed = true`. V `workspace/mod.rs` po render_history_split_view: pokud `result.content_changed` → propsát `current_content` do `tab.content`, nastavit `tab.modified = true`, `tab.last_edit = Some(Instant::now())`, `tab.save_status = SaveStatus::Modified`.

7. **Upravit `workspace/mod.rs`** — (a) Před mutable borrow na `ws.history_view` extrahovat z `ws.editor`: highlighter referenci (nemožné kvůli lifetime — místo toho předat potřebná data jako owned/cloned). Řešení: `let theme_name = settings.syntect_theme_name()`, `let ext = ws.editor.extension()`, `let fname = ws.editor.filename()`, `let font_size = Editor::current_editor_font_size(ui)`. Highlighter: buď extrahovat jako shared ref před if-blok, nebo přesunout highlighter do WorkspaceState (preferovat první). (b) Předat parametry do `render_history_split_view()`. (c) Zpracovat `HistorySplitResult` — close → `ws.history_view = None`, content_changed → tab sync. (d) Upravit inicializaci: `selected_index: if entries.len() > 1 { Some(0) } else { None }`, inicializovat nové fieldy (content_hash z xxh3_64, scroll offsety 0.0, scroll_source None, right_panel_text a diff mapy prázdné — budou naplněny při prvním renderu).

8. **Verifikace** — `cargo check`, `./check.sh`, vizuální kontrola v běžícím editoru: otevřít soubor s historií → ověřit syntax barvy v obou panelech → ověřit diff pozadí na správných řádcích → editovat v levém panelu → ověřit tab modified (●) → scrollovat jedním panelem → ověřit sync druhého.

## Must-Haves

- [ ] Levý panel je `TextEdit::multiline` s layouter callbackem (syntax + diff background)
- [ ] Pravý panel je `Label` s `LayoutJob` (syntax + diff background)
- [ ] Scroll sync funguje proportionálně bez feedback loop
- [ ] Editace v levém panelu → tab.content aktualizován + tab.modified = true
- [ ] Výchozí stav: 1 verze → selected_index=None (prázdný pravý panel), >1 verze → Some(0)
- [ ] Diff cache se invaliduje při změně content_hash (editace) i selected_index (navigace)
- [ ] Borrow checker v workspace/mod.rs vyřešen bez unsafe
- [ ] `cargo check` + `./check.sh` prochází

## Verification

- `cargo check` — kompilace bez chyb
- `./check.sh` — clippy + testy prochází
- Vizuální kontrola v běžícím editoru:
  - Levý panel: editovatelný, syntax barvy viditelné, diff řádky mají barevné pozadí
  - Pravý panel: read-only, syntax barvy viditelné, diff řádky mají barevné pozadí
  - Scroll: pohyb jedním panelem pohne druhým
  - Editace: změna v levém panelu → tab se označí jako modified (●)
  - Výchozí stav: soubor s 1 verzí → pravý panel prázdný

## Observability Impact

- Signals added/changed: `HistorySplitResult.content_changed` signalizuje tab sync potřebu; `content_hash` v HistoryViewState trackuje editační stav
- How a future agent inspects this: debugger breakpoint na render_history_split_view — HistoryViewState fieldy ukazují aktuální scroll, diff mapy, content hash
- Failure state exposed: pokud diff mapy jsou prázdné ale text není → diff recompute selhal; pokud content_changed je true ale tab.modified je false → tab sync selhal

## Inputs

- `src/app/ui/workspace/history/mod.rs` — T01 output: ScrollSource, rozšířený HistoryViewState, HistorySplitResult, PanelTexts, build_panel_texts(), compute_line_offsets(), apply_diff_backgrounds()
- `src/app/ui/workspace/mod.rs` — stávající volání render_history_split_view() + inicializace HistoryViewState
- `src/app/ui/editor/render/normal.rs` — vzor pro TextEdit + layouter + ScrollArea pattern
- S01-RESEARCH.md — detailní technický design pro rendering, scroll sync, tab sync, borrow checker řešení

## Expected Output

- `src/app/ui/workspace/history/mod.rs` — přepsaná `render_history_split_view()` s TextEdit+layouter levý panel, Label+LayoutJob+syntax pravý panel, scroll sync, nová signatura + return type
- `src/app/ui/workspace/mod.rs` — upravené volání s novými parametry, borrow-checker řešení, tab sync, výchozí stav panelů
