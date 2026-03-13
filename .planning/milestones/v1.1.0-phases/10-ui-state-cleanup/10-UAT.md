---
status: diagnosed
phase: 10-ui-state-cleanup
source: 10-01-SUMMARY.md
started: 2026-03-06T00:23:00Z
updated: 2026-03-06T00:30:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Settings dialog bez sandbox prvků
expected: Otevřete Settings dialog. NESMÍ obsahovat sandbox checkbox, sandbox tooltip ani sandbox inline poznámku/hint. Ostatní nastavení zobrazena normálně.
result: pass

### 2. Build bar bez Terminal mode labelu
expected: Podívejte se na build bar (spodní panel). NENÍ tam žádný "Terminal" mode label se sandbox hover textem. Bar zobrazuje pouze relevantní build/compile/git tlačítka.
result: issue
reported: "je tam jenom jeden separator navic, jak byl smazany Terminal | Sandbox"
severity: minor

### 3. File tree zobrazuje počet řádků u souborů
expected: Ve file tree jsou u souborů zobrazeny počty řádků. Soubory s 500+ řádky jsou vizuálně odlišeny (large file highlighting). Tato funkce funguje pro VŠECHNY soubory, ne jen sandbox.
result: issue
reported: "jenom mala pripominka, nedavej tam barvu, jenom podtrzeni a pocet radku, kvuli git barvam"
severity: cosmetic

### 4. File tree bez sandbox toggle
expected: File tree panel NEMÁ žádný "Sandbox" toggle button. Hlavička panelu zobrazuje název projektu bez "(Sandbox)" suffixu.
result: pass

### 5. Gitignore filtr bez sandbox
expected: Semantic indexer (pokud aktivní) nefiltruje adresáře s názvem "sandbox". Adresáře pojmenované "sandbox" v projektu se normálně zobrazují ve file tree a jsou indexovány.
result: skipped

## Summary

total: 5
passed: 2
issues: 2
pending: 0
skipped: 1

## Gaps

- truth: "Build bar nezobrazuje Terminal mode label se sandbox hover"
  status: failed
  reason: "User reported: je tam jenom jeden separator navic, jak byl smazany Terminal | Sandbox"
  severity: minor
  test: 2
  root_cause: "Duplicitní ui.separator() na řádku 14 v build_bar.rs — zůstal po smazání Terminal mode labelu mezi dvěma separátory"
  artifacts:
    - path: "src/app/ui/terminal/bottom/build_bar.rs"
      issue: "Řádek 14: duplicitní ui.separator() — smazat tento řádek"
  missing:
    - "Odstranit řádek 14 (ui.separator()) z build_bar.rs"
  debug_session: ""

- truth: "Soubory s 500+ řádky vizuálně odlišeny (large file highlighting)"
  status: failed
  reason: "User reported: jenom mala pripominka, nedavej tam barvu, jenom podtrzeni a pocet radku, kvuli git barvam"
  severity: cosmetic
  test: 3
  root_cause: "render.rs řádky 133-137 přepisují file_color na bílou/fialovou pro large files, čímž se přepíše git status barva"
  artifacts:
    - path: "src/app/ui/file_tree/render.rs"
      issue: "Řádky 133-137: file_color se přepisuje pro large files — odstranit přepis barvy, ponechat pouze underline + počet řádků"
  missing:
    - "Odstranit blok file_color = if visuals.dark_mode { ... } pro large files"
    - "Zachovat underline stroke ale použít původní file_color (z git statusu)"
  debug_session: ""
