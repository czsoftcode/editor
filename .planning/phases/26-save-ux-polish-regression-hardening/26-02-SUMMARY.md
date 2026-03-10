---
phase: 26-save-ux-polish-regression-hardening
plan: 02
subsystem: ui
tags: [save-ux, egui, regression-tests, status-bar]
requires:
  - phase: 26-01
    provides: Save mode runtime indikace a MODE-04 baseline
provides:
  - Dirty-first vizuální priorita v editor status baru
  - Regression testy pro save UX kontrast/prioritu v save_mode scénářích
affects: [workspace-save-flow, tab-indicators, mode-04]
tech-stack:
  added: []
  patterns: [TDD test-first pro UI kontrakt, status prezentace přes explicitní mapping]
key-files:
  created: []
  modified:
    - src/app/ui/editor/ui.rs
    - src/app/ui/workspace/tests/save_mode.rs
key-decisions:
  - "Dirty stav (`statusbar-unsaved`) zůstává primární signál přes `is_primary` mapování a strong render."
  - "Save UX regression guard je ukotven v `save_mode.rs` nad runtime mode key + dirty/mode tab prioritou."
patterns-established:
  - "Save status texty zůstávají semanticky beze změny (`unsaved/saving/saved`), mění se jen vizuální priorita."
  - "Kontrastní regression testy se filtrují přes `save_ux_contrast_regression` a `dirty_state_visual_priority`."
requirements-completed: [MODE-04]
duration: 6min
completed: 2026-03-10
---

# Phase 26 Plan 02: Save UX Priority and Contrast Guards Summary

**Dirty stav je nyní explicitně renderovaný jako primární status signál a save UX kontrast/priorita jsou kryté regresními testy pro mode key i dirty-vs-mode badge.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-10T19:34:54Z
- **Completed:** 2026-03-10T19:41:02Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Zavedeno mapování save status prezentace v `editor/ui.rs` s explicitní primární prioritou pro dirty stav.
- Zachovány existující semantic texty statusu (`statusbar-unsaved`, `statusbar-saving`, `statusbar-saved`).
- Dopsány regression testy `save_ux_contrast_regression*` pro mode key větve a dirty marker prioritu vůči mode badge.

## Task Commits

Each task was committed atomically:

1. **Task 1: Vynucení pravidla dirty-first ve status baru** - `cd2ccff` (test), `54eab27` (feat)
2. **Task 2: Light/Dark regression guard pro čitelnost indikací** - `0d691a8` (test), `a5d58fd` (test)

## Files Created/Modified

- `src/app/ui/editor/ui.rs` - Save status render přes prezentaci s prioritou dirty-first.
- `src/app/ui/workspace/tests/save_mode.rs` - Regression testy pro save UX kontrast/prioritu.

## Decisions Made

- Dirty stav dostal explicitní primární důraz bez změny textové semantiky statusů.
- Regression coverage byla rozšířena cíleně na save UX kontrakt bez redesignu UI systému.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Cargo běhy padaly kvůli `sccache` oprávněním**
- **Found during:** Task 1 verifikace
- **Issue:** `cargo test` selhal na `sccache: Operation not permitted`.
- **Fix:** Verifikační příkazy spuštěny s `RUSTC_WRAPPER=` pro obejití sccache wrapperu.
- **Files modified:** žádné
- **Verification:** `cargo test dirty_state_visual_priority -- --nocapture`, `cargo check`, `cargo test save_ux -- --nocapture`
- **Committed in:** N/A (environment fix)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Nutný environment workaround pro dokončení verifikace; bez scope creep.

## Issues Encountered

- `./check.sh` padá na repo-wide `cargo fmt --check` driftu mimo scope tohoto plánu (informativní kontrola).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Save UX priority/contrast kontrakt pro MODE-04 je stabilizovaný a testovatelný.
- Fáze je připravená na navazující regression hardening kroky (26-03/26-04).

## Self-Check

PASSED

- FOUND: `.planning/phases/26-save-ux-polish-regression-hardening/26-02-SUMMARY.md`
- FOUND commits: `cd2ccff`, `54eab27`, `0d691a8`, `a5d58fd`

---
*Phase: 26-save-ux-polish-regression-hardening*
*Completed: 2026-03-10*
