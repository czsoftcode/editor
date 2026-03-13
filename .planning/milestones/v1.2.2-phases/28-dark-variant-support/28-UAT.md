---
status: testing
phase: 28-dark-variant-support
source: 28-01-SUMMARY.md
started: 2026-03-10T22:08:16Z
updated: 2026-03-10T22:10:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

number: 3
name: Přepnutí mezi Default a Midnight mění dark vizuál bez rozbití UI
expected: |
  Přepnutí varianty v dark režimu viditelně změní paletu (např. panely/pozadí),
  ale texty, kontrast a ovladatelnost zůstanou čitelné.
awaiting: user response

## Tests

### 1. Dark Variant Picker je dostupný v Settings
expected: V Settings při aktivním dark režimu je viditelný výběr dark varianty a obsahuje minimálně volby "Default" a "Midnight".
result: issue
reported: "nic neobsahuje, asi to neni implementovano"
severity: major

### 2. Výběr Midnight se aplikuje a přetrvá po uložení
expected: Po výběru "Midnight" a potvrzení nastavení se UI přepne na midnight dark vzhled a po restartu editoru zůstane varianta "Midnight" aktivní.
result: issue
reported: "issue - neni nic na vyber ani dark ani light"
severity: major

### 3. Přepnutí mezi Default a Midnight mění dark vizuál bez rozbití UI
expected: Přepnutí varianty v dark režimu viditelně změní paletu (např. panely/pozadí), ale texty, kontrast a ovladatelnost zůstanou čitelné.
result: pending

### 4. Překlady dark variant jsou dostupné ve vícejazyčném UI
expected: V UI jsou názvy dark variant lokalizované (ne hardcoded interní klíče) alespoň při přepnutí EN/CZ/SK/DE/RU.
result: pending

## Summary

total: 4
passed: 0
issues: 2
pending: 2
skipped: 0

## Gaps

- truth: "V Settings při aktivním dark režimu je viditelný výběr dark varianty a obsahuje minimálně volby Default a Midnight."
  status: failed
  reason: "User reported: nic neobsahuje, asi to neni implementovano"
  severity: major
  test: 1
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
- truth: "Po výběru Midnight a potvrzení nastavení se UI přepne na midnight dark vzhled a po restartu editoru zůstane varianta Midnight aktivní."
  status: failed
  reason: "User reported: issue - neni nic na vyber ani dark ani light"
  severity: major
  test: 2
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
