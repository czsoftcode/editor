---
id: S01
parent: M005
milestone: M005
provides:
  - Search engine s regex/case/whole-word togglery a kontextovými řádky se sloučením
  - Rozšířené datové typy (SearchResult s match_ranges/context, SearchOptions, SearchBatch enum)
  - File type filtr přes globset (filename i cesta)
  - Inkrementální streamování výsledků per-soubor přes mpsc SearchBatch
  - UI dialog s toggle buttons (regex/case/whole-word), file filter inputem, inline regex error
  - LayoutJob zvýraznění matchů (oranžový bg) + kontextové řádky (dim barva)
  - Klikací výsledky otevírající soubor na řádku (open_file_in_ws + jump_to_location)
  - i18n 21 klíčů s prefixem project-search-* × 5 jazyků
requires:
  - slice: none
    provides: none
affects:
  - S02
key_files:
  - src/app/ui/search_picker.rs
  - src/app/ui/workspace/state/types.rs
  - src/app/ui/workspace/state/mod.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - "build_regex() jako centrální factory — plain mode escapuje, regex mode přímé Regex::new, whole-word \b wrapping, case přes RegexBuilder"
  - "SearchBatch enum (Results/Done/Error) pro typed streamování per-soubor přes mpsc"
  - "Kontext sloučení: blízké matche (distance ≤ 2*context_lines) sloučeny do jednoho bloku, ale každý match = vlastní SearchResult"
  - "File filter glob matchuje filename i celou relativní cestu"
  - "LayoutJob multi-section: prefix (šedý line num) + text + zvýrazněný match (oranžový bg rgba(200,130,0,120))"
  - "Label::new(job).sense(Sense::click()) pro klikací výsledky — ověřený pattern z terminal/bottom/mod.rs"
  - "SearchBatch::Error → toast (runtime chyba), regex_error → inline v dialogu (UI validace)"
  - "collect_project_files() ponecháno — živá závislost v ProjectIndex::full_rescan()"
patterns_established:
  - "build_regex(query, opts) → Result<Regex, String> — centrální regex factory"
  - "SearchBatch enum (Results/Done/Error) pro typed streamování"
  - "search_file_with_context() jako pure funkce (path+regex+context → Vec<SearchResult>)"
  - "build_match_layout_job() a build_context_layout_job() — reusable LayoutJob buildery"
  - "start_project_search() — extrahovaná helper pro spuštění searche (reusable z togglerů i Enter)"
observability_surfaces:
  - "ProjectSearch.regex_error: Option<String> — regex kompilační chyba inline v dialogu"
  - "ProjectSearch.searching: bool — indikátor probíhajícího vyhledávání (spinner + text)"
  - "SearchBatch::Error(String) → toast — propaguje I/O/runtime chyby ze search threadu"
  - "eprintln!() pro I/O chyby při čtení souborů v search threadu"
  - "Počet výsledků v titulku results dialogu"
drill_down_paths:
  - .gsd/milestones/M005/slices/S01/tasks/T01-SUMMARY.md
  - .gsd/milestones/M005/slices/S01/tasks/T02-SUMMARY.md
duration: 55m
verification_result: passed
completed_at: 2026-03-13
---

# S01: Vylepšený search dialog s regex engine, zvýrazněnými výsledky a kontextem

**Kompletní přestavba project search — regex engine s case/whole-word togglery, file type filtr, zvýrazněné matche v kontextu ±2 řádků, klikací výsledky, inkrementální streamování, i18n.**

## What Happened

**T01 (engine):** Rozšířen `SearchResult` o `match_ranges`, `context_before`, `context_after`. Nový `SearchOptions` struct (use_regex, case_sensitive, whole_word, file_filter). `SearchBatch` enum (Results/Done/Error) pro typed streamování. Implementován `build_regex()` — escapuje plain text, obalí `\b` pro whole-word, nastaví case přes RegexBuilder. `search_file_with_context()` — find_iter na řádcích, kontext ±2, sloučení blízkých matchů. File filter přes globset. Refaktorovaný `run_project_search()` s per-soubor dávkováním a cancel epoch.

**T02 (UI + i18n):** Přestavěný input dialog s 3 toggle buttons (`.*` regex, `Aa` case, `W` whole-word) jako selectable_label + file filter input + inline regex error červeně. Validace při Enter i změně togglerů. Výsledkový dialog s inkrementální akumulací, spinnerem, per-soubor sekcemi. LayoutJob zvýraznění matchů (oranžový bg) + kontextové řádky (dim barva). Klikací výsledky přes `Label::new(job).sense(Sense::click())` → `open_file_in_ws()` + `jump_to_location()`. 21 i18n klíčů × 5 jazyků.

## Verification

- `cargo test --bin polycredo-editor app::ui::search_picker` — **15 testů pass** (10 build_regex, 3 search_file_with_context, 2 file_filter)
- `cargo check` — kompilace čistá ✓
- `./check.sh` — fmt, clippy, všechny testy pass ✓
- `grep -c 'project-search-' locales/*/ui.ftl` — **21 per jazyk** (> 19 požadovaných) ✓
- Diagnostika: `build_regex_invalid_pattern` ověřuje Err(String) s prefixem "Neplatný regex:" ✓
- Failure-path: `build_regex_empty_query` ověřuje Err pro prázdný dotaz ✓

