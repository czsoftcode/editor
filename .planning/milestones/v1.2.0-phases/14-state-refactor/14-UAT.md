---
status: diagnosed
phase: 14-state-refactor
source: 14-01-SUMMARY.md, 14-02-SUMMARY.md
started: 2026-03-06T11:00:00Z
updated: 2026-03-06T11:05:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Aplikace se spustí bez chyb
expected: Spusť aplikaci (`cargo run`). Hlavní okno se otevře bez pádů nebo error logů v terminálu. Editor zobrazí workspace normálně.
result: pass

### 2. AI Chat panel funguje
expected: Otevři AI Chat panel (Claude panel vpravo). Napiš zprávu do promptu a odešli. Prompt se vyčistí, zpráva se zobrazí v historii konverzace. Loading indikátor se objeví při čekání na odpověď.
result: issue
reported: "nechapu, co kam napsat, z ollama tam mam jenom modely bez moznosti ollama spustit, externi CLI bezi a prikazy v terminalu bezi"
severity: major

### 3. AI Settings jsou přístupné
expected: Otevři AI settings (přes menu nebo UI). Expertise role, reasoning depth, font scale, language a provider selection jsou viditelné a nastavitelné. Změna font scale se projeví okamžitě na velikosti textu v AI panelu.
result: pass

### 4. Ollama stav v AI baru
expected: V AI baru (pravý panel) se zobrazuje stav Ollama připojení — status ikona, seznam dostupných modelů a vybraný model. Pokud Ollama není dostupná, zobrazí se odpovídající stav (disconnected/error).
result: pass

## Summary

total: 4
passed: 3
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "AI Chat panel umožňuje napsat zprávu, odeslat ji a zobrazit konverzaci"
  status: failed
  reason: "User reported: nechapu, co kam napsat, z ollama tam mam jenom modely bez moznosti ollama spustit, externi CLI bezi a prikazy v terminalu bezi"
  severity: major
  test: 2
  root_cause: "Pre-existující UX problém — textový vstup pro AI chat je v odděleném plovoucím okně (ai_chat/mod.rs), ne v pravém panelu. Pravý panel (right/mod.rs + ai_bar.rs) zobrazuje pouze Ollama status, model picker a terminály. AI Chat okno je defaultně zavřené (show_ai_chat: false). Není regrese z phase 14."
  artifacts:
    - path: "src/app/ui/terminal/right/mod.rs"
      issue: "Pravý panel nemá chat vstup, pouze terminály a ai_bar"
    - path: "src/app/ui/terminal/ai_chat/mod.rs"
      issue: "AI Chat okno defaultně zavřené, uživatel neví jak ho otevřít"
  missing:
    - "Vizuální návod nebo tlačítko v pravém panelu pro otevření AI Chat okna"
    - "Nebo integrace chat vstupu přímo do pravého panelu"
  debug_session: ".planning/debug/ai-chat-input-visibility.md"
