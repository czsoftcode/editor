---
phase: 36-safe-move-to-trash-engine
plan: 01
subsystem: testing
tags: [rust, trash, move-to-trash, fail-closed, egui]
requires:
  - phase: 35-trash-foundation-async-safety
    provides: async delete dispatch and initial move_path_to_trash foundation
provides:
  - explicit engine guard blocking delete operations inside `.polycredo/trash`
  - stable move-to-trash destination resolution contract with timestamp+counter collision policy
  - fail-closed move error formatter with actionable toast-ready text
  - regression tests for TRASH-01, TRASH-02, TRASH-04 and guard behavior
affects: [phase-37-trash-preview-restore, delete-flow, toast-propagation]
tech-stack:
  added: []
  patterns: [tdd-red-green for contract tests, fail-closed io error surfacing]
key-files:
  created: [tests/phase36_move_to_trash.rs]
  modified: [src/app/trash.rs, src/app/project_config.rs]
key-decisions:
  - "Engine-level guard blocks both `.polycredo/trash` root and any nested path to avoid UI-only protection."
  - "Move failures use one formatter to keep fail-closed wording and next-step guidance consistent."
patterns-established:
  - "Delete path contracts are enforced by source-level regression tests per requirement."
  - "No hard-delete fallback is validated alongside move-to-trash behavior."
requirements-completed: [TRASH-01, TRASH-02, TRASH-04, RELIAB-02]
duration: 3m 25s
completed: 2026-03-12
---

# Phase 36 Plan 01: Safe Move-to-Trash Engine Summary

**Move-to-trash delete engine now blocks internal trash deletes, keeps deterministic collision-safe target naming, and enforces fail-closed I/O error messaging.**

## Performance

- **Duration:** 3m 25s
- **Started:** 2026-03-12T09:40:54Z
- **Completed:** 2026-03-12T09:44:19Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- Přidán explicitní guard proti mazání `.polycredo/trash` v engine vrstvě, včetně podcest.
- Sjednocen kontrakt pro cílovou trash cestu (`resolve_trash_destination`) a stabilní helper `project_trash_dir`.
- Doplněn fail-closed formatter chyby a regresní testy `phase36_*` pro file/dir/collision/failure scénáře.

## Task Commits

Each task was committed atomically:

1. **Task 1: Explicitni engine guard proti mazani `.polycredo/trash`** - `b7a0529`, `141c829` (test + feat)
2. **Task 2: Zpevnit move-to-trash kontrakt pro soubor/adresar + kolize** - `48a0126`, `fcdd818` (test + feat)
3. **Task 3: Fail-closed dukaz pro TRASH-04** - `25c9b22`, `11f66c3` (test + fix)

**Additional gate fix:** `9de7304` (chore; formatting needed for `./check.sh`)

## Files Created/Modified
- `tests/phase36_move_to_trash.rs` - requirement-level contract testy pro phase 36 delete engine.
- `src/app/trash.rs` - guardy interniho trash prostoru, destination resolver, fail-closed error formatter.
- `src/app/project_config.rs` - stabilni helper `project_trash_dir` pro interni trash path.

## Decisions Made
- Guard je vynucen v engine vrstve, ne jen ve file tree skryti `.polycredo`, aby se zabránilo obejití přes jiné entrypointy.
- Chybové hlášky pro move failure jsou centralizované a toast-ready (důvod + doporučený další krok).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `./check.sh` failed on formatting**
- **Found during:** Final verification
- **Issue:** Nový test soubor `tests/phase36_move_to_trash.rs` nebyl ve formátu `cargo fmt`.
- **Fix:** Spuštěno `cargo fmt --all` a znovu ověřena celá quality gate.
- **Files modified:** tests/phase36_move_to_trash.rs
- **Verification:** `cargo test phase36_move_to_trash -- --nocapture`, `cargo check`, `./check.sh`
- **Committed in:** 9de7304

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Bez scope creep; šlo o nutnou gate opravu.

## Issues Encountered
- `rg` kontrola hard-delete bez matchů vrátila exit code 1; ověřeno samostatným guard commandem s explicitním PASS výstupem.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Engine kontrakty pro safe delete jsou připravené pro phase 37 restore/trash preview navázání.
- Požadavky TRASH-01/TRASH-02/TRASH-04/RELIAB-02 mají testové důkazy v jednom regresním souboru.

## Self-Check: PASSED

- FOUND: `.planning/phases/36-safe-move-to-trash-engine/36-01-SUMMARY.md`
- FOUND commits: `b7a0529`, `141c829`, `48a0126`, `fcdd818`, `25c9b22`, `11f66c3`, `9de7304`

---
*Phase: 36-safe-move-to-trash-engine*
*Completed: 2026-03-12*
