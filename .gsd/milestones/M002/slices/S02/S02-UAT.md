# S02: History Split View s Diff a Navigací — UAT

**Milestone:** M002
**Written:** 2026-03-13

## UAT Type

- UAT mode: mixed (artifact-driven + human-experience)
- Why this mode is sufficient: Diff logika a barvy jsou pokryté unit testy (artifact-driven). Split view layout, vizuální diff zvýraznění a UX navigace šipkami vyžadují vizuální kontrolu v běžícím GUI editoru (human-experience).

## Preconditions

- Editor zkompilován a spuštěn (`cargo run`).
- Otevřen workspace s alespoň jedním textovým souborem.
- Soubor uložen minimálně 3× s různým obsahem (aby existovaly historické verze v `.polycredo/history/`).

## Smoke Test

Pravý klik na tab → "Historie souboru" → zobrazí se split view se dvěma panely vedle sebe (ne jednoduchý list+preview z S01). Zavírací ✕ vrátí editor do normálu.

## Test Cases

### 1. Split view se zobrazí správně

1. Otevřít textový soubor s historií (min. 2 verze).
2. Pravý klik na tab → kliknout "Historie souboru".
3. **Expected:** Zobrazí se toolbar nahoře (název souboru, info o verzi, šipky ← →, ✕) a dva panely pod ním — levý "Aktuální verze", pravý "Historická verze". Editor (textarea) se nekreslí.

### 2. Diff zvýraznění je viditelné

1. V otevřeném split view zkontrolovat oba panely.
2. **Expected:** Řádky přidané v aktuální verzi (oproti historické) jsou zvýrazněné zeleně v levém panelu. Řádky přítomné v historické verzi, ale ne v aktuální, jsou zvýrazněné červeně v pravém panelu. Společné řádky jsou bez barvy.

### 3. Navigace šipkami funguje

1. V toolbaru kliknout ← (starší verze).
2. **Expected:** Pravý panel se aktualizuje na starší verzi. Timestamp/info se změní. Diff zvýraznění se přepočítá.
3. Kliknout → (novější verze).
4. **Expected:** Pravý panel se vrátí na novější verzi. Diff se aktualizuje.

### 4. Šipky mají správný disabled stav

1. Navigovat na nejstarší verzi (opakovaně ←).
2. **Expected:** ← šipka je disabled (nelze kliknout dál do minulosti).
3. Navigovat na nejnovější verzi (opakovaně →).
4. **Expected:** → šipka je disabled.

### 5. Zavření vrátí normální editor

1. V toolbaru kliknout ✕.
2. **Expected:** Split view zmizí. Normální editor se znovu zobrazí s původním obsahem tabu. Žádné vizuální artefakty.

### 6. Resize handle funguje

1. V otevřeném split view přetáhnout svislý resize handle mezi panely.
2. **Expected:** Poměr šířek levého a pravého panelu se změní plynule.

### 7. Dark/light mode barvy

1. V otevřeném split view přepnout mezi dark a light mode (Settings).
2. **Expected:** Diff barvy se přizpůsobí — v dark mode semitransparentní zelená/červená, v light mode opaque zelená/červená s vyšším kontrastem. Text zůstává čitelný v obou režimech.

## Edge Cases

### Soubor bez historie

1. Vytvořit nový soubor, uložit jednou (1 verze).
2. Pravý klik → "Historie souboru".
3. **Expected:** Split view se zobrazí, ale navigační šipky jsou obě disabled (pouze 1 verze). Diff neukazuje žádné změny (aktuální == historická).

### Velmi velký soubor

1. Otevřít soubor s 1000+ řádky a historií.
2. Otevřít history view, navigovat mezi verzemi.
3. **Expected:** Diff se přepočítá bez viditelného zamrznutí UI (cachování funguje). ScrollArea panely scrollují plynule.

## Failure Signals

- Split view se nezobrazí po kliknutí na "Historie souboru" — chybí render_history_split_view() volání.
- Editor textarea je viditelná současně se split view — podmínka history_view.is_none() nefunguje.
- Diff barvy chybí nebo jsou nesprávné (zelená/červená prohozená) — chyba v DiffColors nebo ChangeTag mapování.
- Šipky nereagují na kliknutí — chybí .clicked() handler nebo disabled stav je špatně.
- Po zavření split view zůstává prázdný prostor — history_view se nenastavil na None.
- Toast s chybou čtení snapshot obsahu — I/O problém při načítání historické verze (funkční, ale indikuje FS problém).

## Not Proven By This UAT

- Cleanup retence (50 verzí, 30 dní) — scope S03.
- Zavření tabu v history mode — edge case pro S03.
- History mode na posledním tabu — edge case pro S03.
- Watcher filtr pro `.polycredo/` adresář — verifikace v S03.
- Synchronizovaný scroll dvou panelů — záměrně mimo scope (nezávislý scroll).

## Notes for Tester

- Preexistující selhání testu `phase35_delete_foundation_scope_guard_has_no_restore_foundation_symbols` v `./check.sh` je známé a nesouvisí s S02 — hledá odstraněný plánovací soubor z v1.3.1.
- Diff rendering zobrazuje celý soubor v obou panelech (ne jen změněné řádky) — je to záměrný side-by-side diff, ne unified diff.
- `on_hover_text()` na šipkách je voláno vždy, ale tooltip se zobrazí jen při hoveru — normální egui chování.
