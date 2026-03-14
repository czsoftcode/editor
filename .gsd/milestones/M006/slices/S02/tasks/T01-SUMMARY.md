---
id: T01
parent: S02
milestone: M006
provides:
  - Regex/case/whole-word toggle buttons v in-file search baru (Ctrl+F)
  - update_search() přepisán na build_regex() + regex.find_iter()
  - Regex error zobrazení v search baru
  - i18n klíče pro togglery ve všech 5 jazycích
key_files:
  - src/app/ui/editor/mod.rs
  - src/app/ui/editor/search.rs
  - locales/cs/ui.ftl
  - locales/en/ui.ftl
  - locales/sk/ui.ftl
  - locales/de/ui.ftl
  - locales/ru/ui.ftl
key_decisions:
  - Sdílení build_regex() engine z search_picker — žádná duplikace logiky
  - Dedikované i18n klíče (search-*-toggle) namísto sdílení project-search-* klíčů — odlišné tooltipy
patterns_established:
  - Toggle button pattern (selectable_label + on_hover_text) sjednocen mezi project search a in-file search
observability_surfaces:
  - editor.search_regex_error zobrazuje červený text v search baru při nevalidním regex patternu
duration: 20m
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: Regex/case/whole-word togglery v in-file search

**Přidány 3 toggle buttons (regex/case/whole-word) do in-file search baru a přepsán update_search() z primitivního substring match na sdílený build_regex() engine.**

## What Happened

1. Do `Editor` struct přidány 4 nové fieldy: `search_use_regex`, `search_case_sensitive`, `search_whole_word` (bool, default false) a `search_regex_error` (Option<String>, default None).

2. V `search_bar()` přidány 3 `selectable_label` toggle buttons (`.* / Aa / W`) za search input, před navigační šipky. Vizuální vzor identický s project search panelem. Při kliknutí na toggle se okamžitě spouští `update_search()`.

3. `update_search()` kompletně přepsán: místo ručního `char_indices` + `eq_ignore_ascii_case` loop nyní volá `build_regex()` ze `search_picker` s `SearchOptions`, pak `regex.find_iter()` pro kolekci matchů. Při nevalidním regex patternu nastaví `search_regex_error` a vyčistí matche.

4. Regex error se zobrazuje červeně přímo v search baru (truncated na 40 znaků).

5. Replace operace (`replace_current`, `replace_all`) fungují beze změn — `regex.find_iter()` vrací byte offsets konzistentní s `String::replace_range()`.

6. i18n klíče `search-regex-toggle`, `search-case-toggle`, `search-word-toggle` přidány do všech 5 locale souborů (cs, en, sk, de, ru).

## Verification

- `./check.sh` — "Quality Gate: All checks passed successfully!" (192 unit + 37 integration testů)
- `cargo check` — čistá kompilace
- `cargo clippy` — žádné warningy
- `grep 'build_regex' src/app/ui/editor/search.rs` → 2 výskyty (import + volání)
- `grep 'search_use_regex' src/app/ui/editor/mod.rs` → nalezeno
- `grep 'search_case_sensitive' src/app/ui/editor/mod.rs` → nalezeno
- `grep 'search_whole_word' src/app/ui/editor/mod.rs` → nalezeno
- `grep 'search-regex-toggle\|search-case-toggle\|search-word-toggle' locales/cs/ui.ftl` → 3 výskyty
- `grep 'search_regex_error' src/app/ui/editor/search.rs` → nalezeno (3 výskyty: clear, set, display)
- Slice-level verification: všechny 4 body pass (cargo check, check.sh, build_regex grep, fieldy grep)

## Diagnostics

- `editor.search_regex_error` — při nevalidním regex patternu zobrazí červený text v search baru. Agent ověří přes `grep 'search_regex_error' src/app/ui/editor/search.rs`.
- Toggle stav persistuje v Editor structu — přežije close/reopen search baru (show_search = false nezmaže toggle stav).

## Deviations

Žádné odchylky od plánu.

## Known Issues

Žádné.

## Files Created/Modified

- `src/app/ui/editor/mod.rs` — přidány 4 nové fieldy do Editor struct (search_use_regex, search_case_sensitive, search_whole_word, search_regex_error) + inicializace v new()
- `src/app/ui/editor/search.rs` — importy build_regex/SearchOptions, přepsaný update_search() na regex engine, 3 toggle buttons v search_bar(), regex error zobrazení
- `locales/cs/ui.ftl` — 3 nové i18n klíče (search-regex-toggle, search-case-toggle, search-word-toggle)
- `locales/en/ui.ftl` — 3 nové i18n klíče
- `locales/sk/ui.ftl` — 3 nové i18n klíče
- `locales/de/ui.ftl` — 3 nové i18n klíče
- `locales/ru/ui.ftl` — 3 nové i18n klíče
- `.gsd/milestones/M006/slices/S02/S02-PLAN.md` — přidána Observability sekce + failure-path verifikace
- `.gsd/milestones/M006/slices/S02/tasks/T01-PLAN.md` — přidána Observability Impact sekce
