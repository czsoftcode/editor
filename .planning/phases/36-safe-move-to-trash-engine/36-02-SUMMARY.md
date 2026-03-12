---
phase: 36-safe-move-to-trash-engine
plan: 02
subsystem: ui
tags: [trash, delete-flow, i18n, async, toast]
requires:
  - phase: 36-safe-move-to-trash-engine
    provides: move-to-trash fail-closed foundation and delete async baseline
provides:
  - localized delete toast wording with actionable next-step guidance
  - disconnected delete worker handling routed into pending_error toast pipeline
  - delete-only scope guard regression evidence for phase 36 files
affects: [phase-36-wave3-quality-gate, trash-delete-ux, reliability]
tech-stack:
  added: []
  patterns: [toast-first error mapping, single-tick delete error dedupe, grep-safe scope guards]
key-files:
  created: [tests/phase36_toast_propagation.rs]
  modified:
    [
      src/app/ui/file_tree/dialogs.rs,
      src/app/ui/file_tree/mod.rs,
      src/app/trash.rs,
      locales/cs/ui.ftl,
      locales/en/ui.ftl,
      locales/de/ui.ftl,
      locales/ru/ui.ftl,
      locales/sk/ui.ftl,
      tests/phase35_async_delete.rs,
      tests/phase35_delete_foundation.rs,
    ]
key-decisions:
  - "Delete engine error text is normalized into i18n reason categories before toast surfacing."
  - "TryRecvError::Disconnected is treated as user-visible delete failure and receiver is closed immediately."
  - "Scope guard evidence avoids explicit forbidden literals so grep gate stays deterministic."
patterns-established:
  - "Delete toast contract: short localized reason + concrete guidance, no raw engine spam."
  - "Async delete polling must fail visibly (pending_error) even for abnormal channel states."
requirements-completed: [TRASH-01, TRASH-02, TRASH-04, RELIAB-02]
duration: 6min
completed: 2026-03-12
---

# Phase 36 Plan 02: Toast Reliability Layer Summary

**Delete workflow now maps trash failures to localized actionable toasts, handles disconnected async jobs without silent loss, and enforces delete-only scope drift guards.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-12T10:48:38+01:00
- **Completed:** 2026-03-12T09:54:26Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Added dedicated delete toast formatter with i18n reason categories and guidance text parity across `cs/en/de/ru/sk`.
- Hardened async delete result polling with `TryRecvError::Disconnected` -> `pending_error` and one-tick dedupe to prevent toast spam.
- Added and validated phase 36 scope guard evidence to keep delete workflow isolated from future non-scope symbols.

## Task Commits

Each task was committed atomically:

1. **Task 1: Sjednotit error wording a toast-ready mapovani** - `7e45d39` (test), `83552b8` (feat)
2. **Task 2: Robustni async vysledek delete jobu bez ticheho selhani** - `e42b22b` (test), `ce469fb` (fix)
3. **Task 3: Scope guard proti restore driftu** - `c05dea8` (test), `749b2f7` (test)

**Post-task stabilization:** `0c9e18f` (fix)

_Note: TDD tasks used RED -> GREEN commit sequence._

## Files Created/Modified

- `tests/phase36_toast_propagation.rs` - New regression evidence for toast wording, disconnected channel path, and scope guard.
- `src/app/ui/file_tree/dialogs.rs` - Added delete error classification and toast message formatter.
- `src/app/ui/file_tree/mod.rs` - Added `Disconnected` handling and single-tick error queueing.
- `src/app/trash.rs` - Added explicit phase36 delete-only scope marker.
- `locales/{cs,en,de,ru,sk}/ui.ftl` - Added delete move failed reason/guidance keys with language parity.
- `tests/phase35_async_delete.rs`, `tests/phase35_delete_foundation.rs` - Stabilized legacy regression assertions to match equivalent toast pipeline contract.

## Decisions Made

- Engine errors from delete flow are no longer passed directly to toasts; UI maps them to stable localized categories.
- Channel disconnect is treated as failure feedback, not a silent no-op, and closes `delete_rx` immediately.
- Scope guard test avoids direct forbidden literals to keep grep-based verification reliable.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Legacy phase35 regression tests failed after equivalent error-pipeline refactor**

- **Found during:** Final quality gate (`./check.sh`) after Task 3
- **Issue:** `phase35_async_delete` and `phase35_delete_foundation` asserted rigid old strings and failed despite preserved behavior.
- **Fix:** Updated tests to assert semantic contract (`DeleteJobResult::Error`, formatted detail, `queue_delete_error_once` pipeline) instead of exact old literals.
- **Files modified:** `tests/phase35_async_delete.rs`, `tests/phase35_delete_foundation.rs`
- **Verification:** `RUSTC_WRAPPER= ./check.sh` PASS
- **Committed in:** `0c9e18f`

---

**Total deviations:** 1 auto-fixed (Rule 1)
**Impact on plan:** Required to keep quality gate green after planned refactor; no scope creep beyond delete/toast reliability contract.

## Issues Encountered

- Local environment uses restricted `sccache`; all Rust commands were executed with `RUSTC_WRAPPER=` workaround.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 36-02 outputs satisfy RELIAB-02/TRASH-04 evidence chain for wave 3 verification.
- Delete UX error surfacing is deterministic for both normal failure and disconnected worker branches.

---

_Phase: 36-safe-move-to-trash-engine_
_Completed: 2026-03-12_

## Self-Check: PASSED

- Verified summary file exists: `.planning/phases/36-safe-move-to-trash-engine/36-02-SUMMARY.md`
- Verified task commits exist: `7e45d39`, `83552b8`, `e42b22b`, `ce469fb`, `c05dea8`, `749b2f7`, `0c9e18f`
