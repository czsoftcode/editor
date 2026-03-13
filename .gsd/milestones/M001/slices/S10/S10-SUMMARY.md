---
id: S10
parent: M001
milestone: M001
provides:
  - Slash command registry a dispatch systém (slash.rs)
  - 7 built-in příkazů (/help, /clear, /new, /settings, /model, /git, /build)
  - SlashResult::Async vzor pro background příkazy
  - Levenshtein fuzzy matching pro překlepy
  - System message rendering s SYSTEM_MSG_MARKER
  - Code-fence aware rendering pro /git výstup
  - Slash command autocomplete popup s keyboard navigací
key_files:
  - src/app/ui/terminal/ai_chat/slash.rs
  - src/app/ui/terminal/ai_chat/logic.rs
  - src/app/ui/widgets/ai/chat/conversation.rs
  - src/app/ui/widgets/ai/chat/input.rs
  - src/app/ui/terminal/ai_chat/render.rs
  - src/app/ui/background.rs
key_decisions:
  - "Static slice registry místo HashMap pro 7 příkazů"
  - "Levenshtein threshold <= 2, word length <= 10"
  - "SYSTEM_MSG_MARKER \\x00SYS\\x00 — stripped at render"
  - "Reuse run_build_check pro /build"
  - "Generation counter pro stale async výsledky"
  - "Conservative code-fence check — skip ALL path regex pro fenced bloky"
  - "Enter v autocomplete selects + sends, Tab completes without send"
  - "Popup renderuje pod inputem"
patterns_established:
  - "SlashResult::Immediate a Silent vzory pro synchronní příkazy"
  - "SlashResult::Async + generation counter pro background příkazy"
  - "Dismissed flag v autocomplete pro Escape tracking"
observability_surfaces:
  - none
drill_down_paths:
  - tasks/T01-SUMMARY.md
  - tasks/T02-SUMMARY.md
  - tasks/T03-SUMMARY.md
  - tasks/T04-SUMMARY.md
duration: 17min
verification_result: passed
completed_at: 2026-03-07
---

# S10: Slash Command Infrastructure

**Kompletní slash command systém se 7 příkazy, async vzorem, fuzzy matching, code-fence renderingem a autocomplete popup.**

## What Happened

Čtyři tasky vybudovaly kompletní slash infrastrukturu: T01 vytvořil command registry, dispatch, Levenshtein fuzzy matching a synchronní příkazy (/help, /clear, /new, /settings) s distinct system message renderingem. T02 přidal async příkazy (/model, /git, /build) s background vlákny, generation counter pro stale result ochranu a build summary formátování. T03 opravil code-fence rendering — /git výstup se nyní renderuje jako korektní code bloky bez falešné monologue detekce. T04 dodal autocomplete popup s prefix filtrací, keyboard navigací (Enter/Tab/Escape/Arrows) a click selekcí.

## Verification

- `cargo check` čistý
- 5 unit testů (levenshtein, help output, fuzzy matching, long word passthrough)
- 4 unit testy pro /model varianty
- matching_commands() test
- Slash dispatch funguje offline (před Ollama check)

## Deviations

- Autocomplete popup renderuje pod inputem místo nad ním (user preference)
- Dismissed + prev_text fieldy přidány nad rámec plánu (lepší UX)

## Known Limitations

- Tab focus traversal po Tab completion (kosmetické)

## Follow-ups

- Žádné — slash infrastruktura je kompletní pro aktuální scope

## Files Created/Modified

- `src/app/ui/terminal/ai_chat/slash.rs` — registry, dispatch, handlers, fuzzy matching, testy
- `src/app/ui/terminal/ai_chat/logic.rs` — slash intercept, system message filter
- `src/app/ui/widgets/ai/chat/conversation.rs` — system message rendering
- `src/app/ui/widgets/ai/chat/input.rs` — SlashAutocomplete, keyboard handling
- `src/app/ui/terminal/ai_chat/render.rs` — popup rendering, click-to-execute
- `src/app/ui/background.rs` — slash_build_rx/git_rx polling, format_slash_build_summary

## Forward Intelligence

### What the next slice should know
- Slash dispatch je match-based — nový příkaz = nový arm v dispatch + záznam v COMMANDS

### What's fragile
- Generation counter (slash_conversation_gen) — musí se vždy bumpnout při /clear

### Authoritative diagnostics
- slash.rs unit testy — pokrývají dispatch, fuzzy matching, command výstupy

### What assumptions changed
- /build nepotřeboval novou implementaci — reuse run_build_check stačil
