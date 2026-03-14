---
id: S02
parent: M006
milestone: M006
provides:
  - Regex/case-sensitive/whole-word toggle buttons v in-file search baru (Ctrl+F)
  - update_search() přepisán z primitivního substring match na build_regex() + regex.find_iter()
  - Regex error handling s inline zobrazením v search baru
  - i18n klíče search-regex-toggle, search-case-toggle, search-word-toggle ve všech 5 jazycích
requires:
  - slice: S01
    provides: build_regex() engine + SearchOptions struct z M005, potvrzené znovupoužití v inline panelu
affects: []
key_files:
  - src/app/ui/editor/mod.rs
  - src/app/ui/editor/search.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - Sdílení build_regex() engine ze search_picker — žádná duplikace logiky
  - Dedikované i18n klíče (search-*-toggle) namísto sdílení project-search-* klíčů — odlišné tooltipy
patterns_established:
  - Toggle button pattern (selectable_label + on_hover_text) sjednocen mezi project search a in-file search
observability_surfaces:
  - editor.search_regex_error — červený text v search baru při nevalidním regex patternu
drill_down_paths:
  - .gsd/milestones/M006/slices/S02/tasks/T01-SUMMARY.md
duration: 20m
verification_result: passed
completed_at: 2026-03-13
---

# S02: In-file search s regex/case/whole-word togglery

**In-file search (Ctrl+F) rozšířen o regex/case-sensitive/whole-word togglery sdílející build_regex() engine — primitivní substring match nahrazen plnohodnotným regex matchingem.**

## What Happened

Jeden task (T01) provedl celou změnu:

1. Do `Editor` struct přidány 4 nové fieldy: `search_use_regex` (bool), `search_case_sensitive` (bool), `search_whole_word` (bool), `search_regex_error` (Option<String>). Všechny default false/None, persistují přes close/reopen search baru.

2. V `search_bar()` přidány 3 `selectable_label` toggle buttons (`.* / Aa / W`) za search input, před navigační šipky. Vizuální vzor identický s project search panelem z M006/S01. Kliknutí na toggle okamžitě spouští `update_search()`.

3. `update_search()` kompletně přepsán: místo ručního `char_indices` + `eq_ignore_ascii_case` loop nyní volá `build_regex()` ze `search_picker` s `SearchOptions { use_regex, case_sensitive, whole_word }`, pak `regex.find_iter()` pro kolekci matchů. Byte offsets z regex jsou konzistentní s existujícím `replace_range()`.

4. Regex error handling: nevalidní pattern → `search_regex_error` se nastaví → červený text v search baru (truncated na 40 znaků) → matche vyčištěny → replace buttons neaktivní.

5. i18n klíče `search-regex-toggle`, `search-case-toggle`, `search-word-toggle` přidány do všech 5 locale souborů.

## Verification

- `./check.sh` → "Quality Gate: All checks passed successfully!" (192 unit + 37 integration testů)
- `cargo check` → čistá kompilace
- `cargo clippy` → žádné warningy
- `grep 'build_regex' src/app/ui/editor/search.rs` → 2 výskyty (import + volání)
- `grep -c 'search_use_regex|search_case_sensitive|search_whole_word' src/app/ui/editor/mod.rs` → 6 výskytů
- `grep 'search_regex_error' src/app/ui/editor/search.rs` → 3 výskyty (clear, set, display)
- `grep -c 'search-regex-toggle|search-case-toggle|search-word-toggle' locales/*/ui.ftl` → 6 per jazyk × 5 jazyků

## Requirements Advanced

- R030 — In-file search nyní používá build_regex() s regex/case/whole-word togglery místo substring match

## Requirements Validated

- R030 — build_regex() napojení ověřeno grep kontrolou, toggle fieldy v Editor struct, i18n kompletní, ./check.sh pass

## New Requirements Surfaced

- none

## Requirements Invalidated or Re-scoped

- none

## Deviations

Žádné odchylky od plánu.

## Known Limitations

Žádné — scope slice je kompletní.

## Follow-ups

- none

## Files Created/Modified

- `src/app/ui/editor/mod.rs` — 4 nové fieldy v Editor struct (search_use_regex, search_case_sensitive, search_whole_word, search_regex_error) + inicializace v new()
- `src/app/ui/editor/search.rs` — importy build_regex/SearchOptions, přepsaný update_search() na regex engine, 3 toggle buttons v search_bar(), regex error zobrazení
- `locales/cs/ui.ftl` — 3 nové i18n klíče
- `locales/en/ui.ftl` — 3 nové i18n klíče
- `locales/sk/ui.ftl` — 3 nové i18n klíče
- `locales/de/ui.ftl` — 3 nové i18n klíče
- `locales/ru/ui.ftl` — 3 nové i18n klíče

## Forward Intelligence

### What the next slice should know
- M006 je nyní kompletní — oba slicey hotové, milestone definition of done splněn.

### What's fragile
- Regex byte offsets z `regex.find_iter()` předpokládají, že `content` je validní UTF-8 String — pokud by editor přešel na rope-based buffer, matching by potřeboval přepsat.

### Authoritative diagnostics
- `editor.search_regex_error` — při nevalidním regex patternu se zobrazí červený text, grep v search.rs ověří přítomnost

### What assumptions changed
- Žádné — scope odpovídal plánu
