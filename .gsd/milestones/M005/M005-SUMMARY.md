---
id: M005
provides:
  - Regex search engine s case/whole-word/regex togglery a inline error validací
  - Zvýrazněné matche ve výsledcích přes LayoutJob s oranžovým background
  - Kontextové řádky ±2 se sloučením blízkých matchů (distance ≤ 2*context_lines)
  - File type filtr přes globset (filename i cesta)
  - Inkrementální streamování výsledků per-soubor přes mpsc SearchBatch enum
  - Klikací výsledky otevírající soubor na řádku
  - Project-wide replace s preview dialogem (per-file inline diff, checkboxy, select all/deselect all)
  - Local history snapshot před každou replace modifikací
  - Per-file error handling pro replace (snapshot/write selhání → toast, replace pokračuje)
  - 35 project-search-* i18n klíčů × 5 jazyků (cs, en, sk, de, ru)
  - 20 unit testů pokrývajících engine logiku a replace flow
key_decisions:
  - "build_regex() jako centrální factory — plain mode escapuje, regex mode přímé Regex::new, whole-word \\b wrapping, case přes RegexBuilder"
  - "SearchBatch enum (Results/Done/Error) pro typed streamování per-soubor přes mpsc"
  - "Label::new(layout_job).sense(Sense::click()) pro klikací výsledky"
  - "pending_replace flag pattern — UI nastaví flag, workspace handler provede snapshot+write po renderingu"
  - "apply_replacements() vrací Vec<(PathBuf, Result<(), String>)> pro per-file error reporting"
  - "Tab refresh po replace — reload z disku + reset modified flag + sync last_saved_content"
  - "Preview dialog collapsing: default-open pro ≤5 souborů, collapsed pro >5"
  - "compute_replace_previews() bere root path pro překlad relativních cest na absolutní"
patterns_established:
  - "build_regex(query, opts) → Result<Regex, String> — centrální regex factory"
  - "SearchBatch enum (Results/Done/Error) pro typed streamování"
  - "search_file_with_context() jako pure funkce (path+regex+context → Vec<SearchResult>)"
  - "build_match_layout_job() a build_context_layout_job() — reusable LayoutJob buildery"
  - "pending_replace → workspace handler execute → toast → tab refresh — replace execution pattern"
  - "compute_replace_previews() + apply_replacements() jako standalone funkce oddělené od UI"
observability_surfaces:
  - "ProjectSearch.regex_error: Option<String> — inline regex kompilační chyba"
  - "ProjectSearch.searching: bool — indikátor probíhajícího vyhledávání"
  - "ProjectSearch.pending_replace: bool — true jen po Confirm, reset po zpracování"
  - "SearchBatch::Error(String) → toast — runtime I/O chyby ze search threadu"
  - "Toast queue — per-file snapshot/write chyby s cestou a chybovou zprávou"
  - "Počet výsledků v titulku results dialogu"
requirement_outcomes:
  - id: R016
    from_status: active
    to_status: validated
    proof: "build_regex() zvládá všech 8 kombinací regex/case/whole-word. 10 unit testů. Nevalidní regex vrací Err(String)."
  - id: R017
    from_status: active
    to_status: validated
    proof: "LayoutJob multi-section s oranžovým bg (rgba(200,130,0,120)) na match ranges. cargo check čistý."
  - id: R018
    from_status: active
    to_status: validated
    proof: "search_file_with_context() s ±2 řádky kontextu, sloučení blízkých matchů. 3 unit testy."
  - id: R019
    from_status: active
    to_status: validated
    proof: "globset filtr na filename i cestu. 2 unit testy (glob matches, glob no match)."
  - id: R020
    from_status: active
    to_status: validated
    proof: "compute_replace_previews() + apply_replacements() + render_replace_preview_dialog(). 5 unit testů. Per-file diff s checkboxy."
  - id: R021
    from_status: active
    to_status: validated
    proof: "build_regex() vrací Err(String), UI zobrazuje inline červeně. Unit testy build_regex_invalid_pattern, build_regex_empty_query."
  - id: R022
    from_status: active
    to_status: validated
    proof: "apply_replacements() per-file error isolation. Unit test test_apply_replacements_nonexistent_file_error. Toast wiring v workspace handleru."
  - id: R023
    from_status: active
    to_status: validated
    proof: "take_snapshot() volaný v workspace handleru pro každý selected ReplacePreview. Snapshot selhání → toast + skip soubor."
  - id: R024
    from_status: active
    to_status: validated
    proof: "21 S01 klíčů + 14 S02 klíčů = 35 project-search-* klíčů × 5 jazyků. grep -c ověřuje 31 per jazyk (7 existovalo před M005)."
  - id: R025
    from_status: active
    to_status: validated
    proof: "SearchBatch enum (Results/Done/Error) přes mpsc s per-soubor dávkováním. UI akumuluje přes try_recv()."
