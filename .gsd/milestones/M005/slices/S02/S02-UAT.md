# S02: Project-wide replace s preview a local history — UAT

**Milestone:** M005
**Written:** 2026-03-13

## UAT Type

- UAT mode: live-runtime
- Why this mode is sufficient: Replace modifikuje soubory na disku a interaguje s local history — musí se ověřit v běžícím editoru s reálným projektem.

## Preconditions

- Editor spuštěn s otevřeným projektem obsahujícím ≥3 `.rs` souborů
- Local history funkční (`.polycredo_editor/history/` existuje)
- Alespoň jeden soubor obsahuje text "old_name" na ≥2 místech
- Alespoň jeden jiný soubor obsahuje "old_name" na ≥1 místě
- Žádné neuložené změny v otevřených tabech

## Smoke Test

1. Ctrl+Shift+F → search dialog se otevře
2. Vedle regex/case/word togglerů je vidět ↔ (replace toggle) button
3. Kliknutí na ↔ → pod query inputem se zobrazí replace input pole
4. **Expected:** Replace input je viditelný, dialog heading se změní na "Nahradit v projektu"

## Test Cases

### 1. Basic replace flow — happy path

1. Ctrl+Shift+F → search dialog
2. Zadat query "old_name" (plain text, regex OFF)
3. Kliknout ↔ toggle → zobrazí se replace input
4. Zadat replace text "new_name"
5. Kliknout "Replace All" (nebo odpovídající lokalizovaný button)
6. **Expected:** Otevře se replace preview dialog s per-file sekcemi
7. Ověřit: každá sekce má checkbox (zaškrtnutý), filename, match count
8. Rozbalit sekci — vidět inline diff: "old_name" červeně, "new_name" zeleně
9. Kliknout "Potvrdit" (Confirm)
10. **Expected:** Preview dialog se zavře. Toast zobrazí "Nahrazeno v N souborech". Soubory na disku obsahují "new_name" místo "old_name".

### 2. Selective replace — odškrtnutí souboru

1. Opakovat kroky 1–6 z testu 1 (s jiným query, např. "TODO")
2. V preview dialogu odškrtnout checkbox jednoho souboru
3. Kliknout "Potvrdit"
4. **Expected:** Odškrtnutý soubor zůstává nezměněn na disku. Ostatní soubory modifikovány. Toast reflektuje počet skutečně modifikovaných souborů.

### 3. Select All / Deselect All

1. Otevřít replace preview dialog (kroky 1–6 z testu 1)
2. Kliknout "Deselect All"
3. **Expected:** Všechny checkboxy odškrtnuty. Selection counter ukazuje "0 z N vybráno".
4. Kliknout "Select All"
5. **Expected:** Všechny checkboxy zaškrtnuty. Selection counter ukazuje "N z N vybráno".

### 4. Local history snapshot existuje po replace

1. Provést replace (test 1 kroky 1–10)
2. Otevřít modifikovaný soubor v editoru
3. Otevřít history view (Ctrl+H nebo přes menu)
4. **Expected:** V history listu existuje snapshot s timestampem odpovídajícím replace operaci. Snapshot obsahuje obsah **před** replace (original_content).

### 5. Regex replace s capture groups

1. Ctrl+Shift+F, zapnout regex toggle (•*)
2. Query: `fn\s+(\w+)` (matchuje funkce)
3. Replace: `fn renamed_$1`
4. Spustit replace flow
5. **Expected:** Preview zobrazuje správné nahrazení — `fn foo` → `fn renamed_foo`, `fn bar` → `fn renamed_bar`. Capture group $1 se expanduje na skutečný název funkce.

### 6. Tab refresh po replace

1. Otevřít soubor "example.rs" v tabu
2. Ověřit, že tab nemá indikátor neuložených změn (●)
3. Provést replace, který modifikuje "example.rs"
4. **Expected:** Obsah tabu "example.rs" se automaticky aktualizuje na nový obsah. Tab nemá ● indikátor (modified = false). Obsah odpovídá souboru na disku.

