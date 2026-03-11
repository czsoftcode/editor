---
status: complete
phase: 30-cli-namespace-removal-foundation
source: 30-01-SUMMARY.md, 30-02-SUMMARY.md, 30-03-SUMMARY.md, 30-04-SUMMARY.md
started: 2026-03-11T11:26:35+01:00
updated: 2026-03-11T11:31:15+01:00
---

## Current Test

[testing complete]

## Tests

### 1. Aplikace naběhne po odstranění CLI vrstvy
expected: Spuštění editoru proběhne bez pádu a bez chyb o chybějícím `app::cli`.
result: pass

### 2. AI terminal je assistant-only (bez model/status prvků)
expected: V AI baru není model selector ani provider connection status; zůstává assistant flow (assistant volba + start akce).
result: pass

### 3. AI chat workflow je funkční po migraci na ai_core
expected: Odeslání dotazu v AI chatu funguje standardně (odeslání, průběh, odpověď nebo smysluplná runtime hláška), bez namespace regresí.
result: pass

### 4. V terminal AI slash flow není dostupný legacy /model ovladač
expected: Slash workflow neobsahuje starý `/model` režim; nabídka odpovídá assistant-only chování.
result: pass

### 5. Regresní kontrola základní práce v editoru
expected: Otevření souboru, editace a běžná práce ve workspace fungují jako dřív, bez zjevných regresí po CLI cleanupu.
result: pass

## Summary

total: 5
passed: 5
issues: 0
pending: 0
skipped: 0

## Gaps

none
