---
phase: 37-trash-preview-restore-mvp
plan: 01
subsystem: api
tags: [trash, restore, serde_json, fail-closed, rust]
requires:
  - phase: 36-safe-move-to-trash-engine
    provides: move-to-trash engine and delete fail-closed baseline
provides:
  - trash preview list engine API with metadata status contract
  - single-item restore engine contract with structured outcome
  - fail-closed restore error mapping suitable for toast propagation
affects: [trash-preview-modal, restore-flow, file-tree-toast]
tech-stack:
  added: []
  patterns: [metadata sidecar contract, fail-closed restore rollback, newest-first trash ordering]
key-files:
  created: [.planning/phases/37-trash-preview-restore-mvp/37-01-SUMMARY.md]
  modified: [src/app/trash.rs, src/app/project_config.rs, tests/phase37_restore_engine.rs, tests/phase36_toast_propagation.rs]
key-decisions:
  - "Trash preview listuje engine API z .polycredo/trash a metadata validuje explicitnim stavem valid/missing/invalid."
  - "Restore je strictne fail-closed: bez silent overwrite, s create_dir_all parentu a rollbackem pri cleanup selhani."
patterns-established:
  - "Restore error contract: vsechny restore vetve mapuji na prefix `restore selhal:` pro jednotne toast mapovani."
  - "Phase drift compatibility: historicke guard testy zustavaji aktivni, ale neblokuji scope schvalene v nove fazi."
requirements-completed: [TRASHUI-01, RESTORE-01]
duration: 6min
completed: 2026-03-12
---

# Phase 37 Plan 01: Engine Preview + Restore Summary

**Trash engine nyní poskytuje preview-ready list API s metadata stavem a fail-closed restore jedné položky na původní cestu včetně parent adresářů.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-12T11:51:47Z
- **Completed:** 2026-03-12T11:57:35Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- Přidán `list_trash_entries` kontrakt pro preview s řazením od nejnovějších položek.
- Přidán `restore_from_trash` kontrakt s `TrashRestoreOutcome` a nedestruktivním conflict guardem.
- Doložen fail-closed behavior testy pro invalid metadata, missing source a I/O parent failure.

## Task Commits

Each task was committed atomically:

1. **Task 1: Engine API pro listování trash preview** - `518b52b`, `c4c6bc6` (test, feat)
2. **Task 2: Single-item restore happy path + parent create** - `c0cab7e`, `582f2f8` (test, feat)
3. **Task 3: Fail-closed restore error kontrakty** - `3d714b4`, `927a43f`, `67cab74` (test, fix, fix)

## Files Created/Modified
- `src/app/trash.rs` - list/restore engine API, metadata kontrakty, fail-closed error flow
- `src/app/project_config.rs` - helper `trash_meta_path` pro stabilní metadata cestu
- `tests/phase37_restore_engine.rs` - focused req-level testy pro list, restore happy path, parent create, fail-closed
- `tests/phase36_toast_propagation.rs` - kompatibilní phase36 scope guard po zavedení phase37 restore symbolů

## Decisions Made
- Metadata sidecar je autoritativní zdroj pro restore mapování; heuristiky názvu nejsou použity.
- Restore konflikt je hard stop s message pro následný "restore jako kopii" UX krok.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Historický phase36 scope guard blokoval phase37 restore API**
- **Found during:** Task 3 (final quality gate `./check.sh`)
- **Issue:** `tests/phase36_toast_propagation.rs` zakazoval symbol `restore` v `src/app/trash.rs`, což po phase37 validně neplatí.
- **Fix:** Guard byl upraven tak, aby stále chránil phase36 scope bez blokace explicitně schváleného phase37 rozsahu.
- **Files modified:** `tests/phase36_toast_propagation.rs`
- **Verification:** `RUSTC_WRAPPER= ./check.sh` PASS
- **Committed in:** `67cab74`

---

**Total deviations:** 1 auto-fixed (Rule 3: 1)
**Impact on plan:** Bez scope creep; změna byla nutná pro průchod quality gate při zachování konzistence mezi phase36 a phase37.

## Issues Encountered
- Lokální prostředí blokovalo `sccache` (`Operation not permitted`), všechny Rust příkazy byly spuštěny s `RUSTC_WRAPPER=`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Engine kontrakty pro preview + restore jsou připravené pro navazující modal UI orchestration.
- Test hooky jsou přítomné pro list, restore happy path i robustní fail-closed branch.

## Self-Check: PASSED
- FOUND: `.planning/phases/37-trash-preview-restore-mvp/37-01-SUMMARY.md`
- FOUND COMMIT: `518b52b`
- FOUND COMMIT: `c4c6bc6`
- FOUND COMMIT: `c0cab7e`
- FOUND COMMIT: `582f2f8`
- FOUND COMMIT: `3d714b4`
- FOUND COMMIT: `927a43f`
- FOUND COMMIT: `67cab74`
