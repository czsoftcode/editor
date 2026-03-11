---
status: complete
phase: 35-trash-foundation-async-safety
source:
  - 35-01-SUMMARY.md
  - 35-02-SUMMARY.md
  - 35-03-SUMMARY.md
started: 2026-03-11T23:27:24Z
updated: 2026-03-12T00:04:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Smazani souboru presune polozku do trash
expected: Ve file tree smazej existujici soubor; soubor zmizi z puvodni slozky a objevi se v `.polycredo/trash`.
result: pass

### 2. On-demand vytvoreni `.polycredo/trash`
expected: Pri prvnim delete v projektu bez trash adresare se `.polycredo/trash` automaticky vytvori a operace dobehne bez padu UI.
result: pass

### 3. Fail-closed pri chybe move/create
expected: Kdyz delete narazi na chybu I/O (napr. konflikt cesty), soubor zustane na puvodnim miste a aplikace zobrazi error toast.
result: skipped

### 4. UI zustava responsivni pri delete
expected: Behem delete operace lze dal klikat/scrollovat/pisovat; UI nezamrzne a po dokonceni se stav korektne obnovi.
result: pass

### 5. Delete flow nepouziva hard-delete fallback
expected: Pri neuspesnem presunu do trash se soubor nesmaze natvrdo; uzivatel vidi chybu a data zustanou zachovana.
result: skipped

## Summary

total: 5
passed: 3
issues: 0
pending: 1
pending: 0
skipped: 2

## Gaps

none
