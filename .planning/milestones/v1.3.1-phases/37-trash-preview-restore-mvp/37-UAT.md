---
status: complete
phase: 37-trash-preview-restore-mvp
source:
  - 37-01-SUMMARY.md
  - 37-02-SUMMARY.md
  - 37-03-SUMMARY.md
  - 37-04-SUMMARY.md
started: 2026-03-12T13:10:17Z
updated: 2026-03-12T13:19:13Z
---

## Current Test

[testing complete]

## Tests

### 1. Otevření Trash Preview z menu
expected: V menu projektu je dostupná akce pro Trash Preview a po kliknutí se otevře modal bez freeze.
result: pass

### 2. Otevření Trash Preview z command palette
expected: Příkaz Trash Preview je dostupný v command palette a otevře stejný modal jako menu akce.
result: pass

### 3. Preview seznam + filtrování
expected: Modal zobrazuje položky (název/typ/čas/původní cesta) a textový filtr okamžitě zužuje seznam.
result: pass

### 4. Restore jedné položky bez konfliktu
expected: Obnovení vrátí soubor na původní cestu, položka zmizí z trash preview a zobrazí se success feedback.
result: pass

### 5. Konfliktní restore nabídne jen bezpečné volby
expected: Při existující cílové cestě se otevře konflikt modal pouze s volbami „Obnovit jako kopii“ a „Zrušit“ (bez overwrite).
result: pass

### 6. Restore jako kopie zachová původní soubor
expected: Volba „Obnovit jako kopii“ vytvoří novou kopii s odlišeným názvem a původní cílový soubor zůstane nedotčen.
result: pass

### 7. UI konzistence po restore
expected: Po úspěšném restore proběhne reload file tree, obnovená cesta je zvýrazněná/rozbalená a neotevře se automaticky nový editor tab.
result: pass

### 8. Lokalizační texty preview/restore flow
expected: Texty pro preview/restore/conflict/success/error jsou konzistentní a nechybí v aktivním jazyce UI.
result: pass

## Summary

total: 8
passed: 8
issues: 0
pending: 0
skipped: 0

## Gaps

none
