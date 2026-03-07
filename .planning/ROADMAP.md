# Roadmap: PolyCredo Editor

## Milestones

- ✅ **v1.0.2 Dark/Light Mode** — Phases 1-5 (shipped 2026-03-05)
- ✅ **v1.0.6 Focus Management** — Phase 6 (shipped 2026-03-05)
- ✅ **v1.1.0 Sandbox Removal** — Phases 9-12 (shipped 2026-03-06)
- ✅ **v1.2.0 AI Chat Rewrite** — Phases 13-18 (shipped 2026-03-06)
- [ ] **v1.2.1-dev GSD Integration + Slash Commands** — Phases 19-23 (in progress)

## Phases

<details>
<summary>✅ v1.0.2 Dark/Light Mode (Phases 1-5) — SHIPPED 2026-03-05</summary>

- [x] Phase 1: Základ (4/4 plans) — completed 2026-03-04
- [x] Phase 2: Terminal + Git barvy (2/2 plans) — completed 2026-03-04
- [x] Phase 3: Light varianty + Settings UI (5/5 plans) — completed 2026-03-05
- [x] Phase 4: Infrastructure (2/2 plans) — completed 2026-03-05
- [x] Phase 5: Okamžité aplikování sandbox režimu (4/4 plans) — completed 2026-03-05

Archive: `.planning/milestones/v1.0.2-ROADMAP.md`

</details>

<details>
<summary>✅ v1.0.6 Focus Management (Phase 6) — SHIPPED 2026-03-05</summary>

- [x] Phase 6: Docked Terminal Focus Suppression (1/1 plans) — completed 2026-03-05
- ~~Phase 7: Float Terminal Focus Suppression~~ — cancelled (covered by Phase 6)
- ~~Phase 8: Focus Restore & Regression Verification~~ — cancelled (covered by Phase 6)

Archive: `.planning/milestones/v1.0.6-ROADMAP.md`

</details>

<details>
<summary>✅ v1.1.0 Sandbox Removal (Phases 9-12) — SHIPPED 2026-03-06</summary>

- [x] Phase 9: Core Sandbox Logic & Settings Removal (3/3 plans) — completed 2026-03-05
- [x] Phase 10: UI & State Cleanup (1/1 plans) — completed 2026-03-05
- [x] Phase 11: File Operations, Watcher & Guard Removal (2/2 plans) — completed 2026-03-05
- [x] Phase 12: I18n Cleanup & Integrity Verification (2/2 plans) — completed 2026-03-05

Archive: `.planning/milestones/v1.1.0-ROADMAP.md`

</details>

<details>
<summary>✅ v1.2.0 AI Chat Rewrite (Phases 13-18) — SHIPPED 2026-03-06</summary>

- [x] Phase 13: Provider Foundation (3/3 plans) — completed 2026-03-06
- [x] Phase 14: State Refactor (2/2 plans) — completed 2026-03-06
- [x] Phase 15: Streaming Chat UI (5/5 plans) — completed 2026-03-06
- [x] Phase 16: Tool Execution (4/4 plans) — completed 2026-03-06
- [x] Phase 17: i18n & WASM Cleanup (3/3 plans) — completed 2026-03-06
- [x] Phase 18: Phase 16 Verification & i18n Fixes (2/2 plans) — completed 2026-03-06

Archive: `.planning/milestones/v1.2.0-ROADMAP.md`

</details>

### v1.2.1-dev GSD Integration + Slash Commands (In Progress)

**Milestone Goal:** Slash command dispatch infrastruktura a kompletni port GSD workflow tools z Node.js do Rustu.

