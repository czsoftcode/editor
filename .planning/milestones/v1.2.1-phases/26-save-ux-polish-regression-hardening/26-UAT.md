---
status: complete
phase: 26-save-ux-polish-regression-hardening
source:
  - 26-01-SUMMARY.md
  - 26-02-SUMMARY.md
  - 26-03-SUMMARY.md
  - 26-04-SUMMARY.md
started: 2026-03-10T19:50:00Z
updated: 2026-03-10T19:57:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Status bar ukazuje runtime save mode
expected: Po uložení settings (Apply) se ve status baru zobrazí "Manual" nebo "Auto" podle runtime hodnoty, ne podle settings draftu.
result: pass

### 2. Aktivní tab má mode marker
expected: Když je aktivní tab dirty (má neuložené změny) a projekt je v Manual nebo Auto režimu, v záložce se zobrazí "●·M" nebo "●·A" (dirty symbol + mode marker).
result: pass

### 3. Dirty je vizuálně primární
expected: Dirty symbol "●" zůstává v tab labelu před mode markerem a je vizuálně dominantní (viditelnější) než mode marker.
result: pass

### 4. Ctrl+S ukládá v různých stavech
expected: Ctrl+S správně ukládá Modified soubor, Clean soubor nezmění, bez aktivního tabu neudělá nic, v Settings modalu uloží draft.
result: pass

### 5. Save failure zobrazí inline chybu a toast
expected: Při pokusu uložit read-only soubor se zobrazí inline chybová zpráva v editoru A současně error toast.
result: skipped
reason: nemám nastavení readonly

### 6. Save failure nezavře tab
expected: Po selhání uložení (error) karta zůstane otevřená, v Modified stavu, bez guard dialogu.
result: skipped
reason: nemám jak overit

### 7. Save UX i18n parity
expected: Všech 5 jazyků (EN, CS, SK, PL, DE) zobrazuje správné save UX texty ve status baru a toastech.
result: skipped
reason: nemám jak overit, nejde zkompilovat kod

## Summary

total: 7
passed: 4
issues: 0
pending: 0
skipped: 3

## Gaps

[none yet]
