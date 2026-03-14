---
estimated_steps: 9
estimated_files: 7
---

# T01: Regex/case/whole-word togglery v in-file search

**Slice:** S02 — In-file search s regex/case/whole-word togglery
**Milestone:** M006

## Description

Rozšířit in-file search bar (Ctrl+F) o 3 toggle buttons (regex/case/whole-word), přepsat `update_search()` z primitivního substring match na `build_regex()` + `regex.find_iter()`, a přidat i18n klíče. Izolovaná změna v editor/search.rs + editor/mod.rs bez vlivu na ostatní moduly.

## Steps

1. V `src/app/ui/editor/mod.rs` — přidat nové fieldy do Editor struct:
   - `pub search_use_regex: bool` (default false)
   - `pub search_case_sensitive: bool` (default false)
   - `pub search_whole_word: bool` (default false)
   - `pub search_regex_error: Option<String>` (default None)
   - Přidat do `Default` nebo `new()` implementace

2. V `src/app/ui/editor/search.rs` — rozšířit `search_bar()` o toggle buttons:
   - Přidat 3 `selectable_label` toggle buttons vedle query inputu
   - `.*` pro regex toggle → `editor.search_use_regex`
   - `Aa` pro case toggle → `editor.search_case_sensitive`
   - `W` pro whole-word toggle → `editor.search_whole_word`
   - Vizuální pattern identický s project search panelem (S01)
   - Při změně toggleru → zavolat `update_search()` pro refresh matchů

3. V `search_bar()` — zobrazit regex error:
   - Pokud `editor.search_regex_error.is_some()` → zobrazit krátkou chybu červeně pod/vedle inputu
   - Omezit délku na ~40 znaků aby se vešla do search baru

4. Přepsat `update_search()`:
   - Import `build_regex` a `SearchOptions` ze `search_picker`
   - Vytvořit `SearchOptions { use_regex: editor.search_use_regex, case_sensitive: editor.search_case_sensitive, whole_word: editor.search_whole_word }`
   - Zavolat `build_regex(&editor.search_query, &opts)`
   - Pokud `Err(msg)` → nastavit `editor.search_regex_error = Some(msg)`, vyčistit `search_matches`, return
   - Pokud `Ok(regex)` → `editor.search_regex_error = None`
   - `editor.search_matches = regex.find_iter(&editor_content).map(|m| (m.start(), m.end())).collect()`
   - Smazat starý `eq_ignore_ascii_case` substring match kód

5. Ověřit replace_current a replace_all kompatibilitu:
   - `regex.find_iter()` vrací byte offsets — konzistentní s existujícím `replace_range(start..end)`
   - Replace text je plain string (regex capture groups v in-file replace nejsou scope)
   - Ověřit že `current_match` index navigace funguje s novými matches

6. i18n — přidat klíče do `locales/{cs,en,sk,de,ru}/ui.ftl`:
   - `search-regex-toggle` = "Regulární výraz" / "Regular expression" / ...
   - `search-case-toggle` = "Rozlišovat velikost" / "Match case" / ...
   - `search-word-toggle` = "Celá slova" / "Whole word" / ...
   - Nebo tooltip texty pro toggle buttons

7. `cargo fmt` — formátování

8. `cargo clippy` — vyřešit warningy

9. `./check.sh` — final quality gate

## Must-Haves

- [ ] 3 toggle buttons v search baru (regex/case/whole-word)
- [ ] update_search() používá build_regex() + regex.find_iter()
- [ ] Regex error zobrazena v search baru
- [ ] Replace operace fungují s novými byte ranges
- [ ] i18n klíče ve všech 5 jazycích
- [ ] `./check.sh` projde

## Verification

- `./check.sh` — "Quality Gate: All checks passed successfully!"
- `grep 'build_regex' src/app/ui/editor/search.rs` → nalezeno
- `grep 'search_use_regex' src/app/ui/editor/mod.rs` → nalezeno
- `grep 'search_case_sensitive' src/app/ui/editor/mod.rs` → nalezeno
- `grep 'search_whole_word' src/app/ui/editor/mod.rs` → nalezeno
- `grep 'search-regex-toggle\|search-case-toggle\|search-word-toggle' locales/cs/ui.ftl` → 3 výskyty
- Existující testy: `cargo test --lib` — všechny pass

## Inputs

- `src/app/ui/editor/search.rs` — stávající search_bar() a update_search() jako základ pro rozšíření
- `src/app/ui/editor/mod.rs` — Editor struct pro nové fieldy
- `src/app/ui/search_picker.rs` — `build_regex()` a `SearchOptions` pro import

## Expected Output

- `src/app/ui/editor/mod.rs` — Editor struct s novými search toggle fieldy
- `src/app/ui/editor/search.rs` — rozšířený search_bar() s togglery, přepsaný update_search() s build_regex()
- `locales/{cs,en,sk,de,ru}/ui.ftl` — 3 nové i18n klíče × 5 jazyků