- [ ] **Phase 19: Slash Command Infrastructure** - Dispatch system, built-in commands (/help, /clear, /new, /model, /git, /build, /settings), error handling
- [ ] **Phase 20: GSD Core + State Engine** - Frontmatter parser, config management, path helpers, state/progress commands
- [ ] **Phase 21: GSD Operations** - Phase/roadmap management, verify/template/scaffold, milestone/commit/requirements commands
- [ ] **Phase 22: GSD Init + AI Integration** - Context aggregation commands, AI model delegation for content generation
- [ ] **Phase 23: Attribution + i18n** - MIT attribution in help/about, localization of all GSD strings in 5 languages

## Phase Details

### Phase 19: Slash Command Infrastructure
**Goal**: Users can interact with the editor through slash commands in the chat panel
**Depends on**: Phase 18 (existing chat infrastructure)
**Requirements**: SLASH-01, SLASH-02, SLASH-03, SLASH-04, SLASH-05, SLASH-06, SLASH-07, SLASH-08, SLASH-09
**Success Criteria** (what must be TRUE):
  1. User types `/help` in chat and sees a formatted list of all available commands with descriptions
  2. User types `/clear` and the conversation is cleared; `/new` resets conversation but keeps prompt history
  3. User types `/model` to list models or `/model <name>` to switch; `/git` shows git status; `/build` triggers cargo build; `/settings` opens settings dialog
  4. Typing `/` followed by an unregistered command shows an error with suggestions of similar commands
  5. Slash commands are intercepted before reaching the AI model -- no AI query is sent for recognized commands
**Plans:** 1/2 plans executed
Plans:
- [ ] 19-01-PLAN.md — Slash dispatch system, sync commands (/help, /clear, /new, /settings), system message rendering
- [ ] 19-02-PLAN.md — Async commands (/model, /git, /build) with background polling

### Phase 20: GSD Core + State Engine
**Goal**: Users can query and update GSD project state directly from the chat panel
**Depends on**: Phase 19
**Requirements**: CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, STATE-01, STATE-02, STATE-03, STATE-04, STATE-05
**Success Criteria** (what must be TRUE):
  1. GSD frontmatter parser correctly extracts and reconstructs YAML-like frontmatter from `.planning/` markdown files without data loss
  2. User runs `/gsd state` and sees current milestone, phase, plan, and progress formatted as markdown in chat
  3. User runs `/gsd state update` or `/gsd state patch` and the STATE.md file is updated on disk with the new values
  4. User runs `/gsd progress` and sees a visual progress bar with phase/plan completion counts
  5. When `.planning/` directory is missing, GSD commands show a helpful message instead of crashing
**Plans**: TBD

### Phase 21: GSD Operations
**Goal**: Users can manage phases, roadmap, verification, templates, milestones, and git commits through GSD commands
**Depends on**: Phase 20
**Requirements**: PHASE-01, PHASE-02, PHASE-03, PHASE-04, PHASE-05, PHASE-06, PHASE-07, VERIFY-01, VERIFY-02, VERIFY-03, VERIFY-04, VERIFY-05, MILE-01, MILE-02, MILE-03, MILE-04
**Success Criteria** (what must be TRUE):
  1. User runs `/gsd phase list` and sees all phases with status; `/gsd phase add`, `/gsd phase insert`, `/gsd phase remove`, `/gsd phase complete` modify ROADMAP.md correctly
  2. User runs `/gsd roadmap analyze` and sees full roadmap analysis; `/gsd roadmap get-phase N` extracts the specific phase section
  3. User runs `/gsd verify` subcommands and gets validation reports for plan structure, phase completeness, and `.planning/` health
  4. User runs `/gsd template fill` or `/gsd scaffold phase-dir` and correctly structured files/directories are created on disk
  5. User runs `/gsd commit` and a git commit is created asynchronously without freezing the UI; `/gsd milestone complete` archives the milestone; `/gsd requirements mark-complete` updates requirement status
**Plans**: TBD

