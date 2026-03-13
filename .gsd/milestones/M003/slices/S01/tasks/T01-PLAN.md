---
estimated_steps: 7
estimated_files: 1
---

# T01: Datový model, diff→panel logika a helper funkce

**Slice:** S01 — Editovatelný panel se syntax highlighting, diff a sync scrollem
**Milestone:** M003

## Description

Připravit všechny stavební bloky potřebné pro rendering: rozšíření `HistoryViewState` o nové fieldy pro scroll sync a diff mapy, funkce pro sestavení panel textů + diff map z diff výstupu, funkce pro aplikaci diff background barev na `LayoutJob` sections, a nový return type `HistorySplitResult`. Vše unit-testované.

## Steps

1. **Přidat `ScrollSource` enum** — `Left`, `Right`, `None`. Jednoduché derivace (Debug, Clone, Copy, PartialEq).

2. **Rozšířit `HistoryViewState`** — nové fieldy: `content_hash: u64`, `left_scroll_y: f32`, `right_scroll_y: f32`, `scroll_source: ScrollSource`, `right_panel_text: String`, `left_diff_map: Vec<ChangeTag>`, `right_diff_map: Vec<ChangeTag>`. Přidat import `xxhash_rust::xxh3::xxh3_64`.

3. **Napsat `HistorySplitResult` struct** — `close: bool`, `content_changed: bool`. Jednoduchý return type pro render funkci.

4. **Napsat `build_panel_texts(diff_lines: &[DiffLine]) -> PanelTexts`** — projde diff řádky, sestaví `left_text` (Equal+Insert řádky spojené), `right_text` (Equal+Delete řádky spojené), `left_diff_map: Vec<ChangeTag>` (per-řádek tag pro levý panel), `right_diff_map: Vec<ChangeTag>` (per-řádek tag pro pravý panel). `PanelTexts` je helper struct.

5. **Napsat `compute_line_offsets(text: &str) -> Vec<usize>`** — vrací byte offset začátku každého řádku. Řádek 0 začíná na offset 0, řádek 1 na offset za prvním `\n`, atd. Potřebné pro mapování LayoutJob section byte_range na řádek.

6. **Napsat `apply_diff_backgrounds(job: &mut LayoutJob, text: &str, diff_map: &[ChangeTag], colors: &DiffColors)`** — projde sections v LayoutJob, pro každou section zjistí řádek z `byte_range.start` (binary search v line_offsets), pokud diff_map[řádek] != Equal, nastaví `section.format.background` na příslušnou diff barvu. Equal řádky ponechá beze změny.

7. **Přidat unit testy:**
   - `build_panel_texts` — ověřit správné rozdělení diff řádků do panelů a korektnost diff map.
   - `build_panel_texts` s prázdným diff — oba texty prázdné, mapy prázdné.
   - `build_panel_texts` s identickým textem — všechny řádky Equal v obou panelech.
   - `compute_line_offsets` — ověřit byte offsety pro multi-byte (UTF-8) text.
   - `apply_diff_backgrounds` — vytvořit minimální LayoutJob, aplikovat diff mapy, ověřit že sections mají správné background barvy.

## Must-Haves

- [ ] `ScrollSource` enum existuje s variantami Left, Right, None
- [ ] `HistoryViewState` rozšířen o content_hash, scroll offsety, scroll_source, right_panel_text, diff mapy
- [ ] `HistorySplitResult` struct s close a content_changed fieldy
- [ ] `build_panel_texts()` korektně rozděluje diff řádky — left=Equal+Insert, right=Equal+Delete
- [ ] `compute_line_offsets()` vrací správné byte offsety i pro UTF-8 text
- [ ] `apply_diff_backgrounds()` nastaví background barvy na správné sections podle diff mapy
- [ ] Unit testy pro všechny nové funkce prochází
- [ ] Existující testy nezlomeny

## Verification

- `cargo check` — kompilace prochází (nové fieldy zatím nepoužité v renderingu, to je OK)
- `cargo test -p polycredo_editor -- history` — všechny testy (staré i nové) prochází
- `./check.sh` — clippy čistý

## Inputs

- `src/app/ui/workspace/history/mod.rs` — existující `HistoryViewState`, `DiffLine`, `DiffColors`, `compute_diff()`, testy
- S01-RESEARCH.md — detailní technický design pro datové struktury a funkce

## Expected Output

- `src/app/ui/workspace/history/mod.rs` — rozšířen o `ScrollSource`, rozšířený `HistoryViewState`, `HistorySplitResult`, `PanelTexts`, `build_panel_texts()`, `compute_line_offsets()`, `apply_diff_backgrounds()`, nové unit testy
