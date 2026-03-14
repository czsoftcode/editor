# S01: Inline project search panel — UAT

**Milestone:** M006
**Written:** 2026-03-13

## UAT Type

- UAT mode: mixed (artifact-driven compilation + live-runtime vizuální ověření)
- Why this mode is sufficient: Kompilace a unit testy ověřují engine a integritu kódu. Vizuální UAT na desktopu ověřuje layout, resize, fokus transfer a UX flow — tyto aspekty nelze ověřit headless.

## Preconditions

- Projekt zkompilován: `cargo build` projde bez chyb
- Editor spuštěn s projektem obsahujícím ≥3 zdrojové soubory (pro per-file seskupení výsledků)
- Žádný modální dialog není otevřen
- Project search panel je zavřený (výchozí stav)

## Smoke Test

1. Stiskni Ctrl+Shift+F
2. **Expected:** Pod editorem se otevře search panel s query inputem, togglery (`.* Aa W ↔ ✕`) a prázdnou oblastí výsledků. Editor zůstane viditelný nad panelem.

## Test Cases

### 1. Panel otevření a layout

1. Stiskni Ctrl+Shift+F
2. Zkontroluj že panel se zobrazil pod editorem
3. Klikni do editoru a zapiš text
4. **Expected:** Editor je plně editovatelný — panel neblokuje editaci. Panel má heading s lokalizovaným názvem.

### 2. Základní vyhledávání

1. Stiskni Ctrl+Shift+F (panel otevřen)
2. Do query inputu zadej výraz, který se vyskytuje ve více souborech (např. `fn `)
3. Stiskni Enter
4. **Expected:** Spinner/"Hledám..." se zobrazí. Výsledky streamují inkrementálně — soubory se objevují postupně. Po dokončení spinner zmizí. Výsledky jsou seskupeny per-file s filename hlavičkou.

### 3. Match highlighting a kontext

1. Proveď vyhledávání s výrazem, který matchuje v několika řádcích
2. Prohlédni výsledky v panelu
3. **Expected:** Matchující části textu jsou zvýrazněny (oranžový background). Kontextové řádky (±2) se zobrazují dim barvou. Nesouvisející bloky odděleny separátorem.

### 4. Klik na výsledek — navigace s fokusem

1. Proveď vyhledávání s výsledky
2. Klikni na libovolný výsledek v panelu
3. **Expected:** Editor otevře příslušný soubor a jumpne na řádek s matchem. Cursor je viditelný na cílovém řádku. Panel zůstane otevřený s výsledky. Kliknutý výsledek má subtilní vizuální highlight (modrý tint).

### 5. Togglery (regex, case, whole-word)

1. Do query zadej `test` s vypnutým case toggle
2. Ověř že výsledky obsahují `Test`, `TEST`, `test`
3. Zapni case toggle (`Aa`)
4. **Expected:** Automaticky se spustí nové hledání. Výsledky obsahují jen přesný case match `test`.

5. Do query zadej `.*` → zapni regex toggle (`.*`)
6. **Expected:** Výsledky matchují jako regex pattern (ne literální `.*`).

7. Do query zadej `fn` → zapni whole-word toggle (`W`)
8. **Expected:** Výsledky obsahují `fn` jako celé slovo, ne `fn` uvnitř `function` nebo `filename`.

### 6. File filter

1. Do file filter inputu zadej `*.rs`
2. Spusť hledání
3. **Expected:** Výsledky obsahují jen soubory s příponou `.rs`.

### 7. Neplatný regex

1. Zapni regex toggle
2. Do query zadej `[invalid`
3. **Expected:** Pod query inputem se zobrazí červená chybová hláška o neplatném regex. Hledání se nespustí.

### 8. Replace flow z panelu

1. Proveď vyhledávání s výsledky
2. Klikni na replace toggle (`↔`)
3. **Expected:** Zobrazí se replace input pole a Replace All button.

4. Do replace inputu zadej náhradní text
5. Klikni Replace All
6. **Expected:** Otevře se modální replace preview dialog s per-file diff a checkboxy. Dialog je plně funkční (select all/deselect, potvrzení/zrušení).

### 9. Persistentní stav panelu

