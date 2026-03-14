---
id: S02
parent: M005
milestone: M005
provides:
  - Replace engine — compute_replace_previews() + apply_replacements() standalone funkce
  - ReplacePreview struct pro per-file replace data s selected flag
  - Replace toggle UI v search dialogu (↔ button + replace input)
  - Replace preview dialog s per-file inline diff (similar::TextDiff), checkboxy, select all/deselect all
  - Replace execution flow v workspace handleru (snapshot → write → tab refresh → toast)
  - Per-file error handling — snapshot/write selhání nesmí zastavit replace, reportuje se přes toast
  - i18n klíče pro replace UI (14 klíčů × 5 jazyků)
  - 5 unit testů pro replace logiku (compute, apply, capture groups, partial skip, nonexistent file)
requires:
  - slice: S01
    provides: build_regex(), SearchOptions, ProjectSearch struct, SearchResult s match_ranges, výsledkový dialog
affects: []
key_files:
  - src/app/ui/search_picker.rs
  - src/app/ui/workspace/mod.rs
  - src/app/ui/workspace/state/types.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - pending_replace flag pattern — UI nastaví flag, workspace handler provede snapshot+write po renderingu, oddělené od UI
  - Tab refresh po replace — načtení nového obsahu z disku, reset modified flag + sync last_saved_content
  - Preview dialog default-open collapsing headers pro ≤5 souborů, collapsed pro >5
  - compute_replace_previews bere root path navíc — nutné pro překlad relativních cest z SearchResult na absolutní
  - apply_replacements vrací Vec<(PathBuf, Result<(), String>)> pro per-file error reporting
patterns_established:
  - Replace execution flow: UI nastaví pending_replace → workspace handler iteruje previews → snapshot → write → toast → tab refresh → cleanup
  - Per-file error isolation: snapshot selhání → skip soubor, write selhání → toast + pokračovat
observability_surfaces:
  - pending_replace flag — true jen po kliknutí Confirm, reset na false po zpracování
  - replace_previews vektor — per-file selected stav, vyčištěn po replace
  - Toast queue — per-file snapshot/write chyby s názvem souboru a error message
  - Summary toast — success count nebo success/total/errors ratio
drill_down_paths:
  - .gsd/milestones/M005/slices/S02/tasks/T01-SUMMARY.md
  - .gsd/milestones/M005/slices/S02/tasks/T02-SUMMARY.md
duration: 40m
verification_result: passed
completed_at: 2026-03-13
---

# S02: Project-wide replace s preview a local history

**Replace engine s preview dialogem zobrazujícím per-file inline diff s checkboxy, local history snapshotem před zápisem, per-file error reportingem přes toast a kompletní i18n.**

## What Happened

T01 implementoval replace engine jako dvě standalone funkce. `compute_replace_previews(root, results, regex, replace_text)` deduplikuje soubory z výsledků hledání, načte obsah z disku a provede `regex.replace_all()` — capture groups ($1, $2) fungují automaticky. `apply_replacements(previews)` zapíše `new_content` do souborů kde `selected == true`, selhání jednoho souboru neblokuje ostatní. `ReplacePreview` struct nese plný original+new content pro diff rendering. 4 unit testy pokrývají základní replace, regex capture groups, úspěšný zápis a partial skip.

T02 přidal replace UI — toggle (↔ button) v search dialogu zobrazí replace input pole. "Replace All" spustí `compute_replace_previews()` a otevře preview dialog. Preview dialog renderuje per-file collapsible sekce s checkboxy a inline diff přes `similar::TextDiff` (červená/zelená, pattern z diff_view.rs). Select All / Deselect All nahoře, selection counter ("N z M vybráno"). Confirm button nastaví `pending_replace` flag, workspace handler pak pro každý vybraný soubor: (1) `take_snapshot()` s original_content, (2) `fs::write()` s new_content. Snapshot selhání → toast + skip soubor. Write selhání → toast + pokračovat. Summary toast: "Nahrazeno v N souborech" nebo partial failure "N z M (K chyb)". Po replace se refreshnou otevřené taby (reload z disku, reset modified flag). 14 i18n klíčů ve všech 5 jazycích. 1 nový unit test pro nonexistent file error handling.

## Verification

