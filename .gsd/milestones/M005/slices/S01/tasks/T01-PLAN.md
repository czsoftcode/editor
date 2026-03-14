---
estimated_steps: 8
estimated_files: 2
---

# T01: Search engine s togglery, file filtrem a kontextovými řádky

**Slice:** S01 — Vylepšený search dialog s regex engine, zvýrazněnými výsledky a kontextem
**Milestone:** M005

## Description

Implementace čisté search engine logiky bez UI: regex matching s toggle kombinacemi (regex/case/whole-word), file type filtrování přes globset, kontextové řádky ±2 se sloučením blízkých matchů, a streamování výsledků přes mpsc kanál. Rozšíření datových typů (`SearchResult`, `ProjectSearch`) pro podporu nových features. Plně pokryto unit testy.

## Steps

1. Rozšířit `SearchResult` v `types.rs` — přidat `match_ranges: Vec<(usize, usize)>` (byte rozsahy matchů v textu řádku), `context_before: Vec<String>`, `context_after: Vec<String>`.
2. Přidat `SearchOptions` struct do `types.rs` — `use_regex: bool`, `case_sensitive: bool`, `whole_word: bool`, `file_filter: String`.
3. Rozšířit `ProjectSearch` v `types.rs` — přidat `options: SearchOptions`, `regex_error: Option<String>`, `replace_text: String`, `show_replace: bool`, `searching: bool`.
4. Implementovat `build_regex(query: &str, opts: &SearchOptions) -> Result<Regex, String>` v `search_picker.rs` — regex mode: `Regex::new(query)`, plain mode: `regex::escape(query)` + `Regex::new()`, whole-word: `\b` prefix/suffix, case: `RegexBuilder::new().case_insensitive(!opts.case_sensitive).build()`. Error mapování na `String`.
5. Implementovat `search_file_with_context(path: &Path, regex: &Regex, context_lines: usize) -> io::Result<Vec<SearchResult>>` — načíst soubor, iterovat řádky, `regex.find_iter()` na každém řádku, sbírat match_ranges, přidat context ±context_lines. Sloučit blízké matche (vzdálenost ≤ 2*context_lines) do jednoho bloku s rozšířeným kontextem.
6. Přidat `SearchBatch` enum — `Results(Vec<SearchResult>)`, `Done`, `Error(String)`. Změnit `run_project_search()` na posílání `SearchBatch` per-soubor přes `mpsc::Sender<SearchBatch>`. File filter přes `globset::Glob::new(filter).compile_matcher()` aplikovat na seznam souborů před iterací. Cancel epoch zachovat.
7. Smazat `collect_project_files()` — nahradit voláním `ProjectIndex::get_files()` (Arc<Vec<PathBuf>>).
8. Unit testy: `build_regex` — 8 testů (plain/regex × case/no-case × word/no-word, nevalidní regex, prázdný query). `search_file_with_context` — 3 testy (jednoduchý match s kontextem, sloučení blízkých matchů, žádný match). File filter — 2 testy (glob matchuje, glob nematchuje).

## Must-Haves

- [ ] `SearchResult` má `match_ranges`, `context_before`, `context_after`
- [ ] `SearchOptions` struct s 4 togglery
- [ ] `build_regex()` zvládá všech 8 toggle kombinací + error handling
- [ ] `search_file_with_context()` vrací kontext ±2 se sloučením
- [ ] `SearchBatch` enum pro streamování (Results/Done/Error)
- [ ] `run_project_search()` posílá dávky per-soubor
- [ ] File filter přes globset
- [ ] 13+ unit testů pass

## Verification

- `cargo test --bin polycredo-editor app::ui::search_picker` — 13+ testů pass
- `cargo check` — kompilace čistá (UI kód se dočasně přizpůsobí novým typům)

## Inputs

- `src/app/ui/search_picker.rs` — existující search implementace (352 řádků)
- `src/app/ui/workspace/state/types.rs` — existující SearchResult a ProjectSearch structs

## Expected Output

- `src/app/ui/workspace/state/types.rs` — rozšířené typy s match_ranges, kontextem, options, SearchBatch
- `src/app/ui/search_picker.rs` — nový engine (build_regex, search_file_with_context, refaktorovaný run_project_search), 13+ unit testů
