# S01: Vylepšený search dialog s regex engine, zvýrazněnými výsledky a kontextem

**Goal:** Přestavět project search z minimálního substring dialogu na plnohodnotný vyhledávací nástroj s regex engine, case/whole-word togglery, file type filtrem, zvýrazněnými matchemi v kontextu ±2 řádků a inkrementálním streamováním výsledků.
**Demo:** Ctrl+Shift+F → dialog s togglery → zadání `fn\s+\w+` s regex ON → zvýrazněné výsledky v kontextu → kliknutí otevře soubor na řádku. Case-sensitive/whole-word/file filter fungují správně.

## Must-Haves

- `build_regex()` funkce sestaví regex z query + SearchOptions (regex/case/whole-word toggle kombinace)
- Nevalidní regex pattern zobrazí inline error v dialogu, search se nespustí
- Výsledky obsahují match_ranges pro zvýraznění a context_before/context_after
- Blízké matche v jednom souboru (≤4 řádky) se slučují do jednoho kontextového bloku
- File type filtr přes globset omezuje search scope
- LayoutJob zvýraznění matchů ve výsledcích (barevný background na matchující text)
- Kliknutí na výsledek otevře soubor a skočí na řádek
- Inkrementální streamování výsledků (dávkování per-soubor)
- Unit testy pro engine logiku (regex, case, whole-word, filtr, kontext sloučení)
- i18n klíče pro togglery, error messages, nové UI prvky (5 jazyků)

## Proof Level

- This slice proves: integration
- Real runtime required: yes (egui rendering, kliknutí na výsledek otevře soubor)
- Human/UAT required: no (headless — vizuální UAT deferred)

## Verification

- `cargo test --bin polycredo-editor app::ui::search_picker` — engine unit testy (regex, case, whole-word, filtr, kontext)
- `cargo check` — kompilace čistá
- `./check.sh` — fmt, clippy, všechny testy pass

## Observability / Diagnostics

- Runtime signals: regex error message v `ProjectSearch.regex_error: Option<String>`, search progres přes `try_recv()` batch count
- Inspection surfaces: `ProjectSearch` struct fieldy (toggle stavy, query, results count, regex_error)
- Failure visibility: inline regex error v dialogu, I/O chyby z file reading logované přes `eprintln!`
- Redaction constraints: none

## Integration Closure

- Upstream surfaces consumed: `CommandId::ProjectSearch` dispatch z M004, `ProjectIndex::get_files()`, `LocalHistory::take_snapshot()` (ne v tomto slici), existující `open_file_in_ws()` + `jump_to_location()`
- New wiring introduced in this slice: nový `run_project_search()` s regex engine, nový UI dialog s togglery, nový výsledkový rendering s LayoutJob
- What remains before the milestone is truly usable end-to-end: replace flow (S02)

## Tasks

- [ ] **T01: Search engine s togglery, file filtrem a kontextovými řádky** `est:45m`
  - Why: Engine logika je základ pro celý search — regex matching, toggle kombinace, kontext extraction, file filtering. Plně testovatelná bez UI.
  - Files: `src/app/ui/search_picker.rs`, `src/app/ui/workspace/state/types.rs`
  - Do: (1) Rozšířit `SearchResult` o `match_ranges: Vec<(usize, usize)>` a `context_before: Vec<String>`, `context_after: Vec<String>`. (2) Přidat `SearchOptions` struct (`use_regex`, `case_sensitive`, `whole_word`, `file_filter: String`). (3) Rozšířit `ProjectSearch` o `SearchOptions`, `regex_error: Option<String>`, `replace_text: String`, `show_replace: bool`. (4) Implementovat `build_regex(query: &str, opts: &SearchOptions) -> Result<Regex, String>` — regex mode: Regex::new(query), plain mode: regex::escape(query), whole-word: `\b` wrapping, case: RegexBuilder::case_insensitive(). (5) Implementovat `search_file_with_context(path, regex, context_lines: usize) -> Vec<SearchResult>` — find_iter na každém řádku, context ±2, sloučení blízkých bloků (distance ≤ 2*context). (6) File filter přes `globset::Glob::new(filter).compile_matcher()` v search threadu. (7) Změnit `run_project_search()` na streamování — `mpsc::Sender<SearchBatch>` kde `SearchBatch` je enum `Results(Vec<SearchResult>)` | `Done` | `Error(String)`. Posílat výsledky per-soubor. (8) Unit testy: build_regex kombinace (8 testů), search_file_with_context s kontextem a sloučením (3 testy), file filter (2 testy).
  - Verify: `cargo test --bin polycredo-editor app::ui::search_picker` — všechny engine testy pass. `cargo check` čistý.
  - Done when: build_regex() zvládá všechny toggle kombinace, search_file_with_context() vrací match_ranges + kontext se sloučením, file filter funguje, streamování přes SearchBatch, 13+ unit testů pass.

