# S02: In-file search s regex/case/whole-word togglery — UAT

**Milestone:** M006
**Written:** 2026-03-13

## UAT Type

- UAT mode: mixed
- Why this mode is sufficient: Kompilační a lintovací verifikace artifact-driven (cargo check, clippy, testy). UI toggle chování a regex matching vyžadují vizuální ověření na desktopu.

## Preconditions

- Editor zkompilován a spuštěn (`cargo run`)
- Otevřen soubor s dostatkem textu pro testování matchů (např. Rust zdrojový kód)
- In-file search bar zavřen (Ctrl+F není aktivní)

## Smoke Test

1. Stiskni Ctrl+F
2. **Expected:** Search bar se zobrazí s query inputem a 3 toggle buttons (`.* / Aa / W`) vedle inputu

## Test Cases

### 1. Základní regex toggle

1. Otevři Ctrl+F search bar
2. Ujisti se, že regex toggle (`.*`) je vypnutý (nezvýrazněný)
3. Zadej text `fn\s+\w+` do query
4. **Expected:** Žádné matche (literal substring `fn\s+\w+` se v kódu nevyskytuje)
5. Klikni na regex toggle (`.*`) — aktivuje se
6. **Expected:** Matche se zvýrazní na řádcích obsahujících `fn jmeno_funkce` — regex matchuje

### 2. Case-sensitive toggle

1. V Ctrl+F search baru zadej `Result`
2. Ujisti se, že case toggle (`Aa`) je vypnutý
3. **Expected:** Matche zahrnují `Result`, `result`, `RESULT` — case-insensitive
4. Klikni na case toggle (`Aa`) — aktivuje se
5. **Expected:** Pouze přesné `Result` (velké R) se zvýrazní, `result` zmizí z matchů

### 3. Whole-word toggle

1. V Ctrl+F search baru zadej `Result`
2. Ujisti se, že whole-word toggle (`W`) je vypnutý
3. **Expected:** Matchuje i `SearchResult`, `ResultSet` — substring match
4. Klikni na whole-word toggle (`W`) — aktivuje se
5. **Expected:** Pouze samostatné slovo `Result` se zvýrazní, `SearchResult` a `ResultSet` zmizí z matchů

### 4. Kombinace togglerů

1. V Ctrl+F search baru zadej `fn\s+new`
2. Aktivuj regex toggle (`.*`) + case toggle (`Aa`)
3. **Expected:** Matchuje pouze `fn new` (ne `fn New`, pokud existuje) — regex + case-sensitive kombinace
4. Deaktivuj case toggle
5. **Expected:** Matchuje `fn new` i `fn New` — regex + case-insensitive

### 5. Nevalidní regex pattern

1. V Ctrl+F search baru aktivuj regex toggle (`.*`)
2. Zadej `(unclosed`
3. **Expected:** Červený error text se zobrazí v search baru, žádné matche (0/0 counter), navigační šipky neaktivní

### 6. Replace s regex matches

1. V Ctrl+F search baru zadej platný pattern (např. `todo`)
2. Otevři replace (Ctrl+H nebo toggle)
3. Zadej replacement text (např. `TODO`)
4. Klikni Replace (nahradí aktuální match)
5. **Expected:** Aktuální match nahrazen, další match zvýrazněn
6. Klikni Replace All
7. **Expected:** Všechny zbývající matche nahrazeny

### 7. Toggle stav persistence

1. Otevři Ctrl+F, aktivuj regex toggle a whole-word toggle
2. Zavři search bar (Escape)
3. Znovu otevři Ctrl+F
4. **Expected:** Regex a whole-word togglery zůstávají aktivní (zvýrazněné) — stav persistuje

### 8. i18n toggle tooltipy

1. Přepni jazyk editoru na angličtinu (Settings)
2. Otevři Ctrl+F
3. Najeď myší na regex toggle (`.*`)
4. **Expected:** Tooltip zobrazí anglický text (např. "Regex")
5. Přepni jazyk na češtinu
6. **Expected:** Tooltip zobrazí český text (např. "Regulární výraz")

## Edge Cases

### Prázdný query s togglery

1. Otevři Ctrl+F s aktivovaným regex toggle
2. Smaž celý query (prázdný input)
3. **Expected:** Žádné matche (0/0), žádný error, search_matches prázdný

### Velmi dlouhý regex error

1. Aktivuj regex toggle
2. Zadej velmi dlouhý nevalidní pattern (30+ znaků se syntaktickou chybou)
3. **Expected:** Error text je truncated (max 40 znaků) s `…` — nezlomí layout search baru

### Regex s unicode

1. Aktivuj regex toggle
2. Zadej `[čřž]+` (regex matchující české znaky)
3. **Expected:** Matche zvýrazní sekvence českých znaků v souboru

## Failure Signals

- Regex toggle (`.* / Aa / W`) buttons nejsou viditelné v search baru
- `build_regex()` není voláno (grep v search.rs nevrací výsledky)
- Regex error se nezobrazuje při nevalidním patternu
- Replace nefunguje s regex byte ranges (IndexOutOfBounds panic)
- Toggle stav se resetuje po zavření/otevření search baru
- i18n tooltipy chybí nebo zobrazují klíč místo textu
- `./check.sh` neprochází

## Requirements Proved By This UAT

- R030 — In-file search s regex/case/whole-word togglery sdílející build_regex() engine
- R033 (partial) — i18n pro in-file search toggle labely ve všech 5 jazycích

## Not Proven By This UAT

- Vizuální soulad toggle buttons s project search panelem (design review)
- Performance s velkými soubory (10k+ řádků) — regrese benchmarking mimo scope

## Notes for Tester

- Toggle buttons jsou `selectable_label` — vizuálně se zvýrazní při aktivaci, ne jako standardní checkbox
- Regex error text je záměrně truncated na 40 znaků aby se vešel do search baru
- Byte offsets z `regex.find_iter()` jsou konzistentní s `String::replace_range()` — replace by měl fungovat beze změn
