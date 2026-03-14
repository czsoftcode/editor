# S02: Project-wide replace s preview a local history

**Goal:** Přidat project-wide find & replace s preview dialogem zobrazujícím diff per-file s checkboxy, local history snapshotem před každou modifikací, a per-file error reportingem.
**Demo:** V search dialogu zadat query "old_name" → zadat replace text "new_name" → kliknout Replace → preview dialog zobrazí diff všech souborů s checkboxy → odškrtnout jeden soubor → potvrdit → soubory se změní (kromě odškrtnutého) → local history snapshot existuje pro každý modifikovaný soubor.

## Must-Haves

- Replace input pole v search dialogu (toggle zobrazení)
- Replace preview dialog s per-file sekcemi zobrazujícími starý → nový text
- Checkboxy per-file pro výběr/odmítnutí nahrazení
- Local history snapshot (`take_snapshot()`) před zápisem každého souboru
- Per-file error handling — selhání jednoho souboru nesmí zastavit replace, chyba přes toast
- Pokud snapshot selže (disk full), nespouštět zápis daného souboru a reportovat
- Regex capture groups ($1, $2) fungují automaticky přes `Regex::replace_all()`
- i18n klíče pro replace UI (5 jazyků) — doplnění k S01 klíčům
- Unit testy pro replace logiku (apply_replacements)

## Proof Level

- This slice proves: integration
- Real runtime required: yes (replace modifikuje soubory, snapshot přes LocalHistory)
- Human/UAT required: no (headless — UAT deferred)

## Verification

- `cargo test --bin polycredo-editor app::ui::search_picker` — replace unit testy pass
- `cargo check` — kompilace čistá
- `./check.sh` — fmt, clippy, všechny testy pass
- `apply_replacements()` s neexistující cestou vrací `Err(String)` per-file — ověřit unit testem že chybný soubor neblokuje ostatní a výsledek obsahuje chybovou hlášku
- Diagnostic: `ProjectSearch` replace-related fieldy (`pending_replace`, `replace_previews`, `show_replace_preview`) inspektovatelné po replace akci — `pending_replace` se resetuje po dokončení, `replace_previews` obsahuje per-file stav

## Observability / Diagnostics

- Runtime signals: toast pro replace chyby (per-file), replace summary toast ("Nahrazeno v N souborech")
- Inspection surfaces: `ProjectSearch` replace-related fieldy (replace_text, show_replace, replace_results)
- Failure visibility: per-file toast s názvem souboru a chybovou zprávou, snapshot selhání blokuje zápis
- Redaction constraints: none

## Integration Closure

- Upstream surfaces consumed: `build_regex()` a `SearchOptions` ze S01, `ProjectSearch` struct, `SearchResult` s match_ranges, `take_snapshot()` z LocalHistory, `open_file_in_ws()`, Toast API
- New wiring introduced in this slice: replace preview dialog, replace execution flow v workspace handleru (snapshot → write → toast)
- What remains before the milestone is truly usable end-to-end: nothing — S02 completion = milestone complete

## Tasks

- [x] **T01: Replace engine a preview data model** `est:30m`
  - Why: Replace logika (compute nahrazení, preview data) musí existovat dřív než UI. Testovatelné bez UI.
  - Files: `src/app/ui/search_picker.rs`, `src/app/ui/workspace/state/types.rs`
  - Do: (1) Přidat `ReplacePreview` struct — `file: PathBuf`, `original_content: String`, `new_content: String`, `match_count: usize`, `selected: bool` (default true). (2) Rozšířit `ProjectSearch` o `replace_previews: Vec<ReplacePreview>`, `show_replace_preview: bool`. (3) Implementovat `compute_replace_previews(results: &[SearchResult], regex: &Regex, replace_text: &str) -> Vec<ReplacePreview>` — seskupit výsledky per-file, načíst obsah souboru, `regex.replace_all()`, porovnat original vs new. (4) Implementovat `apply_replacements(previews: &[ReplacePreview]) -> Vec<(PathBuf, Result<(), String>)>` — standalone funkce pro zápis `new_content` do souborů kde `selected == true`. Vrací výsledek per-file. (5) Unit testy: compute_replace_previews (2 testy), apply_replacements s tempdir (2 testy).
  - Verify: `cargo test --bin polycredo-editor app::ui::search_picker` — replace testy pass
  - Done when: compute_replace_previews() generuje správné preview data, apply_replacements() zapisuje soubory, 4 unit testy pass.

- [x] **T02: Replace UI, preview dialog, snapshot wiring a i18n** `est:40m`
  - Why: Replace engine z T01 potřebuje UI — replace input, preview dialog s diff a checkboxy, snapshot volání v workspace handleru, error toasty, i18n.
  - Files: `src/app/ui/search_picker.rs`, `src/app/ui/workspace/mod.rs`, `locales/cs/ui.ftl`, `locales/en/ui.ftl`, `locales/sk/ui.ftl`, `locales/de/ui.ftl`, `locales/ru/ui.ftl`
  - Do: (1) V search dialogu: replace toggle button → zobrazí replace input pole pod query. "Replace All" button spustí `compute_replace_previews()` a otevře preview dialog. (2) Replace preview dialog: per-file collapsible sekce s checkboxem. Každá sekce zobrazuje filename + match count. Uvnitř: starý text (červená) → nový text (zelená) diff přes LayoutJob (pattern z diff_view.rs). Select all / Deselect all checkboxy nahoře. (3) "Potvrdit" button: zavře preview → v workspace handleru: pro každý vybraný soubor: `take_snapshot(relative_path, original_content)` → `fs::write(path, new_content)`. Pokud snapshot selže → toast, skip soubor. Pokud write selže → toast, pokračovat. (4) Summary toast: "Nahrazeno v {N} souborech" nebo "Nahrazeno v {N} z {M} souborů ({K} chyb)". (5) Po replace: refresh otevřených tabů jejichž soubor byl modifikován (reload content z disku). (6) i18n: doplnit chybějící replace klíče — `project-search-replace-preview-title`, `project-search-replace-confirm`, `project-search-replace-select-all`, `project-search-replace-deselect-all`, `project-search-replace-success`, `project-search-replace-error`, `project-search-replace-snapshot-error` (5 jazyků).
  - Verify: `cargo check` čistý. `./check.sh` pass. `grep -c 'project-search-replace' locales/*/ui.ftl` → ≥7 per jazyk.
  - Done when: Replace flow funguje end-to-end: query → replace text → preview → checkboxy → potvrzení → soubory modifikovány → snapshoty existují → toasty. `./check.sh` pass.

## Files Likely Touched

- `src/app/ui/search_picker.rs` — replace engine + preview UI
- `src/app/ui/workspace/state/types.rs` — ReplacePreview struct, ProjectSearch rozšíření
- `src/app/ui/workspace/mod.rs` — replace execution flow (snapshot + write + tab refresh)
- `locales/cs/ui.ftl` — replace i18n klíče
- `locales/en/ui.ftl` — replace i18n klíče
- `locales/sk/ui.ftl` — replace i18n klíče
- `locales/de/ui.ftl` — replace i18n klíče
- `locales/ru/ui.ftl` — replace i18n klíče
