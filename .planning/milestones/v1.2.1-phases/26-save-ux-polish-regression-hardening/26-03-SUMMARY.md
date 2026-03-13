---
phase: 26-save-ux-polish-regression-hardening
plan: 03
subsystem: testing
tags: [save, ctrl-s, unsaved-close-guard, dedupe, regression]
requires:
  - phase: 26-01
    provides: save mode routing baseline and MODE-04 runtime status coverage
provides:
  - Regression lock for Ctrl+S branch routing (modified/clean/no-active/settings)
  - Guard save-failure contract with inline+toast+no-close assertions
  - Explicit dedupe window classification tests for 1.5s contract
affects: [phase-26-04, mode-04-regressions, save-feedback-contract]
tech-stack:
  added: []
  patterns:
    - TDD red/green commits per task with regression-focused assertions
    - Extracted pure helpers for deterministic guard/save branch testing
key-files:
  created: []
  modified:
    - src/app/ui/workspace/mod.rs
    - src/app/ui/workspace/tests/unsaved_close_guard.rs
    - src/app/types.rs
    - src/app/ui/editor/mod.rs
    - src/app/ui/editor/render/tabs.rs
key-decisions:
  - "Ctrl+S routing is mediated through manual_save_request_for_shortcut to keep branch mapping deterministic."
  - "Guard save-failure handling is centralized to keep inline error, toast feedback, and close eligibility testable."
  - "Save error dedupe uses an explicit within-window classifier while preserving existing 1.5s semantics."
patterns-established:
  - "Guard flow close decision is asserted via helper against save_result (Save+Err never closes)."
  - "Dedupe boundary (exact 1500ms) is treated as inside suppression window."
requirements-completed: [MODE-04]
duration: 6 min
completed: 2026-03-10
---

# Phase 26 Plan 03: Save Feedback Regression Pack Summary

**Regression pack uzamyka Ctrl+S routing, guard save-failure UX kontrakt a 1.5s save-error dedupe semantiku přes TDD testy bez změny externího chování.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-10T19:34:46Z
- **Completed:** 2026-03-10T19:41:23Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Doplněné branch testy pro Ctrl+S pokrývají settings-draft, modified, clean i no-active-tab větev.
- Guard save-failure flow má explicitní regresní assert na inline chybu, toast a zákaz předčasného close.
- Dedupe kontrakt má explicitní klasifikaci uvnitř/vně 1.5s okna včetně hraniční hodnoty.

## Task Commits

1. **Task 1: Kontraktní testy Ctrl+S větví ve workspace**
2. `1baec74` (`test`) RED
3. `db59e7c` (`feat`) GREEN
4. **Task 2: Guard save fail kontrakt inline + toast + no-close**
5. `8fa6ccd` (`test`) RED
6. `7d426a9` (`feat`) GREEN
7. **Task 3: Uzamčení dedupe kontraktu save chyb (1.5s)**
8. `c744b46` (`test`) RED
9. `f717a88` (`feat`) GREEN

## Files Created/Modified
- `src/app/ui/workspace/mod.rs` - helpery pro deterministic routing a guard save-failure decision flow.
- `src/app/ui/workspace/tests/unsaved_close_guard.rs` - regresní test `unsaved_close_guard_save_failure_feedback`.
- `src/app/types.rs` - explicitní `is_within_save_error_dedupe_window` + nové dedupe window testy.
- `src/app/ui/editor/mod.rs` - minimální unblock fix (`SaveStatus: Copy`) kvůli preexistující test kompilaci.
- `src/app/ui/editor/render/tabs.rs` - test-only export helperu pro preexistující `save_mode` test import.

## Decisions Made
- Guard save-failure branch se testuje přes čisté helper funkce místo frame-dependent UI interakce.
- Dedupe rozhodnutí je oddělené na classifier + decision wrapper pro čitelnější regresní testy.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] sccache wrapper blokoval spuštění cargo test**
- **Found during:** Task 1 verification
- **Issue:** `sccache` vracel `Operation not permitted`.
- **Fix:** Verifikace spuštěny konzistentně s `RUSTC_WRAPPER=""`.
- **Files modified:** none
- **Verification:** všechny požadované `cargo test`/`cargo check` doběhly.
- **Committed in:** N/A (runtime execution fix)

**2. [Rule 3 - Blocking] Preexistující kompilace test targetu mimo scope tasku**
- **Found during:** Task 1/2 verification
- **Issue:** build blokovaly chyby nesouvisející s plánem (`SaveStatus` move, chybějící test helper import).
- **Fix:** minimální kompatibilní unblock (`SaveStatus` jako `Copy`; test-only helper export pro tab label).
- **Files modified:** `src/app/ui/editor/mod.rs`, `src/app/ui/editor/render/tabs.rs`
- **Verification:** následné task verifikace proběhly zeleně.
- **Committed in:** `db59e7c`, `7d426a9`

---

**Total deviations:** 2 auto-fixed (2x Rule 3 - Blocking)
**Impact on plan:** pouze unblock změny nutné pro spuštění plánových testů; funkční scope plánu zachován.

## Issues Encountered
- `./check.sh` (informational) hlásí repo-wide `cargo fmt --check` drift mimo scope tohoto plánu.

## Authentication Gates
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plan 26-03 je uzavřený, regresní kontrakty save feedback flow jsou stabilní.
- Ready for `26-04-PLAN.md`.

---
*Phase: 26-save-ux-polish-regression-hardening*
*Completed: 2026-03-10*

## Self-Check: PASSED

```text
FOUND: .planning/phases/26-save-ux-polish-regression-hardening/26-03-SUMMARY.md
FOUND COMMIT: 1baec74
FOUND COMMIT: db59e7c
FOUND COMMIT: 8fa6ccd
FOUND COMMIT: 7d426a9
FOUND COMMIT: c744b46
FOUND COMMIT: f717a88
```
