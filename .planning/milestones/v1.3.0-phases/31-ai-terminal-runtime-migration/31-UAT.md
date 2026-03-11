---
status: diagnosed
phase: 31-ai-terminal-runtime-migration
source: 31-01-SUMMARY.md, 31-02-SUMMARY.md, 31-03-SUMMARY.md, 31-04-SUMMARY.md
started: 2026-03-11T13:19:18+01:00
updated: 2026-03-11T13:25:38+01:00
---

## Current Test

[testing complete]

## Tests

### 1. AI terminal se otevře a přijme dotaz
expected: AI terminal panel jde otevřít a lze odeslat prompt bez pádu UI.
result: issue
reported: "co dela ollama model v AI terminalu??? tam nema co delat, Ollama jde dopryc s celym PolyCredo CLI neboli ai_chat, zzustava POUZE AI terminal, kde je ai_bar a nic vic!!! Uprav to"
severity: major

### 2. Streaming odpovědi nezamrzne UI
expected: Při streamování odpovědi zůstává UI responzivní (lze přepínat focus/panely, editor nereaguje se zpožděním).
result: skipped
reason: UAT předčasně ukončena uživatelem, přechod na gaps.

### 3. Model picker a slash/GSD tok fungují jako dřív
expected: Model picker je použitelný a slash/GSD workflow v AI terminalu funguje bez regresí v běžném použití.
result: skipped
reason: UAT předčasně ukončena uživatelem, přechod na gaps.

### 4. Retry flow při chybě je srozumitelný
expected: Když dotaz selže (např. dočasná chyba asistenta), zobrazí se krátká lidská hláška a jasná možnost Retry.
result: skipped
reason: UAT předčasně ukončena uživatelem, přechod na gaps.

### 5. Approval flow zůstává funkční
expected: Citlivé akce pořád vyžadují potvrzení stejně jako před migrací; schválení/odmítnutí se projeví očekávaně.
result: skipped
reason: UAT předčasně ukončena uživatelem, přechod na gaps.

### 6. Security guardy pořád blokují nebezpečné operace
expected: Zakázané/nebezpečné operace jsou blokované a důvod blokace je uživatelsky čitelný (bez tichého selhání).
result: skipped
reason: UAT předčasně ukončena uživatelem, přechod na gaps.

## Summary

total: 6
passed: 0
issues: 1
pending: 0
skipped: 5

## Gaps

- truth: "V AI terminalu nemají být prvky/navázání na Ollama modely; po odstranění PolyCredo CLI má zůstat pouze AI terminal tok dle uzamčeného scope."
  status: failed
  reason: "User reported: co dela ollama model v AI terminalu??? tam nema co delat, Ollama jde dopryc s celym PolyCredo CLI neboli ai_chat, zzustava POUZE AI terminal, kde je ai_bar a nic vic!!! Uprav to"
  severity: major
  test: 1
  root_cause: "Requirements/scope drift: phase 31 plans + state explicitně drží model picker/Ollama runtime vazby, takže kód to konzistentně zachovává."
  artifacts:
    - path: ".planning/phases/31-ai-terminal-runtime-migration/31-02-PLAN.md"
      issue: "TERM-03 task požaduje model picker continuity"
    - path: "src/app/ui/terminal/right/ai_bar.rs"
      issue: "AI bar renderuje model combobox"
    - path: "src/app/ui/terminal/ai_chat/logic.rs"
      issue: "runtime chat path je navázaný na OllamaProvider"
    - path: "src/app/ui/background.rs"
      issue: "background loop stále polluje/syncuje Ollama modely"
  missing:
    - "Sjednotit source-of-truth: explicitně odstranit model picker/Ollama vazby z phase 31 fix planů"
    - "Odstranit UI i runtime coupling na ws.ai.ollama.* v AI terminal toku"
    - "Zachovat jen externí assistant flow + approval/security kontrakt"
  debug_session: ".planning/debug/31-ollama-in-ai-terminal.md"
