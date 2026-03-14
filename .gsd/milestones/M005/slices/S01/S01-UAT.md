# S01: Vylepšený search dialog s regex engine, zvýrazněnými výsledky a kontextem — UAT

**Milestone:** M005
**Written:** 2026-03-13

## UAT Type

- UAT mode: live-runtime
- Why this mode is sufficient: Search dialog vyžaduje interakci s egui UI — togglery, input, klikání na výsledky, vizuální ověření zvýraznění. Unit testy pokrývají engine logiku, UAT ověří end-to-end UX.

## Preconditions

- Editor zkompilován a spuštěn (`cargo run`)
- Otevřen projekt s alespoň 10+ `.rs` soubory (např. samotný polycredo_editor)
- Projekt obsahuje soubory s řetězci "fn ", "TODO", "Result", "SearchResult"

## Smoke Test

1. Stiskni Ctrl+Shift+F
2. **Expected:** Otevře se project search dialog s input polem, 3 toggle buttons (`.*`, `Aa`, `W`) a file filter inputem

## Test Cases

### 1. Regex search — funkce matchující pattern

1. Otevři Ctrl+Shift+F
2. Klikni toggle `.*` (regex ON — podsvícený)
3. Zadej `fn\s+\w+` do query
4. Stiskni Enter
5. **Expected:** Výsledky zobrazí matchující řádky se zvýrazněnými `fn nazev_funkce` (oranžový background na matchující části). Výsledky seskupeny per-soubor s bold filename heading. Kontextové řádky ±2 s tlumenou barvou.

### 2. Case-sensitive search

1. Otevři Ctrl+Shift+F
2. Ujisti se, že regex je OFF (`.*` nepodsvícený)
3. Klikni toggle `Aa` (case-sensitive ON)
4. Zadej `TODO`
5. Stiskni Enter
6. **Expected:** Výsledky obsahují POUZE řádky s "TODO" (velká písmena). Žádné "todo", "Todo" apod.

### 3. Case-insensitive search (default)

1. Otevři Ctrl+Shift+F
2. Ujisti se, že `Aa` je OFF (case-insensitive)
3. Zadej `todo`
4. Stiskni Enter
5. **Expected:** Výsledky obsahují "TODO", "todo", "Todo" — všechny varianty.

### 4. Whole-word search

1. Otevři Ctrl+Shift+F
2. Klikni toggle `W` (whole-word ON)
3. Zadej `Result`
4. Stiskni Enter
5. **Expected:** Výsledky obsahují "Result" jako celé slovo, ALE NE "SearchResult", "ResultSet" apod.

### 5. Whole-word OFF (substring)

1. Otevři Ctrl+Shift+F
2. Ujisti se, že `W` je OFF
3. Zadej `Result`
4. Stiskni Enter
5. **Expected:** Výsledky obsahují "Result" i "SearchResult", "SearchResults" atd.

### 6. File type filtr — jen Rust soubory

1. Otevři Ctrl+Shift+F
2. Do file filter pole zadej `*.rs`
3. Zadej libovolný query (např. `fn`)
4. Stiskni Enter
5. **Expected:** Výsledky obsahují POUZE soubory s příponou `.rs`. Žádné `.toml`, `.md`, `.ftl` soubory.

### 7. File type filtr — jen TOML soubory

1. Otevři Ctrl+Shift+F
2. Do file filter pole zadej `*.toml`
3. Zadej `version`
4. Stiskni Enter
5. **Expected:** Výsledky pouze z `.toml` souborů (Cargo.toml, settings.toml).

### 8. Kliknutí na výsledek otevře soubor

1. Proveď libovolný search s výsledky
2. Klikni na jeden z výsledkových řádků (kurzor se změní na PointingHand při hoveru)
3. **Expected:** Soubor se otevře v editoru na správném řádku. Kurzor je na řádku, kde je match.

### 9. Nevalidní regex — inline error

1. Otevři Ctrl+Shift+F
2. Klikni toggle `.*` (regex ON)
3. Zadej nevalidní regex: `[unclosed`
4. Stiskni Enter
5. **Expected:** Pod inputem se zobrazí červený text s chybovou zprávou (obsahuje "Neplatný regex:"). Search se NESPUSTÍ — žádné výsledky.

### 10. Prázdný query

1. Otevři Ctrl+Shift+F
2. Nech query pole prázdné
3. Stiskni Enter
4. **Expected:** Search se nespustí. Zobrazí se chybová zpráva (inline error).

### 11. Inkrementální streamování

1. Otevři Ctrl+Shift+F
2. Zadej běžný pattern (např. `fn`) v projektu s mnoha soubory
3. Stiskni Enter
4. **Expected:** Výsledky se zobrazují postupně (ne naráz po sekundách). Během hledání je vidět spinner/loading indikátor s textem "Hledám...". Po dokončení zmizí.

