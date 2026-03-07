# Requirements: PolyCredo Editor v1.2.1-dev

**Defined:** 2026-03-07
**Core Value:** Editor nesmí zahřívat notebook v klidovém stavu — idle CPU zátěž musí být minimální.

## v1.2.1-dev Requirements

Requirements for GSD Integration + Slash Commands milestone. Each maps to roadmap phases.

### Slash Command Infrastructure

- [x] **SLASH-01**: User can type `/help` in chat and see list of all available slash commands with descriptions
- [x] **SLASH-02**: User can type `/clear` to clear current chat conversation
- [x] **SLASH-03**: User can type `/new` to start a fresh conversation (reset conversation, keep prompt history)
- [x] **SLASH-04**: User can type `/model` to list available models or `/model <name>` to switch active model
- [x] **SLASH-05**: User can type `/git` to see git status/diff summary of current project
- [x] **SLASH-06**: User can type `/build` to trigger cargo build from chat panel
- [x] **SLASH-07**: User can type `/settings` to open settings dialog from chat
- [x] **SLASH-08**: Slash command dispatch intercepts `/` prefixed input before sending to AI model
- [x] **SLASH-09**: Unknown slash commands show helpful error message with suggestions

### GSD Core

- [ ] **CORE-01**: GSD frontmatter parser can parse YAML-like frontmatter from `.planning/` markdown files
- [ ] **CORE-02**: GSD frontmatter serializer can write frontmatter back to markdown files preserving content
- [ ] **CORE-03**: GSD config module can load, read, and update `.planning/config.json` with dot-notation paths
- [ ] **CORE-04**: GSD core utilities provide path helpers, phase numbering (integer + decimal), slug generation
- [ ] **CORE-05**: GSD handles missing `.planning/` directory gracefully with helpful user message

### GSD State & Progress

- [ ] **STATE-01**: User can run `/gsd state` to see current project state (milestone, phase, plan, progress)
- [ ] **STATE-02**: User can run `/gsd state update <field> <value>` to update STATE.md fields
- [ ] **STATE-03**: User can run `/gsd state patch` to batch-update multiple STATE.md fields
- [ ] **STATE-04**: User can run `/gsd progress` to see visual progress bar and phase/plan counts
- [ ] **STATE-05**: GSD state module can record metrics, add decisions, add blockers to STATE.md

### GSD Phase & Roadmap

- [ ] **PHASE-01**: User can run `/gsd phase list` to see all phases with status
- [ ] **PHASE-02**: User can run `/gsd phase add <description>` to append new phase to roadmap
- [ ] **PHASE-03**: User can run `/gsd phase insert <after> <description>` to insert decimal phase
- [ ] **PHASE-04**: User can run `/gsd phase remove <phase>` to remove and renumber phases
- [ ] **PHASE-05**: User can run `/gsd phase complete <phase>` to mark phase as complete
- [ ] **PHASE-06**: User can run `/gsd roadmap analyze` to see full roadmap analysis with disk status
- [ ] **PHASE-07**: User can run `/gsd roadmap get-phase <N>` to extract specific phase section

### GSD Verify & Template

- [ ] **VERIFY-01**: User can run `/gsd verify plan-structure <file>` to validate plan file structure
- [ ] **VERIFY-02**: User can run `/gsd verify phase-completeness <phase>` to check phase artifacts
- [ ] **VERIFY-03**: User can run `/gsd validate health` to check `.planning/` directory integrity
- [ ] **VERIFY-04**: User can run `/gsd template fill <type>` to generate pre-filled template files
- [ ] **VERIFY-05**: User can run `/gsd scaffold phase-dir --phase N --name <name>` to create phase directory

### GSD Milestone & Git

- [ ] **MILE-01**: User can run `/gsd milestone complete <version>` to archive completed milestone
- [ ] **MILE-02**: User can run `/gsd commit <message>` to create atomic git commit with GSD format
- [ ] **MILE-03**: Git operations run asynchronously (non-blocking) via background thread + channel pattern
- [ ] **MILE-04**: User can run `/gsd requirements mark-complete <ids>` to mark requirements as done

### GSD Init & AI Integration

- [ ] **INIT-01**: User can run `/gsd init execute-phase <N>` to aggregate full execution context
- [ ] **INIT-02**: User can run `/gsd init plan-phase <N>` to aggregate planning context
- [ ] **INIT-03**: User can run `/gsd init new-project` to bootstrap new project context
- [ ] **INIT-04**: User can run `/gsd init quick <description>` to set up quick task context
- [ ] **INIT-05**: GSD init commands can delegate aggregated context to AI model for content generation
- [ ] **INIT-06**: AI-delegated GSD commands use existing Ollama provider and streaming infrastructure

