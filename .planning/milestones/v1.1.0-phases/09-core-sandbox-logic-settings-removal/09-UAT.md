---
status: complete
phase: 09-core-sandbox-logic-settings-removal
source: 09-01-SUMMARY.md, 09-02-SUMMARY.md, 09-03-SUMMARY.md
started: 2026-03-06T00:15:00Z
updated: 2026-03-06T00:22:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Editor spuštění po odstranění sandboxu
expected: Spusťte editor (cargo run). Editor se spustí bez chyb, zobrazí hlavní okno s file tree, editorem a terminálovým panelem. Žádné panic, crash ani chybové hlášky.
result: pass

### 2. Settings dialog bez sandbox toggle
expected: Otevřete Settings dialog. V dialogu NENÍ žádný sandbox checkbox, toggle ani zmínka o sandbox režimu. Dialog zobrazuje ostatní nastavení normálně.
result: pass

### 3. Migrace starých settings
expected: Pokud máte existující settings soubor (TOML nebo JSON) se starým polem sandbox_mode nebo project_read_only, editor ho načte bez chyby. Pole jsou tiše ignorována/odstraněna.
result: pass

### 4. File tree zobrazuje projektový kořen
expected: File tree vlevo zobrazuje přímo kořenový adresář projektu. Není žádný "Sandbox" toggle button ani "Soubory (Sandbox)" label. Stromová struktura odpovídá skutečnému obsahu projektu.
result: pass

### 5. Terminál používá projektový adresář
expected: Otevřete nový terminál (build, compile, git). Working directory terminálu je kořenový adresář projektu (ne sandbox adresář). Příkaz `pwd` vrátí cestu k projektu.
result: pass

### 6. AI agent start bez sync dialogu
expected: Spusťte AI agenta (Claude panel). Agent se spustí přímo bez zobrazení sync/promotion dialogu. Žádné dotazy na synchronizaci sandboxu.
result: pass

### 7. Kompilace bez chyb
expected: `cargo build` proběhne úspěšně bez errorů. Warningy mohou být přítomny (řeší fáze 12).
result: pass

### 8. Všechny testy procházejí
expected: `cargo test` spustí všechny testy a všechny projdou (0 failures). Žádné sandbox-related testy by neměly existovat.
result: pass

## Summary

total: 8
passed: 8
issues: 0
pending: 0
skipped: 0

## Gaps

[none yet]
