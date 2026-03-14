---
estimated_steps: 8
estimated_files: 4
---

# T01: State rozšíření + panel skeleton + layout integrace

**Slice:** S01 — Inline project search panel
**Milestone:** M006

## Description

Rozšířit `ProjectSearch` struct o panel-specifické fieldy, vytvořit skeleton `render_search_panel()` funkci s query inputem a togglery, a vložit panel do `render_workspace()` layout pořadí PŘED `CentralPanel`. Dokáže že egui layout pořadí funguje — panel se zobrazí pod editorem, editor nad ním zůstane viditelný. Search se spouští z panelu a výsledky streamují do ProjectSearch.results.

## Steps

1. V `src/app/ui/workspace/state/types.rs` rozšířit `ProjectSearch`:
   - Přidat `show_panel: bool` (default false)
   - Přidat `last_selected_index: Option<usize>` (default None)
   - Ověřit že `show_input` zůstane zatím (T03 ho smaže po přepojení keymapu)

2. V `src/app/ui/search_picker.rs` vytvořit novou funkci `render_search_panel()`:
   - Signature: `pub fn render_search_panel(ctx: &egui::Context, ws: &mut WorkspaceState, t: &Translator)`
   - Early return pokud `!ws.project_search.show_panel`
   - `TopBottomPanel::bottom("search_panel").resizable(true).default_height(250.0).min_height(100.0).max_height(ctx.screen_rect().height() * 0.6).show(ctx, |ui| { ... })`

3. V panelu vykreslit horní řádek s query inputem a togglery:
   - Panel title/close button (✕) napravo
   - Query `TextEdit::singleline` s `focus_requested` podporou
   - Toggle buttons: regex (`.*`), case (`Aa`), whole-word (`W`) jako `selectable_label` — stejný pattern jako v render_project_search_dialog()
   - File filter input pod togglery
   - Replace toggle button (↔) + replace input (podmíněný na `show_replace`)

4. Search spouštění:
   - Enter v query inputu nebo klik na Search button → `build_regex()` pro validaci → pokud Ok, spustit `run_project_search()` v threadu
   - Regex error → inline červeně pod inputem (přesně jako v modálu)
   - Search spuštění vyčistí předchozí výsledky a nastaví `searching = true`

5. V `src/app/ui/workspace/mod.rs` — vložit volání `render_search_panel()` na správné místo v layout pořadí:
   - Najít místo PŘED `CentralPanel::default().show()` (cca řádek ~720)
   - Přidat: `search_picker::render_search_panel(ctx, ws, t);`
   - Panel se vykreslí jen pokud `ws.project_search.show_panel`

6. Dočasné testovací nastavení: v keymap dispatch pro `CommandId::ProjectSearch` přidat `show_panel = true` (zatím ne toggle, jen open — T03 přidá toggle logiku)

7. Ověřit kompilaci: `cargo check` — žádné chyby

8. Ověřit že existující testy stále procházejí (engine funkce nezměněny)

## Must-Haves

- [ ] `TopBottomPanel::bottom("search_panel")` vložen PŘED `CentralPanel` v render_workspace()
- [ ] Panel se zobrazí s query inputem a togglery při show_panel = true
- [ ] Search se spouští z panelu (Enter/button) přes run_project_search()
- [ ] Regex error zobrazena inline v panelu
- [ ] `cargo check` čistá kompilace

## Verification

- `cargo check` — čistá kompilace
- `grep 'search_panel' src/app/ui/workspace/mod.rs` → nalezeno
- `grep 'show_panel' src/app/ui/workspace/state/types.rs` → nalezeno
- `grep 'render_search_panel' src/app/ui/search_picker.rs` → nalezeno
- Existující 20 unit testů pass: `cargo test --lib app::ui::search_picker::tests`

## Observability Impact

- **`ProjectSearch.show_panel: bool`** — nový runtime signál viditelnosti panelu. Inspektovatelný v debuggeru nebo přes egui state. Hodnota `true` = panel zobrazen, `false` = skryt.
- **`ProjectSearch.last_selected_index: Option<usize>`** — transitní signál kliknutého výsledku. Po zpracování se vždy resetuje na `None` (`.take()`). Pokud zůstane `Some` déle než jeden frame, je to bug.
- **`ProjectSearch.searching: bool`** — indikátor běžícího search threadu (existující). Panel zobrazuje spinner při `true`.
- **`ProjectSearch.regex_error: Option<String>`** — chybová hláška zobrazená inline v panelu při nevalidním regexu.
- **Failure visibility:** nevalidní regex → červená hláška v panelu. Search chyba → toast přes `SearchBatch::Error`.

## Inputs

- `src/app/ui/search_picker.rs` — engine funkce (build_regex, run_project_search) a existující modální UI jako reference
- `src/app/ui/workspace/mod.rs` — layout pořadí v render_workspace()
- `src/app/ui/workspace/state/types.rs` — ProjectSearch struct

## Expected Output

- `src/app/ui/workspace/state/types.rs` — rozšířený ProjectSearch s show_panel, last_selected_index
- `src/app/ui/search_picker.rs` — nová render_search_panel() funkce se skeleton UI
- `src/app/ui/workspace/mod.rs` — volání render_search_panel() v layout pořadí