### 7. Cancel replace preview

1. Otevřít replace preview dialog (kroky 1–6 z testu 1)
2. Kliknout "Zrušit" (Cancel)
3. **Expected:** Preview dialog se zavře. Žádné soubory na disku nebyly modifikovány. Žádný toast.

### 8. i18n — replace klíče ve všech jazycích

1. Přepnout jazyk editoru na každý z 5 jazyků (cs, en, sk, de, ru)
2. Otevřít replace preview dialog
3. **Expected:** Všechny texty (nadpis, tlačítka Potvrdit/Zrušit, Select All/Deselect All, selection counter, success toast) jsou přeloženy. Žádný hardcoded anglický text.

## Edge Cases

### Replace s prázdným výsledkem (no matches)

1. Zadat query, který nemá žádné matche v projektu
2. Pokusit se spustit replace
3. **Expected:** Buď "Replace All" button je disabled, nebo preview dialog se otevře s prázdným seznamem a informativní hláškou.

### Replace s nevalidním regex

1. Zapnout regex toggle
2. Zadat nevalidní regex (např. `[invalid`)
3. **Expected:** Inline chyba pod inputem (ze S01). Replace se nespustí. "Replace All" button nedostupný nebo nefunkční.

### Snapshot selhání (disk full / permission denied)

1. Simulovat: nastavit `.polycredo_editor/history/` na read-only (`chmod 555`)
2. Provést replace
3. **Expected:** Toast s chybovou zprávou obsahující název souboru a popis snapshot chyby. Soubor NEBYL modifikován (write se přeskočí). Ostatní soubory s funkčním snapshotem se modifikují.

### Write selhání (read-only soubor)

1. Nastavit jeden cílový soubor na read-only (`chmod 444`)
2. Provést replace zahrnující tento i jiné soubory
3. **Expected:** Toast s chybovou zprávou pro read-only soubor (obsahuje název souboru). Ostatní soubory modifikovány úspěšně. Summary toast zobrazuje partial failure ("N z M souborů, K chyb").

### Soubory beze změny (query == replace text)

1. Zadat query "foo" a replace text "foo" (identický)
2. Spustit replace
3. **Expected:** compute_replace_previews buď vyfiltruje soubory kde original == new content, nebo preview dialog zobrazuje 0 změn. Žádný soubor se nezapíše.

## Failure Signals

- Replace preview dialog se neotevře po kliknutí "Replace All"
- Checkboxy v preview nereagují na kliknutí
- Soubory se modifikují i přes odškrtnutý checkbox
- Chybí local history snapshot po replace
- Tab neukazuje aktualizovaný obsah po replace
- Tab má ● indikátor po replace (falešný modified flag)
- Toast se nezobrazí po replace (ani success, ani error)
- Hardcoded anglický text v ne-anglickém jazyce
- Selhání jednoho souboru zastaví replace ostatních

## Requirements Proved By This UAT

- R020 — Test 1, 2, 3, 7: replace preview s checkboxy, selective replace, cancel
- R022 — Edge case snapshot/write selhání: per-file toast, replace pokračuje
- R023 — Test 4: local history snapshot existuje s original obsahem
- R024 — Test 8: i18n klíče pro replace UI ve všech 5 jazycích

## Not Proven By This UAT

- Vizuální kvalita diff renderingu (barvy, fonty, layout) — vyžaduje subjektivní posouzení
- Performance replace na 100+ souborech — vyžaduje benchmark s velkým projektem
- Capture group rozšíření v edge cases (nested groups, backreferences) — unit testy pokrývají základní fungování

## Notes for Tester

- Replace modifikuje soubory na disku — **před testováním vytvořit git snapshot nebo zálohu projektu**
- Po testu 1 je potřeba vrátit "old_name" zpět pro opakování dalších testů (git checkout)
- Snapshot selhání test vyžaduje manuální chmod — po testu vrátit permissions zpět
- Preview dialog používá collapsing headers — pro ≤5 souborů jsou defaultně rozbalené, pro >5 collapsed