1. Proveď vyhledávání s výsledky
2. Stiskni Escape (panel se zavře)
3. Stiskni Ctrl+Shift+F (panel se znovu otevře)
4. **Expected:** Query text, togglery, a výsledky jsou zachovány. Panel se otevře ve stejném stavu jako před zavřením.

### 10. Panel resize

1. Otevři panel (Ctrl+Shift+F)
2. Najeď myší na horní okraj panelu
3. Táhni nahoru/dolů
4. **Expected:** Panel mění výšku. Minimální výška ~100px, maximální ~60% výšky okna. Editor se přizpůsobuje.

### 11. Ctrl+Shift+F toggle

1. Panel zavřený → Ctrl+Shift+F → panel se otevře
2. Panel otevřený → Ctrl+Shift+F → panel se zavře
3. **Expected:** Ctrl+Shift+F funguje jako toggle. Stav se přepíná.

### 12. Menu action

1. Otevři menu → Project Search (nebo ekvivalentní menu položka)
2. **Expected:** Panel se otevře (vždy otevírá, ne toggle). Query input dostane fokus.

## Edge Cases

### Prázdný dotaz

1. Nech query prázdný a stiskni Enter
2. **Expected:** Zobrazí se inline chyba "Prázdný vyhledávací dotaz" (nebo lokalizovaný ekvivalent). Hledání se nespustí.

### Hledání bez výsledků

1. Zadej query, který se nenachází v žádném souboru projektu
2. Stiskni Enter
3. **Expected:** Po dokončení hledání se zobrazí prázdný panel (žádné výsledky). Replace All button je neaktivní (disabled).

### Escape s otevřeným replace preview

1. Otevři panel → proveď hledání → spusť Replace All → preview dialog se otevře
2. Stiskni Escape
3. **Expected:** Escape zavře replace preview dialog, NE search panel. Panel zůstane otevřený.

### Kliknutí na výsledek z jiného souboru

1. Proveď hledání s výsledky z více souborů
2. Klikni na výsledek ze souboru, který není aktuálně otevřen
3. **Expected:** Soubor se otevře v novém tabu, editor jumpne na řádek. Panel zůstane otevřený.

### Opakované kliknutí na různé výsledky

1. Klikni na výsledek A → editor jumpne
2. Klikni na výsledek B → editor jumpne
3. **Expected:** Highlight se přesune z A na B. Editor správně přeskočí na nový řádek v novém souboru.

## Failure Signals

- Panel se nezobrazí pod editorem (layout pořadí chybné)
- Editor není editovatelný když je panel otevřený (panel překrývá/blokuje)
- Klik na výsledek neotevře soubor nebo nejumpne na řádek
- Klik na výsledek zavře panel
- Výsledky zmizí po close/reopen panelu
- Replace All button nereaguje nebo nespustí preview dialog
- Spinner se nezobrazí během hledání
- Escape zavře panel i když je otevřený replace preview dialog
- Ctrl+Shift+F neotevře/nezavře panel
- `cargo check` nebo `./check.sh` selhává

## Requirements Proved By This UAT

- R026 — TC 1 (panel pod editorem, editor editovatelný)
- R027 — TC 4 (klik → jump + fokus, panel zůstane otevřený)
- R028 — TC 9 (persistentní stav přes close/reopen)
- R029 — TC 11 (Ctrl+Shift+F toggle)
- R031 — TC 10 (resize tažením)
- R032 — TC 8 (replace flow z panelu → preview dialog)
- R033 — TC 1 (lokalizovaný heading), TC 12 (menu action)
- R034 — TC 2 (spinner + indikátor)
- R035 — TC 2 (per-file seskupení)
- R036 — TC 3 (match highlighting + kontextové řádky)

## Not Proven By This UAT

- R030 (In-file search regex togglery) — scope S02, ne S01
- Automatizované end-to-end testy — egui nemá headless UI testing framework, vizuální UAT je manuální
- Performance při velkých projektech (tisíce souborů) — nepokryto, ale search engine je beze změn z M005

## Notes for Tester

- Panel se otevírá prázdný při prvním spuštění — to je správné chování (žádné předchozí hledání).
- Replace preview dialog je modální (zakrývá editor) — toto je záměrné chování zachované z M005.
- Poll loop vyžaduje, aby projekt měl otevřený adresář — search prochází soubory v project root.
- egui resize handle na horním okraji panelu je subtilní — cursor se změní na resize cursor při najetí na správné místo.
