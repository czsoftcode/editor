# S02: Obnovení historické verze s potvrzením a i18n — UAT

**Milestone:** M003
**Written:** 2026-03-13

## UAT Type

- UAT mode: mixed (artifact-driven + live-runtime)
- Why this mode is sufficient: i18n klíče ověřitelné grepem, restore flow vyžaduje vizuální potvrzení v běžícím editoru

## Preconditions

- Editor zkompilován a spuštěn (`cargo run`)
- Otevřen projekt se soubory, které mají alespoň 2 historické verze (provedena editace + uložení)
- Alespoň 1 soubor s jedinou verzí (nově vytvořený)

## Smoke Test

Otevřít history view na souboru s historií → vybrat historickou verzi → kliknout "Obnovit" → potvrdit v dialogu → obsah se zapíše do editoru, nová verze se objeví v seznamu na pozici 0.

## Test Cases

### 1. Tlačítko Obnovit — disabled stav

1. Otevřít history view na souboru s více verzemi
2. Nekliknut na žádnou historickou verzi (default stav — verze je automaticky vybraná pokud existuje)
3. Pokud je verze automaticky vybraná, tlačítko musí být enabled
4. **Expected:** Tlačítko "Obnovit" viditelné v toolbaru. Pokud žádná verze nevybraná, tlačítko je disabled (neaktivní).

### 2. Confirm dialog

1. Otevřít history view, vybrat historickou verzi
2. Kliknout na tlačítko "Obnovit"
3. **Expected:** Zobrazí se potvrzovací dialog s titulkem a textem o obnovení. Dvě tlačítka: OK a Zrušit.

### 3. Cancel restore

1. V confirm dialogu kliknout "Zrušit"
2. **Expected:** Dialog se zavře, žádná změna v editoru ani v historii.

### 4. Confirm restore — hlavní flow

1. Otevřít history view na souboru s ≥3 verzemi
2. Zapamatovat aktuální obsah levého panelu
3. Vybrat starší historickou verzi (ne nejnovější)
4. Kliknout "Obnovit" → potvrdit OK
5. **Expected:** 
   - Levý panel obsahuje obsah vybrané historické verze
   - Tab je označen jako modified (●)
   - V seznamu historie se objeví nová verze na pozici 0 (nejnovější)
   - Všechny předchozí verze zůstávají zachovány (append, ne replace)
   - selected_index ukazuje na novou verzi (pozice 0)

### 5. Restore na souboru s jedinou verzí

1. Otevřít history view na souboru s jedinou verzí
2. **Expected:** Pravý panel je prázdný, tlačítko "Obnovit" je disabled (žádná historická verze k obnovení).

### 6. i18n klíče — lokalizace

1. Přepnout jazyk editoru na každý z 5 jazyků (cs, en, sk, de, ru)
2. Otevřít history view s historií
3. Kliknout "Obnovit"
4. **Expected:** Tlačítko a dialog texty jsou ve správném jazyce pro každou lokalizaci.

## Edge Cases

### Restore při neuloženém obsahu v levém panelu

1. Otevřít history view, editovat v levém panelu
2. Kliknout "Obnovit" → potvrdit
3. **Expected:** Historický obsah přepíše editované změny v levém panelu. Tab je modified. Neuložená editace v levém panelu je nahrazena.

### Opakovaný restore

1. Provést restore jedné verze
2. Vybrat jinou historickou verzi
3. Provést restore znovu
4. **Expected:** Každý restore vytvoří nový snapshot. Historie obsahuje oba restore snapshoty.

## Failure Signals

- Tlačítko "Obnovit" není viditelné v toolbaru
- Klik na "Obnovit" nic neudělá (chybí confirm dialog)
- Po potvrzení se obsah nezmění
- Po restore chybí nový snapshot v historii
- Starší verze zmizely z historie (replace místo append)
- i18n klíče se zobrazují jako raw identifikátory (history-restore-btn)
- `[Restore]` chybové hlášky ve stderr

## Requirements Proved By This UAT

- R004 — Obnovení historické verze (append, ne replace): test cases 4, edge case "opakovaný restore"
- R005 — Potvrzovací dialog před obnovením: test cases 2, 3
- R008 — i18n klíče pro nové UI prvky: test case 6

## Not Proven By This UAT

- R001–R003, R006–R007, R009 — tyto požadavky byly implementovány a ověřeny v S01 (editovatelný panel, syntax highlighting, sync scroll, tab sync, výchozí stav, diff zvýraznění)
- Stresové testování restore na velmi velkých souborech (>10k řádků)
- Chování při chybě čtení historické verze z disku (vyžaduje simulaci I/O selhání)

## Notes for Tester

- Confirm dialog používá existující `show_modal()` pattern — vypadá stejně jako ostatní confirm dialogy v editoru.
- Pokud restore selže (chyba čtení), hledej `[Restore]` ve stderr — v GUI se chyba momentálně nezobrazuje (known limitation).
- Pre-existující selhání testu `phase35_delete_foundation_scope_guard` nesouvisí s tímto slice.