- [ ] **T02: Vylepšený UI dialog s togglery, zvýrazněnými výsledky a i18n** `est:50m`
  - Why: Engine z T01 potřebuje UI — toggle ikonky, zvýrazněné výsledky s LayoutJob, kontextové řádky, klikací výsledky, inline regex error, inkrementální akumulace výsledků, i18n.
  - Files: `src/app/ui/search_picker.rs`, `src/app/ui/workspace/mod.rs`, `locales/cs/ui.ftl`, `locales/en/ui.ftl`, `locales/sk/ui.ftl`, `locales/de/ui.ftl`, `locales/ru/ui.ftl`
  - Do: (1) Přestavit `render_project_search_dialog()` — input pole + toggle buttons (selectable_label: ".*" regex, "Aa" case, "W" whole-word) + file filter input + inline regex error pod inputem červeně. (2) Validace regex při změně query/togglerů — `build_regex()`, error do `regex_error`, search se spustí jen pokud Ok. (3) Přestavit `poll_and_render_project_search_results()` — akumulace dávek z `try_recv()` loop, loading indikátor dokud `Done` nepřijde. (4) Výsledkový rendering: per-soubor sekce (filename heading), per-match řádek s LayoutJob — matchující text zvýrazněný background barvou (žlutá/oranžová), kontext řádky s tlumenou barvou. Klikací přes `ui.add(Label::new(job).sense(Sense::click()))` nebo `Button::new(job)`. (5) Kliknutí na výsledek → `open_file_in_ws()` + `jump_to_location()` (existující pattern). (6) i18n: ~12 nových klíčů × 5 jazyků — `project-search-regex-toggle`, `project-search-case-toggle`, `project-search-word-toggle`, `project-search-file-filter`, `project-search-file-filter-hint`, `project-search-regex-error`, `project-search-searching`, `project-search-results-count`, `project-search-replace-heading`, `project-search-replace-btn`, `project-search-replace-with`, `project-search-context-separator`. (7) Smazat `collect_project_files()` — nahrazeno `ProjectIndex::get_files()` (už se používá v dispatch kódu).
  - Verify: `cargo check` čistý. `./check.sh` pass. Manuální ověření: grep -c 'project-search-' locales/*/ui.ftl → minimálně 19 klíčů per jazyk (7 existujících + 12 nových).
  - Done when: Dialog zobrazuje togglery a file filter. Regex error je inline. Výsledky mají zvýrazněné matche s kontextem. Kliknutí otevře soubor. i18n kompletní. `./check.sh` pass.

## Files Likely Touched

- `src/app/ui/search_picker.rs` — kompletní přestavba (engine + UI)
- `src/app/ui/workspace/state/types.rs` — rozšíření SearchResult, ProjectSearch, nový SearchOptions
- `src/app/ui/workspace/mod.rs` — dispatch úpravy pro nový search flow
- `locales/cs/ui.ftl` — nové i18n klíče
- `locales/en/ui.ftl` — nové i18n klíče
- `locales/sk/ui.ftl` — nové i18n klíče
- `locales/de/ui.ftl` — nové i18n klíče
- `locales/ru/ui.ftl` — nové i18n klíče
