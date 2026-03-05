---
status: complete
phase: 02-terminal-git-barvy
source:
  - 02-01-SUMMARY.md
  - 02-02-SUMMARY.md
started: 2026-03-04T20:46:51Z
updated: 2026-03-04T20:53:42Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Runtime theme switch v Claude terminálu
expected: V Claude terminálu se po přepnutí dark/light ihned změní barvy (v light mode světlé pozadí), aniž by se přerušil běžící proces.
result: pass

### 2. Runtime theme switch v Build terminálu
expected: V Build terminálu se po přepnutí dark/light ihned změní barvy stejně jako v Claude terminálu a běžící příkaz pokračuje bez restartu.
result: pass

### 3. Scrollbar terminálu odpovídá aktivnímu tématu
expected: Scrollbar v terminálu není tmavě hardcoded v light mode; track/thumb odpovídají aktivnímu tématu a při hover/drag je thumb kontrastnější.
result: pass

### 4. Unfocused terminal overlay je čitelný v light mode
expected: Když terminál není fokusovaný, overlay/cursor prvky zůstávají čitelné i v light mode (bez tmavých artefaktů).
result: pass

### 5. Git statusy M/A/D/?? jsou čitelné v light mode
expected: Ve file tree jsou stavy Modified, Added, Deleted a Untracked v light mode vzájemně rozlišitelné a čitelné.
result: pass

### 6. Untracked (??) je v light mode jasně viditelný
expected: Untracked soubory (`??`) nejsou v light mode zašedlé do nečitelna; barva je zřetelná na světlém pozadí.
result: skipped
reason: user skipped

## Summary

total: 6
passed: 5
issues: 0
pending: 0
skipped: 1

## Gaps

[none yet]