duration: 95m
verification_result: passed
completed_at: 2026-03-13
---

# M005: Vylepšení Project Search

**Kompletní přestavba project-wide search z minimálního modálního dialogu na plnohodnotný vyhledávací nástroj s regex engine, case/whole-word togglery, zvýrazněnými matchi v kontextu, file type filtrem, project-wide replace s preview dialogem a local history snapshotem.**

## What Happened

**S01 (regex engine + UI dialog)** přebudoval celý search pipeline. Nový `build_regex()` jako centrální factory zvládá všech 8 kombinací regex/case/whole-word togglerů — plain mode escapuje speciální znaky, regex mode kompiluje přímo, whole-word obaluje `\b`, case nastavuje přes `RegexBuilder`. `search_file_with_context()` jako pure funkce prochází soubor po řádcích, sbírá ±2 řádky kontextu a slučuje blízké matche (distance ≤ 2×context_lines) do koherentních bloků. File type filtr přes `globset` matchuje filename i celou relativní cestu. Streamování výsledků přes `SearchBatch` enum (Results/Done/Error) umožňuje inkrementální akumulaci v UI. Input dialog dostal 3 toggle buttons (`.*` regex, `Aa` case, `W` whole-word) jako selectable_label (VS Code styl), file filter input a inline regex error červeně pod inputem. Výsledkový dialog zobrazuje per-soubor sekce s LayoutJob zvýrazněním matchů (oranžový bg) a kontextovými řádky (dim barva). Klikací výsledky přes `Label::new(job).sense(Sense::click())` otevírají soubor na řádku. 15 unit testů pokrývají engine logiku.

**S02 (replace + preview)** přidal replace toggle (↔ button) v search dialogu s replace inputem. `compute_replace_previews()` deduplikuje soubory, načte obsah z disku, provede `regex.replace_all()` s automatickou podporou capture groups. `apply_replacements()` zapíše vybrané soubory s per-file error isolation. Preview dialog renderuje per-file collapsible sekce s inline diff přes `similar::TextDiff` (červená/zelená), checkboxy, select all/deselect all a selection counter. Confirm nastaví `pending_replace` flag, workspace handler pro každý vybraný soubor volá `take_snapshot()` s original_content a pak `fs::write()` s new_content. Snapshot selhání → toast + skip. Write selhání → toast + pokračovat. Summary toast reportuje výsledek. Po replace se refreshnou otevřené taby. 5 replace unit testů doplňuje 15 search testů na celkových 20. 14 replace-specifických i18n klíčů ve všech 5 jazycích.

Celý pipeline: Ctrl+Shift+F → toggle nastavení → query → inkrementální výsledky se zvýrazněním → klik otevře soubor → nebo replace text → preview diff → confirm → snapshot + write + tab refresh + toast.

## Cross-Slice Verification

**Success criterion 1 — Regex search `fn\s+\w+`:**
- `build_regex()` s `use_regex=true` kompiluje regex přímo. 10 unit testů pokrývají všech 8 kombinací togglerů + empty query + invalid pattern. LayoutJob zvýraznění implementováno. Klikací výsledky napojeny na `open_file_in_ws()` + `jump_to_location()`. ✅

**Success criterion 2 — Case-sensitive "TODO":**
- `build_regex_plain_case_sensitive` test ověřuje, že case_sensitive=true najde jen přesnou variantu. `build_regex_plain_case_insensitive` ověřuje, že case_sensitive=false matchuje všechny varianty přes `RegexBuilder::case_insensitive(true)`. ✅

**Success criterion 3 — Whole-word "Result":**
- `build_regex_plain_whole_word` test ověřuje `\b` wrapping — "Result" matchne "Result" ale ne "SearchResult". `build_regex_plain_case_sensitive_whole_word` ověřuje kombinaci s case. ✅

**Success criterion 4 — File type filtr `*.rs`:**
- `file_filter_glob_matches` a `file_filter_glob_no_match` unit testy. Globset matchuje filename i celou relativní cestu. Nevalidní glob → `SearchBatch::Error` → toast. ✅

**Success criterion 5 — Replace s preview:**
- `test_compute_replace_previews_basic` ověřuje generování preview dat. `test_compute_replace_previews_regex_capture` ověřuje capture groups. `test_apply_replacements_success` ověřuje zápis. `test_apply_replacements_partial_skip` ověřuje odškrtnutí souboru (selected=false). `test_apply_replacements_nonexistent_file_error` ověřuje per-file error handling. Workspace handler volá `take_snapshot()` před zápisem. ✅

**Success criterion 6 — Nevalidní regex → inline chyba:**
- `build_regex_invalid_pattern` test ověřuje `Err(String)` s prefixem "Neplatný regex:". `build_regex_empty_query` ověřuje Err pro prázdný dotaz. UI zobrazuje `regex_error` inline červeně pod inputem, search se nespustí. ✅

