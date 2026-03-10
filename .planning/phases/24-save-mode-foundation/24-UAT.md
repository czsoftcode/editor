---
status: complete
phase: 24-save-mode-foundation
source:
  - 24-01-SUMMARY.md
  - 24-02-SUMMARY.md
  - 24-03-SUMMARY.md
  - 24-04-SUMMARY.md
started: 2026-03-09T19:57:50Z
updated: 2026-03-09T20:06:08Z
---

## Current Test

[testing complete]

## Tests

### 1. M-CTRL-S-EDITOR - Ctrl+S ulozi aktivni tab bez zmeny fokusu
expected: Po uprave obsahu v aktivnim tabu stisk Ctrl+S okamzite ulozi soubor, tab prejde do Saved a obsah na disku odpovida editoru.
result: pass

### 2. M-SAVE-FAILURE - Save failure je viditelny a tab zustane Modified
expected: Pri save read-only souboru se zobrazi error toast, tab zustane Modified a nenastane tichy fail.
result: skipped
reason: skip - nevim, kde se nastavuje nyni readonly stav

### 3. M-CTRL-S-MODAL - Ctrl+S v Settings modalu ulozi settings draft
expected: V otevrenem Settings modalu Ctrl+S ulozi settings draft (ekvivalent modal Save) a nespusti editor file-save.
result: pass

### 4. M-RESTART-PERSISTENCE - Save mode se persistuje pres restart
expected: Po ulozeni Automatic/Manual a restartu aplikace zustane aktivni naposledy ulozeny rezim.
result: skipped
reason: skip

### 5. M-RUNTIME-APPLY - Zmena save mode se aplikuje bez restartu
expected: Po Save v Settings se autosave chovani okamzite prepne v obou smerech (Manual->Automatic i zpet).
result: pass

## Summary

total: 5
passed: 3
issues: 0
pending: 0
skipped: 2

## Gaps

none
