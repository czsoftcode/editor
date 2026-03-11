---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: milestone
status: in_progress
last_updated: "2026-03-11T11:28:35.235Z"
last_activity: 2026-03-11 - Completed 31-01-PLAN.md
progress:
  total_phases: 3
  completed_phases: 1
  total_plans: 8
  completed_plans: 5
  percent: 62
---

## gsd_state_version: 1.0

## Current Milestone: v1.3.0 AI Terminal Cleanup

**Goal:** Odstranit PolyCredo CLI vrstvu (`src/app/cli/*`) a ponechat pouze AI terminal bez regresi.

**Target features:**
- Odstraneni `src/app/cli/*` a navazanych importu
- Zachovani AI terminal chat + streaming + model picker + slash/GSD
- Zachovani approval/security guardu pri AI operacich

**Status:** In progress

---

## Project Reference

See: .planning/PROJECT.md

**Core value:** Editor nesmi zahrivat notebook v klidovem stavu - idle CPU zatez musi byt minimalni.
**Current focus:** Phase 31 (AI terminal runtime migration)

## Current Position

Phase: 31-ai-terminal-runtime-migration
Plan: 31-01 completed
Status: 31-01 completed, 31-02 až 31-04 pending
Last activity: 2026-03-11 - Completed 31-01-PLAN.md

Progress: [██████░░░░] 62%

---

## Accumulated Context

### Decisions

- [v1.3.0]: `src/app/cli/*` je mimo dalsi smer produktu, zustane pouze AI terminal workflow.
- [v1.3.0]: Cleanup bude rozdelen do fazi 30-32 kvuli kontrolovane migraci.
- [Phase 30]: Public root export switched from app::cli to app::ai_core with internal cli visibility retained for staged migration.
- [Phase 30]: Foundation migration scope locked to settings/types/workspace-state; runtime AI terminal migration deferred to next plans.
- [Phase 30-cli-namespace-removal-foundation]: AI terminal head/right bar byl uzamcen do assistant-only UX bez provider model/status prvku.
- [Phase 30-cli-namespace-removal-foundation]: CLI-02 audit scope byl docisten i o ai_chat/slash.rs, protoze je soucasti overovaneho subsetu.
- [Phase 30]: AiManager a runtime AI vrstva byly presunuty do app::ai_core a app::cli namespace byl odstraneny bez fallback aliasu.
- [Phase 30]: CLI-01 je dokumentovan explicitnim 30-02-AUDIT.md artefaktem s build a grep PASS dukazy.
- [Phase 30-cli-namespace-removal-foundation]: Public API app root bylo zúženo na ai_core; ostatní moduly jsou interní (pub(crate)).
- [Phase 30-cli-namespace-removal-foundation]: Task 2 byl potvrzen samostatným audit commitem bez změny kódu kvůli atomickému task trace.
- [Phase 31-ai-terminal-runtime-migration]: Retry flow je explicitni UI akce vazana na posledni validni prompt, aktivovana jen po runtime chybe.
- [Phase 31-ai-terminal-runtime-migration]: Slash async stale-guard je sjednoceny jednim helperem pro /build i /git.

### Known Tech Debt

- Warning text kontrast v light mode (Settings modal)
- Syntax highlighting v AI chatu nefunguje (egui_commonmark code blocky cernobile)

### Pending Todos

- Opravit kontrast warning textu v light mode (`modal_dialogs/settings.rs`)
- Upravit dolni listu: branch oznameni vice doprava, UTF vice doleva (`src/app/ui/terminal/bottom/git_bar.rs`)
- V `Sestavit > Upravit` pridat online nahled zmen zapisovanych do `.polycredo/profiles.toml`
- Zprovoznit `.polycredo/trash` a presouvat tam smazane soubory

### Blockers/Concerns

- Migrace approval/security casti bez regresi muze odhalit skryte couplingy mimo `src/app/cli/*`.

---

## Quick Tasks Completed

| #  | Description | Date | Commit | Directory |
|----|-------------|------|--------|-----------|
| ... | (pokracovani z historie) | | | |
| 9 | GitHub Actions Windows build: localtime + warningy | 2026-03-11 | f3ba79e | .planning/quick/9-github-action-windows-build-fix-localtim |

---
*Last updated: 2026-03-11*
