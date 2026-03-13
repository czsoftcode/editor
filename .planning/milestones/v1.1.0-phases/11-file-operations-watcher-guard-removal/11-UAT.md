---
status: complete
phase: 11-file-operations-watcher-guard-removal
source: 11-01-SUMMARY.md, 11-02-SUMMARY.md
started: 2026-03-06T00:32:00Z
updated: 2026-03-06T00:36:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Ukládání souborů funguje přímo
expected: Otevřete soubor v editoru, proveďte změnu a uložte (Ctrl+S). Soubor se uloží bez chyb. Žádné kontroly sandbox cesty, žádné read-only blokování.
result: pass

### 2. Autosave funguje
expected: Pokud je autosave zapnutý v settings, upravte soubor a počkejte. Soubor se automaticky uloží bez chyb.
result: skipped
reason: Autosave nemá UI toggle v settings dialogu, nelze testovat z UI

### 3. Watcher ignoruje .polycredo/
expected: Vytvořte nebo upravte soubor v adresáři .polycredo/. Editor NESMÍ reagovat na tuto změnu (žádný reload, žádný conflict dialog). Změny v projektových souborech ale watcher stále hlídá.
result: pass

### 4. AI agent používá "exec" místo "exec_in_sandbox"
expected: Spusťte AI agenta a požádejte ho o spuštění příkazu. Agent použije nástroj "exec" (ne "exec_in_sandbox"). Příkaz se spustí v kořenovém adresáři projektu.
result: pass

### 5. Git operace fungují bez guardu
expected: Proveďte git operaci (commit, push, pull) přes editor. Operace proběhne bez sandbox guardu — žádné blokování kvůli sandbox režimu.
result: skipped

## Summary

total: 5
passed: 3
issues: 0
pending: 0
skipped: 2

## Gaps

[none yet]
