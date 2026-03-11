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
| Complete    | 2026-03-11 |

Coverage: 11/11 requirements mapped.

### Phase 33: odstranit veskerou zminku a funkce polycredo cli ze systemu

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 32
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd:plan-phase 33 to break down)
