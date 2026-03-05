---
status: complete
phase: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu
source:
  - 05-01-SUMMARY.md
  - 05-02-SUMMARY.md
  - 05-03-SUMMARY.md
  - 05-04-SUMMARY.md
started: 2026-03-05T12:00:00Z
updated: 2026-03-05T12:00:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Sandbox Save/Cancel flow
expected: Otevri Settings modal. Prepni sandbox rezim a klikni Cancel — nic se neulozi. Pak prepni znovu a klikni Save — zmena se ulozi a projevi se okamzite s toast potvrzenim.
result: pass

### 2. Potvrzeni sandbox OFF
expected: Pri prepnuti sandbox z ON na OFF se zobrazi potvrzovaci dialog (apply now / defer). Uzivatel musi explicitne potvrdit vypnuti.
result: pass

### 3. Multi-window propagace sandbox rezimu
expected: Pokud mas otevrena 2 okna editoru, zmena sandbox rezimu v jednom okne se automaticky projevi i v druhem okne (pres settings_version).
result: pass

### 4. Graceful restart terminalu po prepnuti sandboxu
expected: Otevri terminal, spust nejaky proces (napr. ls). Prepni sandbox rezim. Terminal se restartuje s novym working dir (sandbox/projekt), bezici proces dobehl gracefully. Label terminalu odpovida novemu rezimu.
result: issue
reported: "adresar se zmeni, ale okno se vymaze"
severity: minor

### 5. Tab remap prompt po prepnuti sandbox rezimu
expected: S otevrenymi taby prepni sandbox rezim. Zobrazi se toast s nabidkou remapovat taby na novy root. Po kliknuti se taby premapuji. Pokud soubor v novem rootu neexistuje, tab zustane otevreny s varovanim.
result: skipped

### 6. Blokace sandbox OFF pri staged souborech
expected: Pokud existuji staged soubory v sandboxu, prepnuti sandbox na OFF je blokovano. Zobrazi se dialog informujici o staged souborech. Draft se vrati na original.
result: issue
reported: ".polycredo/ je v .gitignore, takze git add na soubory v sandboxu nefunguje — blokace sandbox OFF pri staged souborech se nemuze nikdy aktivovat"
severity: major

### 7. Sync dialog pri zapnuti sandbox ON
expected: Pri prepnuti sandbox z OFF na ON se zobrazi sync dialog s planem prenosu souboru z projektu do sandboxu. Po potvrzeni probehne sync asynchronne s toast notifikaci o vysledku.
result: pass

### 8. Cleanup remap toastu po expiraci
expected: Po prepnuti sandbox rezimu se zobrazi remap toast. Pokud uzivatel neklikne a toast expiruje, pending remap request se automaticky vymaze (zadny zombie stav).
result: skipped

## Summary

total: 8
passed: 4
issues: 2
pending: 0
skipped: 2
skipped: 0

## Gaps

- truth: "Terminal se restartuje s novym working dir, bezici proces dobehl gracefully, label odpovida novemu rezimu"
  status: failed
  reason: "User reported: adresar se zmeni, ale okno se vymaze"
  severity: minor
  test: 4
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
- truth: "Prepnuti sandbox na OFF je blokovano pri staged souborech s dialogem"
  status: failed
  reason: "User reported: .polycredo/ je v .gitignore, takze git add na soubory v sandboxu nefunguje — blokace sandbox OFF pri staged souborech se nemuze nikdy aktivovat"
  severity: major
  test: 6
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
