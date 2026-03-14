# S01: Modal volby okna s guard flow a workspace reinicializací — UAT

**Milestone:** M007
**Written:** 2026-03-14

## UAT Type

- UAT mode: live-runtime
- Why this mode is sufficient: Celý flow je UI-driven (menu → modal → guard → reinicializace). Artifact-driven verifikace (cargo check, grep, ./check.sh) již prošla. Zbývá vizuální a interakční ověření v živém editoru.

## Preconditions

- Editor zkompilován a spuštěn (`cargo run` nebo release build)
- Existují alespoň 2 platné projektové složky na disku (pro přepnutí)
- Jeden otevřený projekt s alespoň jedním souborem
- Alespoň jeden nedávný projekt v menu Nedávné

## Smoke Test

1. Menu → Projekt → Otevřít projekt → vybrat složku
2. **Expected:** Zobrazí se modal s třemi tlačítky: "Nové okno" (výchozí/zvýrazněné), "Stávající okno", "Zrušit"

## Test Cases

### 1. Open Project → Nové okno

1. Menu → Projekt → Otevřít projekt
2. Vybrat existující složku v file dialogu
3. V modalu kliknout "Nové okno"
4. **Expected:** Nový viewport/okno se otevře s vybraným projektem. Původní okno zůstane beze změny.

### 2. Open Project → Stávající okno (bez dirty tabs)

1. Ujistit se, že žádný otevřený soubor nemá neuložené změny (žádná ● v tabu)
2. Menu → Projekt → Otevřít projekt
3. Vybrat jinou složku
4. V modalu kliknout "Stávající okno"
5. **Expected:** Stávající okno se přepne na nový projekt. Starý projekt zmizí — nový strom souborů, nové tagy, terminály starého projektu ukončeny.

### 3. Open Project → Stávající okno (s dirty tabs) → Save

1. Otevřít soubor a provést editaci (tab ukáže ●)
2. Menu → Projekt → Otevřít projekt → vybrat jinou složku
3. V modalu kliknout "Stávající okno"
4. **Expected:** Zobrazí se unsaved changes guard dialog (Save / Discard / Cancel)
5. Kliknout "Uložit"
6. **Expected:** Soubor se uloží (● zmizí), poté se projekt přepne na nový.

### 4. Open Project → Stávající okno (s dirty tabs) → Discard

1. Otevřít soubor a provést editaci
2. Menu → Projekt → Otevřít projekt → vybrat jinou složku → "Stávající okno"
3. V guard dialogu kliknout "Zahodit"
4. **Expected:** Projekt se přepne bez uložení. Editace ztracena. Nový projekt otevřen.

### 5. Open Project → Stávající okno (s dirty tabs) → Cancel

1. Otevřít soubor a provést editaci
2. Menu → Projekt → Otevřít projekt → vybrat jinou složku → "Stávající okno"
3. V guard dialogu kliknout "Zrušit"
4. **Expected:** Nic se nestane. Modal zmizí. Stávající projekt zůstane beze změny. Editovaný soubor stále dirty.

### 6. Open Project → Zrušit

1. Menu → Projekt → Otevřít projekt → vybrat složku
2. V modalu kliknout "Zrušit"
3. **Expected:** Modal zmizí. Žádná změna — stávající projekt zůstane otevřený.

### 7. Nedávné projekty → modal

1. Menu → Nedávné → kliknout na projekt
2. **Expected:** Zobrazí se stejný modal (Nové okno / Stávající okno / Zrušit)
3. Kliknout "Nové okno"
4. **Expected:** Nový viewport s vybraným projektem

### 8. Nedávné projekty → stávající okno

1. Menu → Nedávné → kliknout na projekt
2. Kliknout "Stávající okno" (bez dirty tabs)
3. **Expected:** Projekt se přepne ve stávajícím okně

### 9. Nový projekt wizard → modal

1. Menu → Projekt → Nový projekt
2. Projít wizard (zadat název, vybrat lokaci, potvrdit)
3. **Expected:** Po dokončení wizardu se zobrazí modal (Nové okno / Stávající okno / Zrušit)
4. Kliknout "Nové okno"
5. **Expected:** Nový viewport s čerstvě vytvořeným projektem

