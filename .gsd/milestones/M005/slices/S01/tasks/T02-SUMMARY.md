---
id: T02
parent: S01
milestone: M005
provides:
  - Vylepšený project search UI dialog s toggle buttony (regex/case/whole-word)
  - File filter input pole
  - Inline regex error zobrazení
  - Zvýrazněné matche přes LayoutJob s oranžovým background
  - Kontextové řádky s tlumenou barvou
  - Klikací výsledky otevírající soubor na řádku
  - Inkrementální akumulace streamovaných výsledků se spinnerem
  - i18n klíče pro 5 jazyků (12 nových klíčů per jazyk)
key_files:
  - src/app/ui/search_picker.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - "collect_project_files() ponecháno — stále živá závislost v ProjectIndex::full_rescan(), nelze smazat"
  - "SearchBatch::Error → toast místo regex_error, protože error v tomto kontextu je runtime chyba ne UI validace"
  - "LayoutJob s multi-section pro match highlighting: prefix (šedý) + normální text + zvýrazněný match (oranžový bg)"
  - "Label::new(job).sense(Sense::click()) pro klikací výsledky — ověřený pattern z terminal/bottom/mod.rs"
  - "Výsledky seskupeny per-soubor s bold heading a separátorem ··· mezi nesouvisejícími bloky"
patterns_established:
  - "build_match_layout_job() a build_context_layout_job() — reusable LayoutJob buildery pro search výsledky"
  - "start_project_search() — extrahovaná helper funkce pro spuštění searche (znovupoužitelná z togglerů i Enter)"
observability_surfaces:
  - "ProjectSearch.regex_error — Some(msg) pro nevalidní regex, None po validním"
  - "ProjectSearch.searching — true během běhu, false po Done/Error/Disconnect"
  - "UI spinner + 'Hledám...' text během searching == true"
  - "Počet výsledků v titulku results dialogu"
  - "SearchBatch::Error → toast zpráva"
duration: 30m
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T02: Vylepšený UI dialog s togglery, zvýrazněnými výsledky a i18n

**Kompletní přestavba project search UI — toggle buttons, file filter, LayoutJob zvýraznění, kontextové řádky, klikací výsledky, i18n.**

## What Happened

1. **Input dialog přestavěn**: Query input + 3 toggle buttons (`.* ` regex, `Aa` case, `W` whole-word) jako `selectable_label` s hover tooltipem. Pod query: file filter input s placeholder hint. Pod tím: inline regex error červeným textem. Togglery automaticky re-spustí search při změně (pokud query neprázdný).

2. **Validace**: Při Enter nebo změně togglerů se volá `build_regex()`. Err → regex_error se zobrazí inline, search se nespustí. Ok → regex_error se vymaže, search se spustí přes novou `start_project_search()` helper funkci.

3. **Výsledkový dialog přestavěn**: Inkrementální akumulace přes `try_recv()` loop. Loading indikátor (spinner + "Hledám...") dokud `searching == true`. `SearchBatch::Error` → toast (ne regex_error — to je runtime chyba). Výsledky seskupeny per-soubor (bold filename heading + separator).

4. **LayoutJob zvýraznění**: Match řádek: prefix s číslem řádku (šedý) + text se zvýrazněnými match ranges (oranžový background `rgba(200,130,0,120)`). Kontextové řádky: prefix + text dim barvou `rgb(140,140,140)`.

5. **Klikací výsledky**: `Label::new(job).sense(Sense::click())` + cursor icon PointingHand na hover. Kliknutí → `open_file_in_ws()` + `jump_to_location()`.

6. **i18n**: 12 nových klíčů ve všech 5 jazycích (cs/en/sk/de/ru). Celkem 21 klíčů s prefixem `project-search-` per jazyk.

7. **Cleanup**: `collect_project_files()` ponecháno — stále se používá v `ProjectIndex::full_rescan()`, nemůže být odstraněno bez refaktoru indexu.

## Verification

- `cargo check` — čistá kompilace ✓
- `./check.sh` — fmt, clippy, všechny testy pass ✓
- `cargo test --bin polycredo-editor app::ui::search_picker` — 15 unit testů pass ✓
- `grep -c 'project-search-' locales/*/ui.ftl` — 21 per jazyk (> 19 požadovaných) ✓
- Diagnostika: `build_regex_invalid_pattern` ověřuje prefix "Neplatný regex:" a neprázdný string ✓
- Failure-path: `build_regex_empty_query` ověřuje Err pro prázdný dotaz ✓

## Diagnostics

- `ProjectSearch.regex_error` — `Some(msg)` při nevalidním regexu, `None` jinak
- `ProjectSearch.searching` — `true` během běžícího searche, `false` po Done/Error/Disconnect
- `ProjectSearch.options` — aktuální stav togglerů (use_regex, case_sensitive, whole_word, file_filter)
- `ProjectSearch.results.len()` — počet akumulovaných výsledků
- UI spinner a "Hledám..." indikátor viditelný při searching == true
- `SearchBatch::Error` → toast zpráva v UI

## Deviations

- `collect_project_files()` nebylo smazáno — stále živá závislost v `ProjectIndex::full_rescan()`. Plán předpokládal, že je nahrazeno `get_files()`, ale `get_files()` jen čte index, `collect_project_files()` ho naplňuje.
- Přidáno 11 nových klíčů místo 12 — `project-search-file-filter` je redundantní s `project-search-file-filter-hint`, proto vynechán (hint stačí jako placeholder text).

## Known Issues

- None

## Files Created/Modified

- `src/app/ui/search_picker.rs` — přestavěný UI dialog s togglery, LayoutJob zvýrazněním, kontextem, klikacími výsledky
- `locales/cs/ui.ftl` — 11 nových project-search-* i18n klíčů
- `locales/en/ui.ftl` — 11 nových project-search-* i18n klíčů
- `locales/sk/ui.ftl` — 11 nových project-search-* i18n klíčů
- `locales/de/ui.ftl` — 11 nových project-search-* i18n klíčů
- `locales/ru/ui.ftl` — 11 nových project-search-* i18n klíčů
