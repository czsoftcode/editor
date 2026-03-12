---
status: complete
phase: 36-safe-move-to-trash-engine
source:
  - .planning/phases/36-safe-move-to-trash-engine/36-01-SUMMARY.md
  - .planning/phases/36-safe-move-to-trash-engine/36-02-SUMMARY.md
  - .planning/phases/36-safe-move-to-trash-engine/36-03-SUMMARY.md
started: 2026-03-12T10:07:21Z
updated: 2026-03-12T10:13:46Z
---

## Current Test

[testing complete]

## Tests

### 1. Smazání souboru přesune položku do koše
expected: Ve stromu vyber běžný soubor mimo `.polycredo/trash` a spusť delete. Soubor zmizí z původního umístění, v UI nevznikne hard-delete efekt, a operace skončí bez pádu aplikace.
result: pass

### 2. Smazání adresáře přesune celý adresář do koše
expected: Smaž adresář mimo `.polycredo/trash`. Adresář zmizí z původního místa stejně jako souborový delete flow, bez pádu a bez tichého selhání.
result: pass

### 3. Guard blokuje mazání interního koše
expected: Pokus o delete položky uvnitř `.polycredo/trash` je zablokován. Aplikace neprovede mazání interního trash prostoru a uživatel dostane srozumitelnou chybu.
result: skipped
reason: skip, .polycredo neni z bezpecnostnich duvodu ve stromu pristupno

### 4. Chyba přesunu do koše se ukáže jako lokalizovaný toast
expected: Vyvolej chybu přesunu do koše (např. práva/simulovaná chyba). Zobrazí se toast se srozumitelným důvodem a doporučeným dalším krokem (ne syrový interní error spam).
result: skipped
reason: skip

### 5. Delete tok zůstává responsivní a fail-closed
expected: Během delete toku UI nezamrzne (lze dál klikat/scrollovat). Pokud delete selže, data zůstávají na původním místě (fail-closed), nedojde k nechtěnému hard delete.
result: pass

## Summary

total: 5
passed: 3
issues: 0
pending: 0
skipped: 2

## Gaps

[]
