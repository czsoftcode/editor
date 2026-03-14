---
id: T02
parent: S01
milestone: M006
provides:
  - Poll loop v render_search_panel() pro inkrementální streamování výsledků
  - Replace All button v panelu → existující preview dialog
  - Vizuální highlight naposledy navštíveného výsledku
  - pending_jump_index pattern pro navigaci z panelu s fokus transferem na editor
key_files:
  - src/app/ui/search_picker.rs
  - src/app/ui/workspace/mod.rs
  - src/app/ui/workspace/state/types.rs
key_decisions:
  - Zavedení pending_jump_index odděleného od last_selected_index — last_selected_index zůstává pro vizuální highlight (nemažeme), pending_jump_index se spotřebuje přes .take() pro navigaci. Důvod: highlight musí přetrvat po kliknutí, ale navigace se provádí jen jednou.
  - Poll loop umístěn na začátek render_search_panel() (před UI rendering) — zajišťuje akumulaci výsledků před jejich vykreslením ve stejném frame
patterns_established:
  - Dual-index pattern: last_selected_index (persistent highlight) + pending_jump_index (consumable navigation) — řeší borrow checker omezení panelu
observability_surfaces:
  - ws.project_search.searching — true = poll loop aktivní, spinner viditelný
  - ws.project_search.results.len() — počet akumulovaných výsledků, roste inkrementálně
  - ws.project_search.pending_jump_index — Some(idx) transitně po kliknutí, None po zpracování
  - ws.project_search.last_selected_index — Some(idx) persistentní highlight, nastavuje se při kliknutí
  - SearchBatch::Error → toast chyba v UI
  - ctx.request_repaint() voláno při searching == true pro continuální polling
duration: 12min
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T02: Výsledky + klik → jump + poll loop + replace flow

**Poll loop, replace flow z panelu, vizuální highlight kliknutého výsledku, a fokus transfer na editor — search panel je plně funkční.**

## What Happened

1. Přidán poll loop na začátek `render_search_panel()`:
   - `rx.try_recv()` loop akumuluje `SearchBatch::Results` do `ws.project_search.results`
   - `SearchBatch::Done` → `searching = false`, drop rx
   - `SearchBatch::Error` → toast + `searching = false`
   - Disconnected rx → graceful cleanup
   - `ctx.request_repaint()` po lopu pokud `searching == true`

2. Replace All button přidán do replace inputu řádku:
   - `ui.horizontal()` s TextEdit + "Replace All" button
   - Button aktivní jen když `!results.is_empty()`
   - Kliknutí → `compute_replace_previews()` → nastaví `replace_previews` + `show_replace_preview = true`
   - Existující `render_replace_preview_dialog()` se vykreslí normálně

3. Vizuální highlight naposledy kliknutého výsledku:
   - `is_last_selected = ws.project_search.last_selected_index == Some(i)`
   - Subtilní modrý tint přes `painter().rect_filled()` na match řádku

4. Navigační pattern refaktorován:
   - Nový field `pending_jump_index: Option<usize>` v `ProjectSearch`
   - Panel nastaví oba: `last_selected_index` (highlight) + `pending_jump_index` (navigace)
   - workspace/mod.rs: `pending_jump_index.take()` → `open_file_in_ws` + `jump_to_location` + `FocusedPanel::Editor`

## Verification

- `cargo check` — čistá kompilace, 0 warningů
- `cargo fmt` — formátovaný kód
- `cargo clippy` — 0 warningů
- `./check.sh` — všech 192 testů pass, fmt + clippy + check čisté
- `cargo test search_picker::tests` — 20 testů pass (engine nezměněn)
- `grep -c 'build_match_layout_job\|build_context_layout_job' src/app/ui/search_picker.rs` → 8 (rendering v panelu i modálu)
- `grep 'pending_jump_index' src/app/ui/workspace/mod.rs` → nalezeno (klik handler s open_file_in_ws + jump_to_location)
- `grep 'compute_replace_previews' src/app/ui/search_picker.rs` → nalezeno (replace flow z panelu)

### Slice-level verification (partial — T02 intermediate):
- ✅ `cargo check` čistá
- ✅ `./check.sh` — 192 testů pass
- ✅ `show_panel` v types.rs ≥1 (2)
- ✅ `search_panel` v mod.rs ≥1 (2)
- ⬜ `render_project_search_dialog` v mod.rs → 0 — zatím 2 (smazáno/podmíněno v T03)
- ✅ 20 unit testů pass

## Diagnostics

- Poll loop stav: `ws.project_search.searching` — `true` = spinner + request_repaint, `false` = hotovo
- Výsledky: `ws.project_search.results.len()` — roste inkrementálně během search
- Kliknutý výsledek: `ws.project_search.last_selected_index` — persistentní highlight, `ws.project_search.pending_jump_index` — transitní navigace
- Replace flow: `ws.project_search.show_replace_preview` → preview dialog, `ws.project_search.pending_replace` → workspace handler
- Chyby: `SearchBatch::Error` → toast, I/O chyby replace → toast

## Deviations

- Task plan specifikoval `open_and_jump(ws, &result.path, result.line)` přímo v search_picker.rs — místo toho použit `pending_jump_index` pattern s handlerem v workspace/mod.rs. Důvod: borrow checker neumožňuje `open_and_jump` uvnitř panelu (potřebuje `&mut ws` ale panel drží `&mut ws` během renderování). Tento pattern byl zaveden v T01 a je konzistentní.
- Přidán `pending_jump_index` field místo jednoho `last_selected_index` — oddělení highlight (persistentní) od navigace (consumable). Task plan nepočítal s potřebou perzistentního highlightu po navigaci.

## Known Issues

- Modální dialog (`render_project_search_dialog` + `poll_and_render_project_search_results`) stále aktivní — T03 ho odstraní.
- Poll loop existuje i v `poll_and_render_project_search_results()` (modální) — duplikátní polling nevadí, protože rx je sdílený a try_recv je thread-safe. Po smazání modálu v T03 zůstane jen panelový poll loop.

## Files Created/Modified

- `src/app/ui/search_picker.rs` — poll loop v render_search_panel(), Replace All button, vizuální highlight, pending_jump_index nastavení
- `src/app/ui/workspace/mod.rs` — pending_jump_index.take() handler s FocusedPanel::Editor
- `src/app/ui/workspace/state/types.rs` — nový field pending_jump_index: Option<usize>
- `.gsd/milestones/M006/slices/S01/tasks/T02-PLAN.md` — přidána Observability Impact sekce
