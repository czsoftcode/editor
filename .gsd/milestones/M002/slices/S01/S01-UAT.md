# S01: Snapshot Pipeline a Tab Context Menu — UAT

**Milestone:** M002
**Written:** 2026-03-13

## UAT Type

- UAT mode: live-runtime
- Why this mode is sufficient: Snapshot pipeline vyžaduje reálný save na disk a UI interakci (context menu, panel) — artefakty na FS a vizuální kontrola jsou jediný spolehlivý důkaz.

## Preconditions

- Editor je zkompilovaný a spuštěný (`cargo run` nebo release build).
- Otevřený workspace s alespoň jedním textovým souborem.
- Adresář `.polycredo/history/` existuje (vytvoří se automaticky při prvním snapshotu).

## Smoke Test

Otevřít textový soubor, přidat řádek, uložit (Ctrl+S). Ověřit `ls .polycredo/history/` — měl by existovat podadresář s jedním snapshot souborem.

## Test Cases

### 1. Tři uložení vytvoří tři snapshoty

1. Otevřít textový soubor v editoru.
2. Přidat řádek "verze 1", uložit (Ctrl+S).
3. Přidat řádek "verze 2", uložit (Ctrl+S).
4. Přidat řádek "verze 3", uložit (Ctrl+S).
5. **Expected:** V `.polycredo/history/<encoded_path>/` existují 3 soubory s příponou `.txt`. Každý má jiný hash v názvu.

### 2. Deduplikace — stejný obsah nevytváří nový snapshot

1. Uložit soubor bez jakékoliv změny (Ctrl+S).
2. **Expected:** Počet souborů v `.polycredo/history/<encoded_path>/` zůstane stejný jako před uložením.

### 3. Context menu na textovém tabu

1. Pravý klik na tab textového souboru.
2. **Expected:** Zobrazí se context menu s položkami "Historie souboru" (nebo lokalizovaný ekvivalent) a "Zavřít tab".

### 4. Context menu na binárním tabu

1. Otevřít binární soubor (obrázek, font).
2. Pravý klik na jeho tab.
3. **Expected:** Context menu obsahuje pouze "Zavřít tab". Položka "Historie souboru" chybí.

### 5. History panel — otevření a výběr verze

1. Mít soubor s alespoň 2 snapshoty (viz test case 1).
2. Pravý klik na tab → "Historie souboru".
3. **Expected:** Otevře se history panel s názvem souboru v nadpisu. Levý panel zobrazuje seznam verzí seřazený od nejnovější (s timestamp). Pravý panel je prázdný nebo zobrazuje nejnovější verzi.
4. Kliknout na starší verzi v seznamu.
5. **Expected:** Pravý panel zobrazí obsah vybrané verze v monospace fontu.

### 6. History panel — zavření

1. V otevřeném history panelu kliknout na zavírací tlačítko (✖ nebo "Zavřít").
2. **Expected:** History panel zmizí. Editor se vrátí do normálního režimu.

### 7. Toast při prázdné historii

1. Vytvořit nový soubor, neuložit ho (nebo ho uložit a smazat snapshot soubory ručně).
2. Pravý klik na tab → "Historie souboru".
3. **Expected:** Zobrazí se toast "Žádné historické verze" (nebo lokalizovaný ekvivalent). Panel se neotevře.

### 8. Autosave vytváří snapshoty

1. Zapnout autosave v nastavení (pokud není defaultně zapnutý).
2. Upravit soubor a počkat na autosave trigger.
3. **Expected:** Po autosave se v `.polycredo/history/<encoded_path>/` objeví nový snapshot.

## Edge Cases

### Binární soubor nevytváří snapshot

1. Otevřít binární soubor (obrázek).
2. Uložit ho (pokud editor umožňuje).
3. **Expected:** V `.polycredo/history/` nevznikne snapshot pro tento soubor.

### I/O chyba při snapshotování

1. Nastavit `.polycredo/history/` na read-only (`chmod -w .polycredo/history/`).
2. Upravit a uložit textový soubor.
3. **Expected:** Zobrazí se toast s chybovou hláškou obsahující cestu souboru a popis I/O chyby. Uložení souboru samotného proběhne úspěšně.
4. Obnovit oprávnění (`chmod +w .polycredo/history/`).

### I/O chyba při čtení snapshotu

1. Otevřít history panel pro soubor s verzemi.
2. Ručně smazat nebo poškodit jeden snapshot soubor na disku.
3. Kliknout na verzi odpovídající smazanému souboru.
4. **Expected:** V pravém panelu se zobrazí chybová hláška "Chyba čtení: ..." místo obsahu.

## Failure Signals

- Po uložení textového souboru neexistuje žádný snapshot v `.polycredo/history/`.
- Pravý klik na tab neukáže context menu.
- "Historie souboru" v menu pro binární tab.
- History panel se neotevře nebo nereaguje na výběr verze.
- I/O chyba při snapshotování nezobrazí toast.
- Chybí i18n překlady — zobrazuje se surový klíč místo textu.

## Not Proven By This UAT

- Split view s diff zvýrazněním (scope S02).
- Navigace šipkami mezi verzemi (scope S02).
- Cleanup starých verzí a retence limit (scope S03).
- Chování při zavření tabu v history mode (scope S03).
- Výkon diff algoritmu na velkých souborech (scope S02).

## Notes for Tester

- Timestamp v history panelu je v UTC, ne v lokálním čase — to je záměrné a bude případně řešeno později.
- Pre-existující failing test `phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols` nesouvisí s touto prací.
- History panel je zatím jednoduchý list + preview. Plný split view s diff barvami přijde v S02.
- Pro ověření snapshot souborů na disku: `find .polycredo/history/ -name "*.txt" | head -20`.
