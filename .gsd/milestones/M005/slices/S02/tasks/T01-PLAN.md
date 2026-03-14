---
estimated_steps: 5
estimated_files: 2
---

# T01: Replace engine a preview data model

**Slice:** S02 — Project-wide replace s preview a local history
**Milestone:** M005

## Description

Implementace replace logiky: compute nahrazení pro všechny matchující soubory, generování preview dat (original vs new content), a standalone funkce pro atomický zápis nahrazení. Plně testovatelné bez UI.

## Steps

1. Přidat `ReplacePreview` struct do `types.rs` — `file: PathBuf`, `original_content: String`, `new_content: String`, `match_count: usize`, `selected: bool` (default `true`).
2. Rozšířit `ProjectSearch` o `replace_previews: Vec<ReplacePreview>` a `show_replace_preview: bool`.
3. Implementovat `compute_replace_previews(results: &[SearchResult], regex: &Regex, replace_text: &str) -> io::Result<Vec<ReplacePreview>>` — seskupit výsledky per-soubor (deduplikovat cesty), pro každý soubor: načíst obsah z disku, `regex.replace_all(&content, replace_text)`, pokud se obsah liší → přidat ReplacePreview s match_count. Capture groups ($1, $2) fungují automaticky.
4. Implementovat `apply_replacements(previews: &[ReplacePreview]) -> Vec<(PathBuf, Result<(), String>)>` — pro každý preview kde `selected == true`: `fs::write(&preview.file, &preview.new_content)`. Vrátit vektor výsledků per-file.
5. Unit testy: `test_compute_replace_previews_basic` (tempdir se 2 soubory, replace "foo" → "bar", ověřit new_content), `test_compute_replace_previews_regex_capture` (regex s capture group, replace s $1), `test_apply_replacements_success` (tempdir, apply, ověřit obsah souboru), `test_apply_replacements_partial_skip` (jeden preview s selected=false, ověřit že soubor nezměněn).

## Must-Haves

- [ ] `ReplacePreview` struct s original/new content a selected flag
- [ ] `compute_replace_previews()` generuje správná preview data včetně capture groups
- [ ] `apply_replacements()` zapisuje jen vybrané soubory
- [ ] 4 unit testy pass

## Verification

- `cargo test --bin polycredo-editor app::ui::search_picker` — replace testy pass
- `cargo check` — kompilace čistá

## Observability Impact

- `compute_replace_previews()` vrací `io::Result<Vec<ReplacePreview>>` — I/O chyby (neexistující/nečitelný soubor) se propagují nahoru, caller je může zobrazit přes toast.
- `apply_replacements()` vrací `Vec<(PathBuf, Result<(), String>)>` — per-file výsledek. Agent/UI může inspektovat vektor a identifikovat přesně které soubory selhaly a s jakou zprávou.
- `ReplacePreview.selected` flag — umožňuje per-file kontrolu před zápisem. Inspektovatelné přes debug/log.
- Selhání se netiší: žádný `unwrap()` na I/O, žádný tichý skip.

## Inputs

- S01 output: `SearchResult` s match_ranges, `build_regex()`, `SearchOptions`
- `src/app/ui/workspace/state/types.rs` — rozšířený ProjectSearch struct

## Expected Output

- `src/app/ui/workspace/state/types.rs` — `ReplacePreview` struct, rozšířený `ProjectSearch`
- `src/app/ui/search_picker.rs` — `compute_replace_previews()`, `apply_replacements()`, 4 unit testy
