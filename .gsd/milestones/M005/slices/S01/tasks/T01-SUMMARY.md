---
id: T01
parent: S01
milestone: M005
provides:
  - Search engine s regex/case/whole-word togglery a kontextovými řádky
  - Rozšířené datové typy (SearchResult, SearchOptions, SearchBatch, ProjectSearch)
  - File filter přes globset
  - Inkrementální streamování výsledků per-soubor přes SearchBatch enum
key_files:
  - src/app/ui/search_picker.rs
  - src/app/ui/workspace/state/types.rs
  - src/app/ui/workspace/state/mod.rs
key_decisions:
  - Kontext sloučení: blízké matche (vzdálenost ≤ 2*context_lines) se slučují do jednoho bloku, ale každý match generuje vlastní SearchResult s přizpůsobeným kontextem
  - collect_project_files() ponecháno — stále se volá z ProjectIndex, mazání přesunuto na T02
  - File filter matchuje jak filename tak celou relativní cestu
patterns_established:
  - build_regex() jako centrální factory pro regex z query + SearchOptions
  - SearchBatch enum (Results/Done/Error) pro typed streamování přes mpsc
  - search_file_with_context() jako pure funkce — čte soubor, vrací Vec<SearchResult>
observability_surfaces:
  - ProjectSearch.regex_error: Option<String> — uchovává regex kompilační chybu
  - ProjectSearch.searching: bool — indikátor probíhajícího vyhledávání
  - SearchBatch::Error(String) — propaguje I/O/regex chyby ze search threadu
  - eprintln!() pro I/O chyby při čtení souborů v search threadu
duration: 25m
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: Search engine s togglery, file filtrem a kontextovými řádky

**Implementován kompletní search engine s regex/case/whole-word togglery, file filtrem přes globset, kontextovými řádky ±2 se sloučením blízkých matchů a inkrementálním streamováním výsledků. 15 unit testů pass.**

## What Happened

1. Rozšířen `SearchResult` o `match_ranges: Vec<(usize, usize)>`, `context_before: Vec<String>`, `context_after: Vec<String>`.
2. Přidán `SearchOptions` struct (use_regex, case_sensitive, whole_word, file_filter) s `#[derive(Default)]`.
3. Přidán `SearchBatch` enum (Results/Done/Error) pro typed streamování.
4. Rozšířen `ProjectSearch` o `options`, `regex_error`, `replace_text`, `show_replace`, `searching`.
5. Implementován `build_regex()` — escapuje plain text, obalí `\b` pro whole-word, nastaví case_insensitive přes RegexBuilder. Vrací `Err(String)` s popisnou chybou.
6. Implementován `search_file_with_context()` — find_iter na každém řádku, sbírá match_ranges, generuje kontext ±N řádků. Blízké matche (vzdálenost ≤ 2*context) se slučují do jednoho kontextového bloku.
7. Refaktorován `run_project_search()` — přijímá `SearchOptions`, posílá `SearchBatch` per-soubor, aplikuje globset file filter na filename i cestu, zachovává cancel epoch.
8. UI kód dočasně přizpůsoben: regex validace před spuštěním searche, akumulace SearchBatch dávek v poll smyčce.
9. Reexportovány nové typy (SearchBatch, SearchOptions) z `state/mod.rs`.

## Verification

- `cargo test --bin polycredo-editor app::ui::search_picker` — **15 testů pass** (10 build_regex, 3 search_file_with_context, 2 file_filter)
- `cargo check` — kompilace čistá
- `./check.sh` — **187 testů pass**, fmt ok, clippy ok

### Slice-level verification status (T01/T02):
- ✅ `cargo test --bin polycredo-editor app::ui::search_picker` — 15 engine testů pass
- ✅ `cargo check` — čistá kompilace
- ✅ `./check.sh` — fmt, clippy, všechny testy pass
- ✅ Diagnostika: test `build_regex_invalid_pattern` ověřuje Err(String) s popisnou chybou

## Diagnostics

- `ProjectSearch.regex_error` — `Some(msg)` při nevalidním regexu, `None` jinak. Chybová hláška začíná prefixem "Neplatný regex:".
- `ProjectSearch.searching` — `true` během běžícího searche, `false` po Done/Error/Disconnect.
- `eprintln!("Search: chyba čtení ...")` — loguje I/O chyby z file reading v search threadu.
- Build_regex empty query vrací specifickou chybu "Prázdný vyhledávací dotaz".

## Deviations

- `collect_project_files()` ponecháno v souboru — stále se volá z `ProjectIndex::new()` v `index.rs`. Mazání přesunuto na T02 kde se řeší dispatch.
- 15 testů místo požadovaných 13 — přidány 2 extra testy (10 build_regex místo 8, plus empty query a invalid regex).

## Known Issues

- Kontext při sloučených blocích: context_before prvního matche v bloku zahrnuje řádky od ctx_start po line_idx, ale filtruje ostatní match řádky z bloku. Může být matoucí u velmi blízkých matchů — kosmetická záležitost pro T02 UI rendering.

## Files Created/Modified

- `src/app/ui/workspace/state/types.rs` — rozšířené SearchResult, nový SearchOptions, SearchBatch, rozšířený ProjectSearch
- `src/app/ui/workspace/state/mod.rs` — reexport SearchBatch, SearchOptions
- `src/app/ui/search_picker.rs` — nové funkce build_regex(), search_file_with_context(), refaktorovaný run_project_search(), 15 unit testů
- `.gsd/milestones/M005/slices/S01/S01-PLAN.md` — přidán diagnostický verifikační krok
- `.gsd/milestones/M005/slices/S01/tasks/T01-PLAN.md` — přidána Observability Impact sekce