### 10. i18n ověření

1. Přepnout jazyk editoru na angličtinu (Settings → Language → English)
2. Menu → Projekt → Otevřít projekt → vybrat složku
3. **Expected:** Modal zobrazuje anglické texty: "Open Project", "Where to open the project?", "New Window", "Current Window", "Cancel"
4. Přepnout na němčinu a ověřit německé texty
5. **Expected:** "Projekt öffnen", "Neues Fenster", "Aktuelles Fenster", "Abbrechen"

## Edge Cases

### Escape zavře modal

1. Menu → Projekt → Otevřít projekt → vybrat složku (modal se zobrazí)
2. Stisknout Escape
3. **Expected:** Modal zmizí. Žádná akce. Stávající projekt beze změny.

### Klik mimo modal (backdrop)

1. Menu → Projekt → Otevřít projekt → vybrat složku (modal se zobrazí)
2. Kliknout mimo modal (na pozadí / backdrop)
3. **Expected:** Modal se zavře (Cancel behavior). Žádná akce.

### Rychlé dvojkliknutí na nedávný projekt

1. Menu → Nedávné → rychle dvakrát kliknout na projekt
2. **Expected:** Modal se zobrazí jednou (ne dvakrát). Druhý klik nahradí cestu v pending_open_choice.

### Guard flow kolize

1. Editovat soubor (dirty tab)
2. Menu → Projekt → Otevřít projekt → vybrat složku → "Stávající okno"
3. Guard dialog se zobrazí
4. **Expected:** Nelze otevřít nový open choice modal přes guard dialog (guard má prioritu). UI zůstane na guard dialogu.

### Editor zamčený během modalu

1. Menu → Projekt → Otevřít projekt → vybrat složku (modal se zobrazí)
2. Zkusit psát do editoru
3. **Expected:** Editor nepřijímá vstup — modal blokuje interakci s pozadím.

## Failure Signals

- Modal se nezobrazí po výběru složky / recent / wizard → `pending_open_choice` se nenastavuje
- "Stávající okno" neudělá nic → `open_here_path` se nenastavuje, chybí post-guard path
- Guard dialog se nezobrazí při dirty tabs → `SwitchProject` mode chybí nebo se nekontrolují dirty tabs
- Po Cancel v guard se modal znovu zobrazí → `pending_open_choice` se nečistí
- Starý terminál běží po přepnutí projektu → Drop cleanup nefunguje
- Texty v modalu jsou hardcoded česky → i18n napojení selhalo
- Crash při Escape v modalu → `should_close()` handling chybí
- `cargo check` nebo `./check.sh` selhává → compile error nebo regrese

## Requirements Proved By This UAT

- R037 — Test 1, 2, 6: Modal se zobrazí po Open Project s třemi tlačítky
- R038 — Test 9: Modal se zobrazí po wizard dokončení
- R039 — Test 7, 8: Modal se zobrazí po kliknutí na nedávný projekt
- R040 — Test 3, 4, 5: Unsaved guard při dirty tabs, Save/Discard/Cancel cesty
- R041 — Test 2, 4, 8: Workspace reinicializace (starý ws nahrazen novým)
- R042 — Test 10: i18n texty v angličtině a němčině
- R043 — Test 2: Terminály starého projektu ukončeny při přepnutí (pozorovat v process listu)

## Not Proven By This UAT

- Automatizované unit/integration testy pro guard flow s SwitchProject — pokryto pouze cargo check + existující guard testy pro WorkspaceClose mode
- Secondary viewport (deferred viewport) chování — testováno jen na root workspace
- Ruština a slovenština i18n — ověřeny grep kontrolou, ne vizuálně

## Notes for Tester

- "Stávající okno" provede kompletní reinicializaci — všechny otevřené tagy, terminály a watchers starého projektu se zruší. To je expected behavior, ne bug.
- Výchozí (zvýrazněné) tlačítko je "Nové okno" — to je záměr pro bezpečnost (nemodifikuje stávající workspace).
- Guard dialog je identický s existujícím unsaved changes guardem při zavírání workspace — ověřit, že texty a chování odpovídají.
