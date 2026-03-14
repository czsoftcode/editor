---
estimated_steps: 7
estimated_files: 7
---

# T02: Vylepšený UI dialog s togglery, zvýrazněnými výsledky a i18n

**Slice:** S01 — Vylepšený search dialog s regex engine, zvýrazněnými výsledky a kontextem
**Milestone:** M005

## Description

Přestavba UI project search dialogu: nový input dialog s toggle buttony (regex/case/whole-word), file filter pole, inline regex error zobrazení, vylepšený výsledkový dialog se zvýrazněnými matchemi přes LayoutJob a kontextovými řádky, klikací výsledky otevírající soubor, inkrementální akumulace streamovaných výsledků, a kompletní i18n pro 5 jazyků.

## Steps

1. Přestavit `render_project_search_dialog()` — input pole pro query, vedle něj 3 toggle buttons jako `selectable_label` (".*" pro regex, "Aa" pro case-sensitive, "W" pro whole-word). Pod query inputem: file filter input s placeholder hint ("*.rs, *.toml"). Pod tím: inline regex error (červený text) pokud `regex_error.is_some()`. Search se triggeruje při Enter nebo změně togglerů (pokud query neprázdný). Replace toggle button pro zobrazení replace inputu (pro S02).
2. Validace: při každé změně query nebo togglerů zavolat `build_regex()`. Pokud Err → nastavit `regex_error`, nespouštět search. Pokud Ok → vymazat `regex_error`, spustit search.
3. Přestavit `poll_and_render_project_search_results()` — `try_recv()` loop akumulující `SearchBatch::Results` do `results` vektoru. `SearchBatch::Done` → nastavit `searching = false`. `SearchBatch::Error` → toast. Loading indikátor ("Hledám...") dokud `searching == true`.
4. Výsledkový rendering: seskupit výsledky per-soubor (soubor heading jako bold label s relativní cestou). Pro každý match řádek: `LayoutJob` s normálním textem + zvýrazněným match textem (žlutý/oranžový background). Kontextové řádky (context_before/after) s tlumenou barvou. Separator ("...") mezi nesouvisejícími bloky v jednom souboru.
5. Klikatelné výsledky: `ui.add(Label::new(job).sense(Sense::click()))`. Pokud tento pattern nefunguje v aktuální egui verzi, fallback na `Button::new(job)` se stejem LayoutJob. Kliknutí → `open_file_in_ws()` + `jump_to_location()`.
6. i18n: přidat ~12 nových klíčů do všech 5 locale souborů. Klíče: `project-search-regex-toggle`, `project-search-case-toggle`, `project-search-word-toggle`, `project-search-file-filter-hint`, `project-search-regex-error`, `project-search-searching`, `project-search-results-count`, `project-search-replace-heading`, `project-search-replace-btn`, `project-search-replace-with`, `project-search-replace-preview`, `project-search-context-separator`.
7. Cleanup: smazat `collect_project_files()` pokud ještě existuje (nahrazeno `get_files()`). Ověřit dispatch v workspace/mod.rs — `render_project_search_dialog()` a `poll_and_render_project_search_results()` musí fungovat s novými signaturami.

## Must-Haves

- [ ] Toggle buttons (regex/case/whole-word) v input dialogu
- [ ] File filter input pole
- [ ] Inline regex error pod inputem
- [ ] Zvýrazněné matche ve výsledcích přes LayoutJob
- [ ] Kontextové řádky s tlumenou barvou
- [ ] Kliknutí na výsledek otevře soubor na řádku
- [ ] Inkrementální akumulace streamovaných výsledků
- [ ] i18n klíče ve všech 5 jazycích
- [ ] `./check.sh` pass

## Verification

- `cargo check` — kompilace čistá
- `./check.sh` — fmt, clippy, všechny testy pass
- `grep -c 'project-search-' locales/*/ui.ftl` — minimálně 19 per jazyk (7 existujících + 12 nových)

## Inputs

- T01 output: rozšířené typy (SearchResult, ProjectSearch, SearchOptions, SearchBatch), engine funkce (build_regex, search_file_with_context, run_project_search)
- `src/app/ui/editor/diff_view.rs` — LayoutJob pattern reference
- `src/app/ui/workspace/history/mod.rs` — LayoutJob + Label::new(job) pattern reference
- Existující dispatch v `src/app/ui/workspace/mod.rs`

## Expected Output

- `src/app/ui/search_picker.rs` — přestavěný UI dialog s togglery, zvýrazněním, kontextem
- `src/app/ui/workspace/mod.rs` — případné úpravy dispatch flow
- `locales/{cs,en,sk,de,ru}/ui.ftl` — 12 nových i18n klíčů per jazyk