## Requirements Advanced

- R016 (Regex search engine s togglery) — build_regex() zvládá všechny 8 kombinací regex/case/whole-word, nevalidní regex vrací inline chybu
- R017 (Zvýrazněné matchující části) — LayoutJob multi-section s oranžovým bg na match ranges
- R018 (Kontextové řádky se sloučením) — ±2 řádky kontextu, blízké matche (≤4 řádky) sloučeny do jednoho bloku
- R019 (File type filtr) — globset filtr na filename i cestu
- R021 (Regex error inline v dialogu) — regex_error zobrazený červeně pod inputem, search se nespustí
- R024 (i18n pro nové UI prvky) — 21 klíčů × 5 jazyků (cs/en/sk/de/ru)
- R025 (Inkrementální streamování) — SearchBatch per-soubor dávkování přes mpsc

## Requirements Validated

- R016 — 10 unit testů pokrývají všech 8 build_regex kombinací + empty query + invalid pattern
- R017 — LayoutJob rendering implementován s build_match_layout_job(), cargo check čistý
- R018 — 3 unit testy (simple match, close matches merged, no match), sloučení ověřeno
- R019 — 2 unit testy (glob matches, glob no match), filename i path matching
- R021 — build_regex vrací Err(String) s popisnou zprávou, UI zobrazuje inline červeně, unit test ověřuje
- R024 — 21 klíčů per jazyk ověřeno grepem (partially — S02 doplní replace UI klíče)
- R025 — SearchBatch enum s Results/Done/Error, per-soubor dávkování implementováno

## New Requirements Surfaced

- none

## Requirements Invalidated or Re-scoped

- R024 — partial validation: S01 dodal 21 klíčů per jazyk, S02 doplní replace-specifické klíče

## Deviations

- `collect_project_files()` nebylo smazáno — živá závislost v `ProjectIndex::full_rescan()`, nemůže být odstraněno bez refaktoru indexu
- 15 testů místo plánovaných 13 — 2 extra testy (empty query, invalid regex)
- 11 nových i18n klíčů místo 12 — `project-search-file-filter` vynechán jako redundantní s `project-search-file-filter-hint`

## Known Limitations

- Vizuální UAT deferred (headless prostředí) — LayoutJob rendering, toggle UX, klikací výsledky nelze ověřit automaticky
- Kontextové řádky u velmi blízkých matchů mohou být kosmeticky matoucí (filtrují match řádky z context_before)

## Follow-ups

- S02: Replace flow využije `SearchOptions`, `build_regex()`, `ProjectSearch` struct a existující výsledkový dialog jako základ pro replace preview

## Files Created/Modified

- `src/app/ui/search_picker.rs` — kompletní přestavba (engine funkce + UI dialog + 15 unit testů)
- `src/app/ui/workspace/state/types.rs` — rozšířený SearchResult, nový SearchOptions, SearchBatch, rozšířený ProjectSearch
- `src/app/ui/workspace/state/mod.rs` — reexport SearchBatch, SearchOptions
- `locales/cs/ui.ftl` — 11 nových project-search-* i18n klíčů
- `locales/en/ui.ftl` — 11 nových project-search-* i18n klíčů
- `locales/sk/ui.ftl` — 11 nových project-search-* i18n klíčů
- `locales/de/ui.ftl` — 11 nových project-search-* i18n klíčů
- `locales/ru/ui.ftl` — 11 nových project-search-* i18n klíčů

## Forward Intelligence

### What the next slice should know
- `SearchOptions` a `build_regex()` jsou přímo znovupoužitelné pro replace matching
- `ProjectSearch` struct má `replace_text: String` a `show_replace: bool` fieldy připravené pro S02
- Výsledkový dialog v `poll_and_render_project_search_results()` je výchozí bod pro replace preview UI
- `start_project_search()` helper funkce spouští search a je volatelná i z replace flow

### What's fragile
- `Label::new(job).sense(Sense::click())` — závisí na egui verzi, fallback pattern je `Button::new(job)` nebo `allocate_response() + paint_galley()`
- Kontext sloučení u velmi blízkých matchů — context_before filtruje match řádky, ale vizuálně to může být matoucí

### Authoritative diagnostics
- `ProjectSearch.regex_error` — pokud je `Some(msg)`, regex kompilace selhala, search se nesmí spustit
- `ProjectSearch.searching` — `true` = search thread běží, `false` = Done/Error/Disconnect přijat
- 15 unit testů v `app::ui::search_picker::tests` — autoritativní zdroj pravdy pro engine logiku

### What assumptions changed
- `collect_project_files()` mělo být smazáno — žije dál, protože `ProjectIndex::full_rescan()` ho volá
- Plánováno 12 nových i18n klíčů — dodáno 11 (file-filter je redundantní s file-filter-hint)
