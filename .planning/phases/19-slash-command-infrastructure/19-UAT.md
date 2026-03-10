---
status: diagnosed
phase: 19-slash-command-infrastructure
source: [19-01-SUMMARY.md, 19-02-SUMMARY.md]
started: 2026-03-07T01:00:00Z
updated: 2026-03-07T01:15:00Z
---

## Current Test

[testing complete]

## Tests

### 1. /help Command
expected: Type `/help` in AI chat prompt and press Enter. A system message appears with a markdown table listing all 7 commands (/help, /clear, /new, /model, /git, /build, /settings) with descriptions. The message has a distinct blue-tinted background and "System" label.
result: issue
reported: "po zadani / by se mel otevrit naseptavac prikazu"
severity: minor

### 2. /clear Command
expected: Type `/clear` and press Enter. The conversation history is cleared (all previous messages disappear). A system message "Conversation cleared." appears. Token counters reset to 0.
result: pass
note: "jeste by bylo dobre tabulatorem dokoncovat prikaz"

### 3. /new Command
expected: Type `/new` and press Enter. Conversation is cleared and a fresh ASCII logo appears with current model name, expertise, and reasoning depth — same as when opening a new chat session.
result: pass

### 4. /settings Command
expected: Type `/settings` and press Enter. The Settings dialog opens. No system message is shown in conversation (silent command).
result: pass

### 5. /model List Models
expected: Type `/model` (no arguments) and press Enter. A system message appears listing all available Ollama models, with the currently active model marked as "(active)" in bold.
result: pass

### 6. /model Switch Model
expected: Type `/model <model-name>` with a valid model name and press Enter. A system message confirms "Switched to model: **<name>**". The active model changes for subsequent AI queries.
result: pass
note: "opet by byl dobry naseptavac a tab dokoncovani"

### 7. /git Command
expected: Type `/git` and press Enter. A placeholder "Loading git status..." appears immediately. After a moment, it updates in-place to show the current branch name and `git diff --stat` output (or "No uncommitted changes" if clean). UI stays responsive during loading.
result: issue
reported: "zobrazi se v jednom radku: [ .planning/ROADMAP.md | 2 +- .planning/STATE.md | 8 ++++---- 2 files changed, 5 insertions(+), 5 deletions(-) ](code) misto v tabulce"
severity: minor

### 8. /build Command
expected: Type `/build` and press Enter. A placeholder "Building..." appears immediately. After cargo build completes, the message updates in-place with build results (success or error summary). UI stays responsive during build.
result: pass

### 9. Fuzzy Command Suggestion
expected: Type a typo like `/halp` or `/bild` and press Enter. A system message suggests the correct command: "Unknown command: `/halp`. Did you mean: `/help`?" Long words (>10 chars) are passed through to AI instead.
result: pass
note: "kliknutim na nabidku opravy by se mohl prikaz vykonat"

### 10. Offline Slash Commands
expected: With Ollama disconnected (or not running), type `/help` and press Enter. The command works normally — slash commands do not require an active Ollama connection. Non-slash messages still show the "Ollama disconnected" toast.
result: pass

## Summary

total: 10
passed: 8
issues: 2
pending: 0
skipped: 0

## Gaps

- truth: "When typing `/` in the chat prompt, a command autocomplete popup should appear"
  status: failed
  reason: "User reported: po zadani / by se mel otevrit naseptavac prikazu"
  severity: minor
  test: 1
  root_cause: "Slash autocomplete nebyl implementován — systém funguje pouze dispatch on Enter. Chybí detekce `/` prefixu za běhu psaní, filtrační API, a popup rendering."
  artifacts:
    - path: "src/app/ui/widgets/ai/chat/input.rs"
      issue: "Žádná detekce / prefixu za běhu psaní"
    - path: "src/app/ui/terminal/ai_chat/slash.rs"
      issue: "Chybí veřejná funkce pro filtraci příkazů podle prefixu"
    - path: "src/app/ui/terminal/ai_chat/render.rs"
      issue: "Chybí popup rendering pro autocomplete"
  missing:
    - "Přidat pub fn matching_commands(filter) do slash.rs"
    - "Detekovat / prefix v input.rs a aktivovat autocomplete stav"
    - "Vykreslit popup s filtrovanými příkazy v render.rs"
    - "Klávesová navigace (ArrowUp/Down, Enter, Escape, Tab) v autocomplete"
  debug_session: ".planning/debug/no-slash-autocomplete.md"

- truth: "Git diff output should render as a multi-line code block preserving formatting"
  status: failed
  reason: "User reported: zobrazi se v jednom radku misto v tabulce"
  severity: minor
  test: 7
  root_cause: "Regex pro cesty v render.rs (flush_block, řádky 119-131) nahrazuje cesty uvnitř fenced code blocků, čímž rozbije markdown syntax. Chybí sledování code-fence stavu."
  artifacts:
    - path: "src/app/ui/widgets/ai/chat/render.rs"
      issue: "Path regex se aplikuje i na obsah uvnitř code blocků"
  missing:
    - "Přidat sledování code-fence stavu do smyčky for line in text.lines()"
    - "Přeskočit regex nahrazování cest uvnitř fenced code blocků"
  debug_session: ".planning/debug/git-cmd-single-line.md"
