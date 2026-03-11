# ROADMAP: PolyCredo Editor - Milestone v1.3.0

**Milestone:** v1.3.0 AI Terminal Cleanup
**Goal:** Odstranit PolyCredo CLI vrstvu (`src/app/cli/*`) a nechat jen AI terminal bez regresi chovani.
**Date:** 2026-03-11

## Progress

- Total phases: 3
- Completed: 3
- In progress: 0
- Pending: 0

## Phases

### Phase 30: CLI Namespace Removal Foundation

**Goal:** Odstranit vazby na `app::cli` namespace a pripravit cisty zaklad pro AI terminal-only architekturu.

**Requirements:** CLI-01, CLI-02, CLI-03

**Success criteria:**
- Vsechny primarni importy `crate::app::cli::*` jsou premapovane na novy modul/namespace.
- Build prochazi bez odkazu na odstranene `app::cli` soubory.
- `src/app/cli/*` lze smazat bez kompilacnich chyb v zakladnim buildu.

### Phase 31: AI Terminal Runtime Migration

**Goal:** Migrovat runtime/executor/tooling casti tak, aby AI terminal fungoval v assistant-only rezimu bez puvodni CLI vrstvy.

**Requirements:** TERM-01, TERM-02, TERM-03, SAFE-01, SAFE-02, SAFE-03

**Success criteria:**
- AI terminal panel jde otevrit, odesilat dotazy a prijimat streaming odpovedi.
- Assistant-only prompt/stream + slash/GSD workflow zustava funkcni bez provider-picker couplingu.
- Approval a security flow funguje stejne jako pred migraci.

### Phase 32: Cleanup, Tests, and Stabilization

**Goal:** Dokoncit cleanup, testy a dokumentaci po odstraneni CLI vrstvy.

**Requirements:** STAB-01, STAB-02
**Verification evidence:** `.planning/phases/32-cleanup-tests-and-stabilization/32-VERIFICATION.md`

**Success criteria:**
- `cargo check` a `./check.sh` prochazi.
- Relevantni testy jsou aktualizovane na novy namespace.
- Docs/planning artefakty reflektuji AI terminal-only architekturu.

## Requirement Coverage

| Requirement | Phase |
|-------------|-------|
| CLI-01 | 30 |
| CLI-02 | 30 |
| CLI-03 | 30 |
| TERM-01 | 31 |
| TERM-02 | 31 |
| TERM-03 | 31 |
| SAFE-01 | 31 |
| SAFE-02 | 31 |
| SAFE-03 | 31 |
| STAB-01 | 32 |
| STAB-02 | 32 |
| R33-A | 33 |
| R33-B | 33 |
| R33-C | 33 |
| R33-D | 33 |

Coverage: 15/15 requirements mapped.

### Phase 33: odstranit veskerou zminku a funkce polycredo cli ze systemu

**Goal:** Odstranit integrovany AI runtime/chat subsystem (`ai_core` + `ui/terminal/ai_chat`) a ponechat pouze `ai_bar` launcher tok do terminalu bez fallback UX.
**Requirements**: R33-A, R33-B, R33-C, R33-D
**Depends on:** Phase 32
**Plans:** 4 plans

Plans:
- [ ] 33-01-PLAN.md — hard removal ai_core + ai_chat + compile gate
- [ ] 33-02-PLAN.md — i18n/no-fallback UI cleanup
- [ ] 33-03-PLAN.md — active planning cleanup + verification bridge
- [ ] 33-04-PLAN.md — global/historical planning cleanup batches
