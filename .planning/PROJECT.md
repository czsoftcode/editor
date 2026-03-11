# PolyCredo Editor

## What This Is

Multiplatformni textovy editor v Rustu (eframe/egui) s terminaly, build workflow a AI terminal panelem. Produkt je local-first a ma zustat responzivni i pri delsi praci.

## Core Value

Editor nesmi zahrivat notebook v klidovem stavu - idle CPU zatez musi byt minimalni.

## Current State

- **Shipped version:** v1.3.0 AI Terminal Cleanup (2026-03-11)
- **Milestone result:** `src/app/cli/*` odstraneno, aktivni AI tok je launcher-only (`ai_bar -> terminal.send_command`)
- **Quality status:** milestone audit `passed` (requirements 15/15, phase verification 4/4, flows 5/5)
- **Primary artifacts:** `.planning/milestones/v1.3.0-ROADMAP.md`, `.planning/milestones/v1.3.0-REQUIREMENTS.md`, `.planning/milestones/v1.3.0-MILESTONE-AUDIT.md`

## Requirements

### Validated

- ✓ v1.3.0: CLI cleanup + AI terminal-only boundary + traceability closure (R33-A/R33-B/R33-C/R33-D)
- ✓ v1.2.2: Additional Themes (WarmTan + Midnight varianty, syntect mapovani, i18n)
- ✓ v1.2.1: Save Modes + Unsaved Changes Guard
- ✓ v1.2.0: AI Chat Rewrite baseline

### Active

- (none yet - next milestone requirements budou zalozeny pres `$gsd-new-milestone`)

### Out of Scope

- Pridavani novych AI provideru bez jasneho produktoveho cile.
- Velke refaktory mimo konkretni milestone scope.

## Next Milestone Goals

- Zalozit novy milestone scope podle aktualnich priorit produktu.
- Definovat nove requirements a traceability mapu od nuly.
- Udrzet quality gate standard: `cargo check` + `./check.sh` pro kazdou fazi.

## Context

**Known tech debt:**
- Warning text kontrast v light mode (Settings modal)
- Syntax highlighting v AI chatu (egui_commonmark code blocky)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Odstranit `src/app/cli/*` v milestone v1.3.0 | CLI vrstva byla slepa ulicka proti smeru produktu | ✓ Implemented v1.3.0 |
| Zachovat assistant-only AI terminal boundary | Minimalizace couplingu a regresi po cleanupu | ✓ Implemented v1.3.0 |
| Delit cleanup do fazi 30-34 | Kontrolovatelna verifikace a mensi riziko | ✓ Implemented v1.3.0 |

<details>
<summary>Archived milestone context (v1.3.0 planning snapshot)</summary>

- Original milestone goal: odstranit legacy CLI vrstvu pri zachovani AI terminal behavior.
- Closure phase: 34 (milestone gap closure and traceability rebaseline).
- Final audit verdict: passed.

</details>

---
*Last updated: 2026-03-11 after completing milestone v1.3.0*
