# S01: Editovatelný panel se syntax highlighting, diff a sync scrollem — UAT

**Milestone:** M003
**Written:** 2026-03-13

## UAT Type

- UAT mode: human-experience
- Why this mode is sufficient: Syntax highlighting barvy, diff overlay kontrast, scroll sync plynulost a editovatelnost TextEdit vyžadují vizuální kontrolu v běžícím editoru. Automatické testy pokrývají logiku (diff→panel, apply_diff_backgrounds), ale vizuální výsledek je netestovatelný headless.

## Preconditions

- Editor spuštěn (`cargo run` nebo release build)
- Otevřený soubor s alespoň 2 historickými verzemi (tj. soubor uložen vícekrát s různým obsahem)
- Znalost aktuálního theme (dark/light) pro posouzení kontrastu

## Smoke Test

Otevřít historii souboru (kontextové menu tabu → "Historie souboru") → levý panel zobrazuje aktuální obsah s barevnou syntaxí → pravý panel zobrazuje historickou verzi s barevnou syntaxí a diff pozadím → kliknout do levého panelu a začít psát → text se mění, tab se označí jako modified (●).

## Test Cases

### 1. Editovatelnost levého panelu (R001)

1. Otevřít historii souboru s ≥2 verzemi
2. Kliknout do levého panelu
3. Napsat text, smazat řádek, přidat nový řádek
4. **Expected:** Text se mění, kurzor je viditelný, chování jako normální TextEdit

### 2. Syntax highlighting v obou panelech (R002)

1. Otevřít historii Rust/JS/Python souboru
2. Podívat se na levý panel
3. Podívat se na pravý panel
4. **Expected:** Oba panely mají syntax barvy (klíčová slova, stringy, komentáře barevně). Barvy odpovídají normálnímu editoru.

### 3. Diff zvýraznění v obou panelech (R009)

1. Otevřít historii souboru kde se mezi verzemi přidal a odebral text
2. Podívat se na levý panel
3. Podívat se na pravý panel
4. **Expected:** Přidané řádky v levém panelu mají zelené pozadí. Odebrané řádky v pravém panelu mají červené pozadí. Syntax barvy textu jsou viditelné přes diff pozadí.

### 4. Scroll sync (R003)

1. Otevřít historii dlouhého souboru (50+ řádků)
2. Scrollovat levým panelem dolů
3. **Expected:** Pravý panel se synchronizovaně posune na odpovídající pozici
4. Scrollovat pravým panelem nahoru
5. **Expected:** Levý panel se synchronizovaně posune zpět

### 5. Tab sync při editaci (R006)

1. Otevřít historii souboru
2. Editovat text v levém panelu (přidat/smazat řádek)
3. Podívat se na tab v tab baru
4. **Expected:** Tab se označí jako modified (●)
5. Zavřít history view
6. **Expected:** Obsah v normálním editoru odpovídá editovanému textu z history view

### 6. Výchozí stav — 1 verze (R007)

1. Otevřít nově vytvořený soubor, uložit jednou
2. Otevřít historii tohoto souboru
3. **Expected:** Levý panel zobrazuje obsah. Pravý panel je prázdný (informační text o absenci historických verzí).

### 7. Výchozí stav — >1 verze (R007)

1. Otevřít soubor s více historickými verzemi
2. Otevřít historii
3. **Expected:** Pravý panel automaticky zobrazuje nejnovější historickou verzi (ne prázdný). Navigační šipky fungují pro přepínání mezi verzemi.

### 8. Diff recompute po editaci

1. Otevřít historii souboru s diff zvýrazněním
2. Editovat text v levém panelu — přidat nebo smazat řádky
3. **Expected:** Diff zvýraznění se aktualizuje na příštím framu — nové přidané řádky se zobrazí zeleně, smazané zmizí z levého panelu. Žádný lag ani freeze.

## Edge Cases

### Velký soubor (1000+ řádků)

1. Otevřít historii velkého souboru
2. Editovat, scrollovat, navigovat mezi verzemi
3. **Expected:** Žádný viditelný lag, žádný freeze. Diff recompute proběhne plynule.

### Soubor bez rozpoznané syntaxe

1. Otevřít historii .txt nebo neznámého typu souboru
2. **Expected:** Oba panely zobrazují text bez syntax highlighting (monochrome), diff pozadí stále funguje.

### Rychlé psaní v levém panelu

1. Otevřít historii a rychle psát do levého panelu
2. **Expected:** Diff se aktualizuje průběžně bez zamrznutí. Content hash invalidace funguje korektně.

## Failure Signals

- Levý panel nelze editovat (chybí kurzor, text se nemění při psaní)
- Syntax barvy chybí v jednom nebo obou panelech (vše monochrome)
- Diff pozadí se nezobrazuje (žádné zelené/červené řádky přestože diff existuje)
- Diff pozadí zakrývá syntax barvy textu (nečitelný text)
- Scroll jedním panelem nepohne druhým
- Scroll sync vytváří feedback loop (nekonečné poskakování)
- Tab se neoznačí jako modified po editaci v history view
- Pravý panel zobrazuje obsah i když existuje jen 1 verze
- UI freeze nebo výrazný lag při editaci nebo scrollování

## Requirements Proved By This UAT

- R001 — Editovatelný levý panel (test 1)
- R002 — Syntax highlighting v obou panelech (test 2)
- R003 — Synchronizovaný scroll (test 4)
- R006 — Editace se propsávají do tab bufferu (test 5)
- R007 — Výchozí stav panelů (testy 6, 7)
- R009 — Diff zvýraznění se syntax highlighting (test 3)

## Not Proven By This UAT

- R004 — Obnovení historické verze (scope S02)
- R005 — Potvrzovací dialog (scope S02)
- R008 — i18n klíče (scope S02)
- Scroll sync přesnost pro extrémně asymetrické diffy (line-based mapování odloženo)
- Vizuální kontrast diff barev ve všech light variantách (WarmIvory, CoolGray, Sepia)

## Notes for Tester

- Scroll sync je proportionální (ne line-based) — pro soubory kde se jedna verze výrazně liší v počtu řádků, scroll může "skákat" místo plynulého mapování. To je known limitation, ne bug.
- Diff barvy mají semiprůhledné pozadí v dark mode a opaque v light mode — testovat oba režimy.
- Pre-existující test `phase35_delete_foundation` selhává — nesouvisí s touto slice.
- `font_size` se bere z `Editor::current_editor_font_size(ui)` — pokud je nastaven nestandardní font size, oba panely by ho měly respektovat.
