---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: milestone
status: executing
last_updated: "2026-03-11T09:53:37Z"
last_activity: "2026-03-11 - Completed 30-01-PLAN.md"
progress:
  total_phases: 3
  completed_phases: 0
  total_plans: 4
  completed_plans: 1
  percent: 25
---

## gsd_state_version: 1.0

## Current Milestone: v1.3.0 AI Terminal Cleanup

**Goal:** Odstranit PolyCredo CLI vrstvu (`src/app/cli/*`) a ponechat pouze AI terminal bez regresi.

**Target features:**
- Odstraneni `src/app/cli/*` a navazanych importu
- Zachovani AI terminal chat + streaming + model picker + slash/GSD
- Zachovani approval/security guardu pri AI operacich

**Status:** Executing phase plans

---

## Project Reference

See: .planning/PROJECT.md

**Core value:** Editor nesmi zahrivat notebook v klidovem stavu - idle CPU zatez musi byt minimalni.
**Current focus:** Plan phase 30 (CLI namespace removal foundation)

## Current Position

Phase: 30-cli-namespace-removal-foundation
Plan: 30-02 next
Status: 30-01 completed
Last activity: 2026-03-11 - Completed 30-01-PLAN.md

Progress: [██--------] 25%

---

## Accumulated Context

### Decisions

- [v1.3.0]: `src/app/cli/*` je mimo dalsi smer produktu, zustane pouze AI terminal workflow.
- [v1.3.0]: Cleanup bude rozdelen do fazi 30-32 kvuli kontrolovane migraci.
- [Phase 30]: Public root export switched from app::cli to app::ai_core with internal cli visibility retained for staged migration.
- [Phase 30]: Foundation migration scope locked to settings/types/workspace-state; runtime AI terminal migration deferred to next plans.

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