- `cargo test --bin polycredo-editor app::ui::search_picker` — 20/20 testů pass (15 search + 5 replace)
- `cargo check` — čistá kompilace
- `./check.sh` — fmt, clippy, všechny testy pass, quality gate passed
- `grep -c 'project-search-replace' locales/*/ui.ftl` — 14 per jazyk (požadováno ≥7)
- `apply_replacements()` s neexistující cestou — unit test ověřuje per-file Err(String) bez blokování ostatních
- Diagnostic: `pending_replace` resetován po dokončení, `replace_previews` vyčištěny

## Requirements Advanced

- R020 — Replace preview s per-file checkboxy implementován kompletně: compute, preview dialog, confirm flow, snapshot, write, tab refresh
- R022 — Per-file error handling: snapshot/write selhání → toast, replace pokračuje s dalšími soubory. Unit test ověřuje.
- R023 — Local history snapshot přes take_snapshot() v workspace handleru před zápisem každého souboru. Snapshot selhání blokuje zápis daného souboru.
- R024 — 14 replace-specifických i18n klíčů × 5 jazyků (cs, en, sk, de, ru) doplňuje S01 klíče

## Requirements Validated

- R020 — compute_replace_previews() generuje správné preview data, apply_replacements() zapisuje soubory s per-file error isolation. Preview dialog s inline diff a checkboxy. 5 unit testů. cargo check + ./check.sh pass.
- R022 — apply_replacements() s neexistující cestou vrací Err(String) per-file, nechybný soubor se zapíše. Toast wiring v workspace handleru. Unit test ověřuje.
- R023 — take_snapshot() voláno v workspace handleru (main thread) pro každý modifikovaný soubor. Snapshot selhání → toast + skip soubor (write se nespustí).
- R024 — 21 S01 klíčů + 14 S02 klíčů = 35 project-search-* klíčů × 5 jazyků. Grep ověřuje pokrytí.

## New Requirements Surfaced

- none

## Requirements Invalidated or Re-scoped

- none

## Deviations

- `compute_replace_previews()` bere navíc `root: &Path` parametr — plán uváděl jen `results, regex, replace_text`. Nutné pro překlad relativních cest na absolutní. Konzistentní s `run_project_search()`.
- Přidán `project-search-replace-selection-info` i18n klíč (counter "N z M vybráno") — nebylo v plánu, zlepšuje UX preview dialogu.
- Heading search dialogu se mění na "Nahradit v projektu" když je replace toggle aktivní.

## Known Limitations

- Replace na 100+ souborech blokuje UI na ~100ms (snapshot + write synchronně na main threadu). Akceptovatelné pro MVP.
- Vizuální ověření replace preview dialogu odloženo (headless prostředí, UAT deferred).

## Follow-ups

- none — S02 completion = M005 milestone complete

## Files Created/Modified

- `src/app/ui/search_picker.rs` — replace engine (compute_replace_previews, apply_replacements), replace toggle UI, render_replace_preview_dialog(), 5 unit testů
- `src/app/ui/workspace/mod.rs` — replace execution flow (snapshot + write + tab refresh + toast)
- `src/app/ui/workspace/state/types.rs` — ReplacePreview struct, ProjectSearch rozšířen o replace_previews, show_replace_preview, pending_replace
- `src/app/ui/workspace/state/mod.rs` — export ReplacePreview
- `locales/cs/ui.ftl` — 14 replace i18n klíčů
- `locales/en/ui.ftl` — 14 replace i18n klíčů
- `locales/sk/ui.ftl` — 14 replace i18n klíčů
- `locales/de/ui.ftl` — 14 replace i18n klíčů
- `locales/ru/ui.ftl` — 14 replace i18n klíčů

## Forward Intelligence

### What the next slice should know
- M005 je kompletní. Není žádný další slice v tomto milestonu.

### What's fragile
- Replace na main threadu — synchronní snapshot + write cyklus pro velký počet souborů může být viditelný lag. Pokud se to ukáže jako problém, refaktor na background thread s výsledkovým kanálem (ale take_snapshot() potřebuje &mut LocalHistory na main threadu).

### Authoritative diagnostics
- `ProjectSearch.pending_replace` — pokud je `true` mimo workspace handler, replace flow zasekl
- Toast queue — per-file chyby s cestou a chybovou zprávou, vždy obsahují název souboru

### What assumptions changed
- Plán předpokládal 7 i18n klíčů per jazyk — ve skutečnosti 14 (přidány cancel, selection-info, partial-success a další)