### Phase 22: GSD Init + AI Integration
**Goal**: Users can aggregate project context and delegate it to the AI model for content generation
**Depends on**: Phase 21
**Requirements**: INIT-01, INIT-02, INIT-03, INIT-04, INIT-05, INIT-06
**Success Criteria** (what must be TRUE):
  1. User runs `/gsd init execute-phase N` or `/gsd init plan-phase N` and sees aggregated context from `.planning/` files displayed in chat
  2. User runs `/gsd init new-project` or `/gsd init quick <description>` and a bootstrapped project context is assembled
  3. When AI delegation is triggered, the aggregated context is injected into the Ollama provider's system prompt and the AI generates content using existing streaming infrastructure
**Plans**: TBD

### Phase 23: Attribution + i18n
**Goal**: GSD integration is properly attributed and all user-facing strings are localized
**Depends on**: Phase 22
**Requirements**: ATTR-01, ATTR-02, I18N-01, I18N-02
**Success Criteria** (what must be TRUE):
  1. `/gsd help` output includes MIT attribution notice (Copyright Lex Christopherson) at the bottom
  2. About dialog mentions GSD integration with MIT license attribution
  3. All GSD slash command descriptions, error messages, and UI chrome are localized in all 5 languages (cs, en, de, ru, sk); GSD data output remains in English
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Základ | v1.0.2 | 4/4 | Complete | 2026-03-04 |
| 2. Terminal + Git barvy | v1.0.2 | 2/2 | Complete | 2026-03-04 |
| 3. Light varianty + Settings UI | v1.0.2 | 5/5 | Complete | 2026-03-05 |
| 4. Infrastructure | v1.0.2 | 2/2 | Complete | 2026-03-05 |
| 5. Okamžité aplikování sandbox režimu | v1.0.2 | 4/4 | Complete | 2026-03-05 |
| 6. Docked Terminal Focus Suppression | v1.0.6 | 1/1 | Complete | 2026-03-05 |
| ~~7. Float Terminal Focus Suppression~~ | v1.0.6 | — | Cancelled | — |
| ~~8. Focus Restore & Regression~~ | v1.0.6 | — | Cancelled | — |
| 9. Core Sandbox Logic & Settings Removal | v1.1.0 | 3/3 | Complete | 2026-03-05 |
| 10. UI & State Cleanup | v1.1.0 | 1/1 | Complete | 2026-03-05 |
| 11. File Operations, Watcher & Guard Removal | v1.1.0 | 2/2 | Complete | 2026-03-05 |
| 12. I18n Cleanup & Integrity Verification | v1.1.0 | 2/2 | Complete | 2026-03-05 |
| 13. Provider Foundation | v1.2.0 | 3/3 | Complete | 2026-03-06 |
| 14. State Refactor | v1.2.0 | 2/2 | Complete | 2026-03-06 |
| 15. Streaming Chat UI | v1.2.0 | 5/5 | Complete | 2026-03-06 |
| 16. Tool Execution | v1.2.0 | 4/4 | Complete | 2026-03-06 |
| 17. i18n & WASM Cleanup | v1.2.0 | 3/3 | Complete | 2026-03-06 |
| 18. Phase 16 Verification & i18n Fixes | v1.2.0 | 2/2 | Complete | 2026-03-06 |
| 19. Slash Command Infrastructure | 1/2 | In Progress|  | - |
| 20. GSD Core + State Engine | v1.2.1-dev | 0/? | Not started | - |
| 21. GSD Operations | v1.2.1-dev | 0/? | Not started | - |
| 22. GSD Init + AI Integration | v1.2.1-dev | 0/? | Not started | - |
| 23. Attribution + i18n | v1.2.1-dev | 0/? | Not started | - |

## Known Issues / TODO

- **Syntax highlighting v AI chatu**: `egui_commonmark` s `better_syntax_highlighting` feature a `syntax_theme_dark("base16-ocean.dark")` nefunguje — code blocky (```rust) se zobrazuji cernobile bez barev. Nutno vyresit — mozna vlastni rendering code bloku pres syntect (uz pouzivame v editoru).