### Attribution & i18n

- [ ] **ATTR-01**: `/gsd help` output includes MIT attribution notice (Copyright Lex Christopherson)
- [ ] **ATTR-02**: About dialog mentions GSD integration with MIT license attribution
- [ ] **I18N-01**: All GSD slash command descriptions and error messages are localized in 5 languages (cs, en, de, ru, sk)
- [ ] **I18N-02**: GSD output markdown uses language-neutral formatting (data stays in English, UI chrome localized)

## Future Requirements

### GSD Advanced

- **ADV-01**: Tab completion for slash commands in chat input
- **ADV-02**: Slash command history filtering (up arrow recalls only `/` commands)
- **ADV-03**: `/gsd frontmatter` CRUD subcommands for direct frontmatter manipulation
- **ADV-04**: `/gsd history-digest` aggregating all SUMMARY.md data
- **ADV-05**: `/gsd todos` integration (add-todo, check-todos, list-todos)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Full YAML parser (serde_yaml) | GSD frontmatter is limited subset; custom parser sufficient |
| GSD web search (Brave API) | Requires API key, belongs to Claude Code agent workflow |
| Model profile resolution (opus/sonnet/haiku) | Claude API tiers, irrelevant for Ollama-based editor |
| Plugin/hook system for slash commands | Over-engineering; match-based dispatch sufficient for 10-15 commands |
| Interactive multi-step wizards | egui immediate mode makes wizards awkward; use single-command with flags |
| Async runtime (tokio) | Contradicts v1.2.0 key decision (ureq + std::thread) |
| Auto-execute GSD commands | Security concern; always show output, use approval workflow |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SLASH-01 | Phase 19 | Complete |
| SLASH-02 | Phase 19 | Complete |
| SLASH-03 | Phase 19 | Complete |
| SLASH-04 | Phase 19 | Complete |
| SLASH-05 | Phase 19 | Complete |
| SLASH-06 | Phase 19 | Complete |
| SLASH-07 | Phase 19 | Complete |
| SLASH-08 | Phase 19 | Complete |
| SLASH-09 | Phase 19 | Complete |
| CORE-01 | Phase 20 | Pending |
| CORE-02 | Phase 20 | Pending |
| CORE-03 | Phase 20 | Pending |
| CORE-04 | Phase 20 | Pending |
| CORE-05 | Phase 20 | Pending |
| STATE-01 | Phase 20 | Pending |
| STATE-02 | Phase 20 | Pending |
| STATE-03 | Phase 20 | Pending |
| STATE-04 | Phase 20 | Pending |
| STATE-05 | Phase 20 | Pending |
| PHASE-01 | Phase 21 | Pending |
| PHASE-02 | Phase 21 | Pending |
| PHASE-03 | Phase 21 | Pending |
| PHASE-04 | Phase 21 | Pending |
| PHASE-05 | Phase 21 | Pending |
| PHASE-06 | Phase 21 | Pending |
| PHASE-07 | Phase 21 | Pending |
| VERIFY-01 | Phase 21 | Pending |
| VERIFY-02 | Phase 21 | Pending |
| VERIFY-03 | Phase 21 | Pending |
| VERIFY-04 | Phase 21 | Pending |
| VERIFY-05 | Phase 21 | Pending |
| MILE-01 | Phase 21 | Pending |
| MILE-02 | Phase 21 | Pending |
| MILE-03 | Phase 21 | Pending |
| MILE-04 | Phase 21 | Pending |
| INIT-01 | Phase 22 | Pending |
| INIT-02 | Phase 22 | Pending |
| INIT-03 | Phase 22 | Pending |
| INIT-04 | Phase 22 | Pending |
| INIT-05 | Phase 22 | Pending |
| INIT-06 | Phase 22 | Pending |
| ATTR-01 | Phase 23 | Pending |
| ATTR-02 | Phase 23 | Pending |
| I18N-01 | Phase 23 | Pending |
| I18N-02 | Phase 23 | Pending |

**Coverage:**
- v1.2.1-dev requirements: 37 total
- Mapped to phases: 37
- Unmapped: 0

---
*Requirements defined: 2026-03-07*
*Last updated: 2026-03-07 after roadmap creation*
