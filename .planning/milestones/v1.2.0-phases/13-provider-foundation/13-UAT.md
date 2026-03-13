---
status: complete
phase: 13-provider-foundation
source: 13-01-SUMMARY.md, 13-02-SUMMARY.md
started: 2026-03-06T12:00:00Z
updated: 2026-03-06T12:30:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Ollama Status Icon
expected: V AI baru je kruhový indikátor stavu Ollama. Zelený = běží, červený/šedý = neběží. Automatický polling každých 10s.
result: pass

### 2. Model Picker (Ollama běží)
expected: Když Ollama běží, v AI baru se zobrazí ComboBox se seznamem dostupných modelů. Seznam odpovídá modelům nainstalovaným v Ollama.
result: pass

### 3. Model Picker (Ollama neběží)
expected: Když Ollama neběží, ComboBox je buď skrytý, prázdný, nebo disabled — nepůsobí chybově.
result: issue
reported: "beres Ollama nastaveni z nastaveni pluginu Ollama, kde mam nyni https://ollama.com"
severity: major

### 4. Výběr modelu — persistence
expected: Vyber model v ComboBoxu, zavři a znovu otevři editor. Vybraný model zůstává zachován.
result: pass

## Summary

total: 4
passed: 3
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "Když Ollama neběží, ComboBox je buď skrytý, prázdný, nebo disabled — nepůsobí chybově."
  status: failed
  reason: "User reported: beres Ollama nastaveni z nastaveni pluginu Ollama, kde mam nyni https://ollama.com"
  severity: major
  test: 3
  root_cause: "init.rs řádky 209-213 čte ollama_base_url z settings.plugins['ollama'].config['API_URL'] — pokud uživatel má v plugin settings špatnou URL (např. https://ollama.com), native provider polling ji použije místo localhost:11434"
  artifacts:
    - path: "src/app/ui/workspace/state/init.rs"
      issue: "Řádky 209-213: ollama_base_url se čte z plugin settings API_URL, sdílí config s WASM pluginem"
    - path: "src/app/ui/background.rs"
      issue: "Řádek 201: spawn_ollama_check používá ws.ollama_base_url bez validace"
  missing:
    - "Native provider by měl mít vlastní URL konfiguraci oddělenou od WASM plugin settings, nebo validovat že URL ukazuje na lokální server"
  debug_session: ""
