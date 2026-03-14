# S02: In-file search s regex/case/whole-word togglery

**Goal:** Rozšířit in-file search (Ctrl+F) o regex/case-sensitive/whole-word togglery sdílející `build_regex()` engine z M005. Nahrazuje primitivní substring match za plnohodnotný regex matching.
**Demo:** Ctrl+F → search bar zobrazí regex/case/whole-word toggle buttony → zadání regex patternu (např. `fn\s+\w+`) → matche se zvýrazní v editoru → whole-word toggle rozliší "Result" od "SearchResult" → replace funguje se regex matchi.

## Must-Haves

- Toggle buttons (regex/case/whole-word) v search baru vedle query inputu
- `update_search()` přepsán z substring match na `build_regex()` + `regex.find_iter()`
- Replace operace fungují s regex byte ranges (replace_current, replace_all)
- Stav togglerů persistuje v Editor structu (přetrvá přes close/reopen Ctrl+F)
- i18n klíče pro nové toggle labely (pokud chybí) ve všech 5 jazycích

## Verification

- `cargo check` — čistá kompilace
- `./check.sh` — fmt, clippy, testy pass
- `grep 'build_regex' src/app/ui/editor/search.rs` → nalezeno (napojení na engine)
- `grep 'search_use_regex\|search_case_sensitive\|search_whole_word' src/app/ui/editor/mod.rs` → 3 výskyty (nové fieldy)

## Tasks

- [ ] **T01: Regex/case/whole-word togglery v in-file search** `est:45m`
  - Why: Jediný task — scope je koherentní (3 nové fieldy, přepis jedné funkce, UI rozšíření jednoho baru, i18n). Izolovaná změna v editor/search.rs + editor/mod.rs.
  - Files: `src/app/ui/editor/mod.rs`, `src/app/ui/editor/search.rs`, `locales/{cs,en,sk,de,ru}/ui.ftl`
  - Do: 1) V Editor struct přidat `search_use_regex: bool` (default false), `search_case_sensitive: bool` (default false), `search_whole_word: bool` (default false). 2) V `search_bar()` přidat 3 toggle buttons (selectable_label) před/za query input — `.*` regex, `Aa` case, `W` whole-word. Stejný vizuální pattern jako v project search panelu. 3) Přepsat `update_search()`: místo `eq_ignore_ascii_case` char_indices loop → `build_regex(query, SearchOptions { use_regex, case_sensitive, whole_word })` → `regex.find_iter(content)` → `search_matches = matches.map(|m| (m.start(), m.end())).collect()`. 4) Regex error handling: pokud `build_regex()` vrací Err, zobrazit krátkou chybu v search baru (červeně) a nespouštět search. Přidat `search_regex_error: Option<String>` do Editor. 5) Ověřit že replace_current a replace_all fungují s novými byte ranges — `regex.find_iter()` vrací byte offsets, existující `replace_range(start..end)` je konzistentní. 6) Toggle state persistence: fieldy v Editor struct přetrvávají přes close/reopen Ctrl+F (show_search = false nezmaže toggle stav). 7) i18n: přidat klíče `search-regex-toggle`, `search-case-toggle`, `search-word-toggle` do všech 5 jazyků (nebo sdílet existující project-search-* klíče pokud texty jsou identické). 8) `cargo fmt` + `cargo clippy`. 9) `./check.sh`.
  - Verify: `cargo check` + `./check.sh`. `grep 'build_regex' src/app/ui/editor/search.rs` nalezeno.
  - Done when: Ctrl+F search bar má 3 togglery, regex matching funguje, replace funguje s regex matches, `./check.sh` projde.

## Files Likely Touched

- `src/app/ui/editor/mod.rs` — nové fieldy v Editor struct
- `src/app/ui/editor/search.rs` — přepis update_search(), rozšíření search_bar()
- `locales/cs/ui.ftl` — nové i18n klíče
- `locales/en/ui.ftl` — nové i18n klíče
- `locales/sk/ui.ftl` — nové i18n klíče
- `locales/de/ui.ftl` — nové i18n klíče
- `locales/ru/ui.ftl` — nové i18n klíče