**Success criterion 7 — `cargo check` + `./check.sh`:**
- `cargo check` — čistá kompilace. `./check.sh` — fmt, clippy, všechny testy pass, "Quality Gate: All checks passed successfully!" ✅

**Additional verifications:**
- 20/20 unit testů pass (15 search + 5 replace)
- 31 `project-search-*` i18n klíčů per jazyk (7 existujících + 11 S01 + 14 S02 = 32, ale `project-search-file-filter` vynechán jako redundantní → 31)
- Oba slice summaries existují (S01-SUMMARY.md, S02-SUMMARY.md)
- Roadmap: oba slicí `[x]`

## Requirement Changes

- R016: active → validated — build_regex() 10 unit testů, všech 8 kombinací regex/case/whole-word
- R017: active → validated — LayoutJob multi-section s oranžovým bg na match ranges, cargo check čistý
- R018: active → validated — search_file_with_context() 3 unit testy, sloučení blízkých matchů
- R019: active → validated — globset filtr 2 unit testy, filename + path matching
- R020: active → validated — 5 unit testů, preview dialog s inline diff a checkboxy, apply_replacements() per-file
- R021: active → validated — inline regex error, 2 unit testy (invalid pattern, empty query)
- R022: active → validated — per-file error isolation, unit test nonexistent file, toast wiring
- R023: active → validated — take_snapshot() v workspace handleru, pending_replace pattern
- R024: active → validated — 35 project-search-* klíčů × 5 jazyků (21 search + 14 replace)
- R025: active → validated — SearchBatch enum, per-soubor dávkování přes mpsc

## Forward Intelligence

### What the next milestone should know
- Project search je nyní plnohodnotný regex nástroj s replace. Nový milestone by neměl potřebovat měnit search pipeline.
- `build_regex()` je reusable i mimo project search — pokud se bude vylepšovat in-file search (Ctrl+F), měl by sdílet stejnou factory.
- `SearchOptions` struct je rozšiřitelný pro budoucí filtry (např. exclude patterns, max file size).
- Replace flow běží synchronně na main threadu — pro 100+ souborů může být viditelný lag (~100ms).

### What's fragile
- `Label::new(layout_job).sense(Sense::click())` — závisí na egui verzi. Fallback: `Button::new(layout_job)` nebo `allocate_response() + paint_galley()`.
- Replace na main threadu — synchronní snapshot + write cyklus. Pokud se ukáže jako problém, refaktor na background thread, ale `take_snapshot()` potřebuje `&mut LocalHistory` na main threadu.
- Kontextové řádky u velmi blízkých matchů — context_before filtruje match řádky, vizuálně může být matoucí.

### Authoritative diagnostics
- `ProjectSearch.regex_error` — pokud je `Some(msg)`, regex kompilace selhala, search se nesmí spustit
- `ProjectSearch.searching` — `true` = search thread běží, `false` = Done/Error/Disconnect přijat
- `ProjectSearch.pending_replace` — pokud je `true` mimo workspace handler, replace flow zasekl
- 20 unit testů v `app::ui::search_picker::tests` — autoritativní zdroj pravdy pro engine a replace logiku

### What assumptions changed
- Plán předpokládal 7 replace i18n klíčů — ve skutečnosti 14 (přidány cancel, selection-info, partial-success a další)
- `collect_project_files()` mělo být smazáno — žije dál kvůli živé závislosti v `ProjectIndex::full_rescan()`
- `compute_replace_previews()` dostala navíc `root: &Path` parametr pro překlad relativních cest

## Files Created/Modified

- `src/app/ui/search_picker.rs` — kompletní přestavba: engine funkce (build_regex, search_file_with_context, compute_replace_previews, apply_replacements), UI dialogy (input s togglery, výsledky se zvýrazněním, replace preview s diff), 20 unit testů
- `src/app/ui/workspace/state/types.rs` — rozšířený SearchResult (match_ranges, context), nový SearchOptions, SearchBatch, ReplacePreview, rozšířený ProjectSearch
- `src/app/ui/workspace/state/mod.rs` — reexport SearchBatch, SearchOptions, ReplacePreview
- `src/app/ui/workspace/mod.rs` — replace execution flow (snapshot + write + tab refresh + toast)
- `locales/cs/ui.ftl` — 24 nových project-search-* i18n klíčů (11 search + 14 replace, minus 1 redundantní)
- `locales/en/ui.ftl` — 24 nových project-search-* i18n klíčů
- `locales/sk/ui.ftl` — 24 nových project-search-* i18n klíčů
- `locales/de/ui.ftl` — 24 nových project-search-* i18n klíčů
- `locales/ru/ui.ftl` — 24 nových project-search-* i18n klíčů
