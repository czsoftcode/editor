---
status: diagnosed
phase: 04-infrastructure
source: [04-01-SUMMARY.md]
started: 2026-03-05T03:43:19Z
updated: 2026-03-05T03:50:07Z
---

## Current Test

[testing complete]

## Tests

### 1. Settings sandbox režim persistuje (Save/Cancel)
expected: V Settings > Projekt přepni sandbox OFF a dej Cancel → nic se neuloží. Pak OFF + Save → zobrazí se toast a v settings.toml se uloží sandbox_mode = false.
result: pass

### 2. Změna se projeví až po reopen projektu
expected: Po uložení sandbox OFF znovu otevři projekt → build_in_sandbox a file_tree_in_sandbox odpovídají OFF (terminály nejedou v sandboxu).
result: pass

### 3. Terminály mají správný cwd a label
expected: Po reopen projektu při OFF běží terminály v rootu a label je "Terminal — <path>". Po zapnutí sandboxu label "Sandbox".
result: pass

### 4. Tooltip a inline poznámka v Settings
expected: U přepínače sandbox režimu je tooltip s vysvětlením OFF režimu a inline poznámka, že změna se projeví po reopen.
result: issue
reported: "popis je small, vubec jsem si ho nevsiml"
severity: cosmetic

## Summary

total: 4
passed: 3
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "U přepínače sandbox režimu je tooltip s vysvětlením OFF režimu a inline poznámka, že změna se projeví po reopen."
  status: failed
  reason: "User reported: popis je small, vubec jsem si ho nevsiml"
  severity: cosmetic
  test: 4
  root_cause: "Tooltip je navázaný jen na malou ikonu ℹ a doprovodné texty jsou renderované jako small().weak(), což snižuje viditelnost."
  artifacts:
    - path: "src/app/ui/workspace/modal_dialogs/settings.rs"
      issue: "Tooltip i inline poznámky jsou malé a deemphasized (small+weak), hover target je malý."
  missing:
    - "Zvětšit hover area tooltipu (např. celý řádek s přepínačem)"
    - "Zvýšit vizuální prioritu inline poznámky (nepoužívat pouze small+weak)"
  debug_session: .planning/debug/sandbox-tooltip-small.md