### 12. Toggle automaticky re-spustí search

1. Otevři Ctrl+Shift+F
2. Zadej `TODO` a stiskni Enter (výsledky se zobrazí)
3. Klikni toggle `Aa` (case-sensitive ON)
4. **Expected:** Search se automaticky re-spustí s novými nastaveními. Počet výsledků se změní (jen přesné "TODO").

### 13. Kontextové řádky se sloučením

1. Otevři Ctrl+Shift+F
2. Najdi soubor kde jsou dva matche blízko sebe (≤4 řádky)
3. Zadej příslušný pattern
4. **Expected:** Blízké matche mají sloučený kontextový blok (ne opakující se řádky). Nesouvisející bloky odděleny separátorem `···`.

## Edge Cases

### Prázdný projekt (žádné soubory)

1. Otevři editor bez projektu nebo s prázdným projektem
2. Stiskni Ctrl+Shift+F a zadej libovolný query
3. **Expected:** Search proběhne bez pádu. Zobrazí se "žádné výsledky" stav.

### Speciální regex znaky v plain mode

1. Otevři Ctrl+Shift+F
2. Ujisti se, že regex je OFF
3. Zadej `.` (tečka — regex metaznak)
4. **Expected:** Hledá doslovnou tečku, ne libovolný znak. Výsledky obsahují jen řádky s literální tečkou.

### File filter bez matchů

1. Otevři Ctrl+Shift+F
2. Do file filter zadej `*.xyz` (neexistující přípona)
3. Zadej libovolný query
4. **Expected:** Žádné výsledky. Žádný pád ani chyba.

### Nevalidní glob pattern

1. Otevři Ctrl+Shift+F
2. Do file filter zadej nevalidní glob: `[unclosed`
3. Zadej libovolný query
4. **Expected:** Toast chybová zpráva o nevalidním file filtru. Search se nespustí nebo proběhne bez filtru.

### Kombinace všech togglerů

1. Otevři Ctrl+Shift+F
2. Zapni `.*` (regex) + `Aa` (case) + `W` (whole-word) současně
3. Zadej `TODO`
4. **Expected:** Hledá case-sensitive celé slovo "TODO" jako regex. Výsledky obsahují jen přesné "TODO" jako celé slovo.

## Failure Signals

- Search dialog se neotevře na Ctrl+Shift+F
- Togglery nereagují na kliknutí nebo nemají vizuální indikaci stavu
- Výsledky nemají zvýrazněné matche (chybí oranžový background)
- Kliknutí na výsledek neotevře soubor nebo skočí na špatný řádek
- Case-sensitive toggle nemá efekt (case-insensitive stále vrací všechny varianty)
- Whole-word toggle nemá efekt (substring matche se stále zobrazují)
- File filter nemá efekt (zobrazují se soubory jiných typů)
- Nevalidní regex způsobí panic místo inline error
- Výsledky se zobrazí až po dokončení celého searche (žádné streamování)
- Chybí kontextové řádky (jen match řádek bez okolí)
- Duplicitní kontextové řádky u blízkých matchů (nesloučené bloky)

## Requirements Proved By This UAT

- R016 — Regex search engine s togglery (test cases 1-5, 12, edge case "kombinace")
- R017 — Zvýrazněné matchující části (test case 1 — oranžový bg)
- R018 — Kontextové řádky se sloučením (test cases 1, 13)
- R019 — File type filtr (test cases 6, 7, edge cases "bez matchů", "nevalidní glob")
- R021 — Regex error inline (test cases 9, 10)
- R024 — i18n pro nové UI prvky (implicitně — UI texty ve správném jazyce dle nastavení)
- R025 — Inkrementální streamování (test case 11)

## Not Proven By This UAT

- R020 — Project-wide replace s preview (S02 scope)
- R022 — Replace I/O error reporting (S02 scope)
- R023 — Local history snapshot před replace (S02 scope)
- R024 — i18n kompletní pro replace UI prvky (S02 doplní)
- Pixel-perfect vizuální ověření barev zvýraznění (headless prostředí)

## Notes for Tester

- Togglery jsou malé `selectable_label` tlačítka — `.*` pro regex, `Aa` pro case-sensitive, `W` pro whole-word. Podsvícený stav = zapnuto.
- File filter input je pod toggle řádkem s placeholder hintem (např. "*.rs, *.toml").
- Výsledky jsou klikací (kurzor se změní na PointingHand) — kliknutí otevře soubor a skočí na řádek.
- Separátor `···` mezi nesouvisejícími kontextovými bloky v rámci jednoho souboru.
- Spinner a text "Hledám..." indikují probíhající search.
