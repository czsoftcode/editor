---
id: S11
parent: M001
milestone: M001
provides:
  - Custom YAML-like frontmatter parser (FmValue, FmNode, FmDocument)
  - Full round-trip fidelity pro .planning/ markdown soubory
  - Dot-notation get/set pro nested hodnoty
  - GSD subcommand dispatch (/gsd state, progress, config, help)
  - GsdConfig s dot-notation load/get/set/save
  - Path helpers (planning_dir, state_path, roadmap_path, slugify)
  - /gsd state display/update/patch + progress bar
  - Body section append (record_decision, record_blocker)
  - Two-level autocomplete pro /gsd subcommands
key_files:
  - src/app/ui/terminal/ai_chat/gsd/frontmatter.rs
  - src/app/ui/terminal/ai_chat/gsd/mod.rs
  - src/app/ui/terminal/ai_chat/gsd/config.rs
  - src/app/ui/terminal/ai_chat/gsd/paths.rs
  - src/app/ui/terminal/ai_chat/gsd/state.rs
key_decisions:
  - "Custom YAML-like parser bez nových závislostí"
  - "Raw source lines per FmNode pro lossless round-trip"
  - "Reject keys s dvojtečkami pro tolerantní parsing"
  - "GSD dispatch match-based routing (mirror slash.rs)"
  - "Two-level autocomplete pro /gsd prefix"
  - "Config value parsing: bool → int → float → string fallback"
  - "ISO timestamp přes Howard Hinnant algoritmus bez chrono"
  - "Body section manipulation přes string search (ne AST)"
patterns_established:
  - "TDD combined approach — testy a implementace v jednom souboru"
  - "FmDocument API: parse → get/set → to_string_content round-trip"
  - "check_planning_dir guard pro graceful chybění .planning/"
observability_surfaces:
  - "/gsd state — zobrazí milestone, phase, status, progress bar, velocity, blockers"
  - "/gsd progress — kompaktní progress bar"
drill_down_paths:
  - tasks/T01-SUMMARY.md
  - tasks/T02-SUMMARY.md
  - tasks/T03-SUMMARY.md
duration: 18min
verification_result: passed
completed_at: 2026-03-07
---

# S11: Gsd Core State Engine

**Custom YAML-like frontmatter parser s round-trip fidelitou, GSD subcommand dispatch, config management a state/progress příkazy.**

## What Happened

Tři tasky vybudovaly GSD datovou a příkazovou vrstvu: T01 implementoval custom YAML-like parser (FmValue, FmNode, FmDocument) s podporou celého potřebného YAML subsetu — nested maps, listy, inline kolekce, block scalars — a plným round-trip (parse→modify→serialize zachovává nemodifikované nody verbatim). 36 unit testů. T02 přidal GSD subcommand dispatch (match-based routing), GsdConfig s dot-notation, path helpers a two-level autocomplete. T03 dodal /gsd state (display/update/patch) a /gsd progress s Unicode progress barem, plus body section manipulaci (append_to_section, record_decision, record_blocker) — 18 unit testů.

## Verification

- `cargo check` čistý
- 75 GSD testů celkem (36 frontmatter + 18 state + dispatch/config testy)
- Round-trip fidelity ověřena — parse(x).to_string_content() == x pro nemodifikované dokumenty
- Tolerantní parsing — malformed řádky přeskočeny bez paniku

## Deviations

- Frontmatter.rs stub vytvořen v T02 pro kompilaci (T01 ho následně naplnil)
- Dva auto-fixed bugy v T01: trailing newline u prázdného body, colon-prefixed malformed lines

## Known Limitations

- Dot-notation podporuje pouze 2 úrovně zanoření
- Body section manipulation přes string search — ne plně AST-aware

## Follow-ups

- Žádné — GSD core state engine je kompletní pro aktuální scope

## Files Created/Modified

- `src/app/ui/terminal/ai_chat/gsd/frontmatter.rs` — parser, FmValue/FmNode/FmDocument, 36 testů
- `src/app/ui/terminal/ai_chat/gsd/mod.rs` — dispatch, help, matching_subcommands
- `src/app/ui/terminal/ai_chat/gsd/config.rs` — GsdConfig, cmd_config
- `src/app/ui/terminal/ai_chat/gsd/paths.rs` — planning_dir, state_path, roadmap_path, slugify
- `src/app/ui/terminal/ai_chat/gsd/state.rs` — cmd_state, cmd_progress, body helpers, 18 testů
- `src/app/ui/terminal/ai_chat/slash.rs` — /gsd v COMMANDS a dispatch
- `src/app/ui/terminal/ai_chat/render.rs` — /gsd autocomplete
- `src/app/ui/widgets/ai/chat/input.rs` — /gsd autocomplete keyboard

## Forward Intelligence

### What the next slice should know
- FmDocument API je základ pro všechny GSD příkazy pracující s .planning/ soubory

### What's fragile
- Dot-notation omezena na 2 úrovně — hlubší zanoření vyžaduje rozšíření

### Authoritative diagnostics
- frontmatter.rs testy — 36 testů pokrývajících edge cases parseru
- state.rs testy — 18 testů pro display, update, patch, append

### What assumptions changed
- Původně se zvažoval serde_yaml — custom parser je jednodušší a nevyžaduje závislost
