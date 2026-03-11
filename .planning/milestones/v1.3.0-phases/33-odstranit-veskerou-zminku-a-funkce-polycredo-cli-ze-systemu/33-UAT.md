---
status: complete
phase: 33-odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu
source:
  - 33-01-SUMMARY.md
  - 33-02-SUMMARY.md
  - 33-03-SUMMARY.md
  - 33-04-SUMMARY.md
started: 2026-03-11T20:32:51Z
updated: 2026-03-11T20:38:29Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Levý panel bez CLI baru
expected: V levém panelu (strom souborů + spodní terminál) už není žádný CLI/AI quick-launch bar s tlačítky typu Start/Settings.
result: pass

### 2. Nastavení AI obsahuje jen správu asistentů
expected: V Nastavení > AI není URL/API key/model/top-p/top-k/seed/blacklist; zůstává jen seznam asistentů (název, příkaz, parametry, přidat/odebrat).
result: pass

### 3. AI launcher stále odesílá do aktivního terminálu
expected: Po zvolení asistenta a spuštění v AI panelu se příkaz odešle do aktivního terminal tabu (launcher-only tok).
result: pass

### 4. Legacy AI chat runtime entrypointy nejsou dostupné
expected: V UI není přepnutí/otevření starého AI chatu/runtime panelu; aplikace běží bez těchto entrypointů.
result: pass

### 5. Stabilita po cleanupu
expected: Běžná práce (otevření projektu, strom souborů, terminál) funguje bez regresí a bez nových chybových hlášek.
result: pass

## Summary

total: 5
passed: 5
issues: 0
pending: 0
skipped: 0

## Gaps

none
