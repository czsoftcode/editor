# S03: Cleanup, Edge Cases a Finální Integrace — UAT

**Milestone:** M002
**Written:** 2026-03-13

## UAT Type

- UAT mode: mixed (artifact-driven + live-runtime)
- Why this mode is sufficient: Cleanup retence a i18n kompletnost ověřitelné automatizovaně (testy, grep). Edge case zavření tabu a celkový end-to-end flow vyžaduje běžící editor.

## Preconditions

- `cargo build` prošel bez chyb
- Workspace s existujícím `.polycredo/history/` adresářem (vytvoří se automaticky po prvním uložení)
- Editor spuštěn s `cargo run`

## Smoke Test

Spustit editor s workspace → uložit soubor 3× s různým obsahem → pravý klik na tab → "Historie souboru" → zobrazí se split view s diff → zavřít split view → editor v normálním režimu.

## Test Cases

### 1. Background cleanup při startu

1. Vytvořit workspace s `.polycredo/history/<rel_path>/` obsahujícím 55 snapshotu (5 nad limit).
2. Spustit editor s tímto workspace.
3. Počkat 2-3 sekundy.
4. **Expected:** V `.polycredo/history/<rel_path>/` zůstane max 50 nejnovějších snapshotů. Starší jsou smazány.

### 2. Cleanup mazání starých verzí (max_age)

1. Vytvořit workspace s `.polycredo/history/<rel_path>/` obsahujícím snapshot starší 30 dní (ručně přejmenovat timestamp v názvu souboru).
2. Spustit editor.
3. **Expected:** Snapshot starší 30 dní je smazán. Novější snapshoty zůstávají.

### 3. Zavření tabu v history mode (clean close)

1. Otevřít soubor, uložit ho 3×.
2. Pravý klik na tab → "Historie souboru" → split view se zobrazí.
3. Pravý klik na tab → "Zavřít tab".
4. **Expected:** Tab se zavře, editor se vrátí do normálního režimu. Žádné relikty history view v UI.

### 4. Zavření tabu v history mode (dirty close)

1. Otevřít soubor, uložit, upravit (neuložit).
2. Pravý klik na tab → "Historie souboru" → split view se zobrazí.
3. Pravý klik na tab → "Zavřít tab".
4. V dialogu neuloženích změn zvolit "Zahodit".
5. **Expected:** Tab se zavře, editor v normálním režimu. Žádné relikty.

### 5. End-to-end flow (milestoneový UAT scénář)

1. Otevřít soubor, uložit 3× s různým obsahem.
2. Ověřit na FS: `ls .polycredo/history/` — 3 snapshoty existují.
3. Pravý klik na tab → "Historie souboru".
4. Split view: aktuální verze vlevo, historická vpravo s diff zvýrazněním.
5. Kliknout šipky pro navigaci mezi verzemi — diff se aktualizuje.
6. Zavřít history view.
7. **Expected:** Editor v normálním režimu, žádné artefakty.

### 6. Binární soubor nespouští snapshot

1. Otevřít binární soubor (obrázek, font) v editoru.
2. Uložit.
3. **Expected:** Žádný snapshot v `.polycredo/history/` pro binární soubor.

## Edge Cases

### Zavření posledního tabu v history mode

1. Mít otevřený jen 1 tab. Otevřít history view.
2. Zavřít tab.
3. **Expected:** Editor korektně přejde do stavu bez tabů, žádný crash.

### History view na soubor bez snapshotů

1. Otevřít nový soubor (nikdy neuložený).
2. Pravý klik na tab → "Historie souboru".
3. **Expected:** Panel zobrazí hlášku "Žádné historické verze" (nebo ekvivalent v aktuálním jazyce).

## Failure Signals

- Crash při zavření tabu v history mode
- History view zůstane viset po zavření zdrojového tabu
- Cleanup nesmaže staré snapshoty (ověřitelné přes `ls -la .polycredo/history/*/`)
- Chybějící i18n klíč → zobrazí se raw klíč místo překladu
- `cargo test -- local_history` neprochází

## Not Proven By This UAT

- Výkon cleanup nad tisíci snapshotů (nepraktické ručně testovat)
- Paralelní přístup k `.polycredo/history/` z více instancí editoru
- Watcher filtr pro `.polycredo/` adresář (ověřen v S01, ne opakovaně)

## Notes for Tester

- Background cleanup je fire-and-forget — log výstup není viditelný v UI. Ověření pouze přes filesystem.
- Preexistující test `phase35_delete_foundation_scope_guard` selhává — nesouvisí s M002, ignorovat.
- i18n klíče jsou ověřeny automatizovaně (grep audit). Vizuální kontrola překladů je bonus, ne requirement.
