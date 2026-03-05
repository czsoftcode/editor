---
phase: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu
plan: 01
subsystem: ui
tags: [sandbox, settings, egui, multiwindow, toasts]

# Dependency graph
requires:
  - phase: 04-infrastructure
    provides: sandbox settings persistence + UX baseline
provides:
  - okamzite apply sandbox rezimu po Save
  - persist -> runtime apply flow s revert/keep volbou
  - multi-window detekce konfliktu settings
affects: [phase-05, sandbox-runtime, settings-modal]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - pending_sandbox_apply fronta pres settings_version
    - action toasty pro apply now/defer a persist failure

key-files:
  created: []
  modified:
    - src/app/ui/workspace/modal_dialogs/settings.rs
    - src/app/ui/workspace/state/mod.rs
    - src/app/ui/workspace/state/init.rs
    - src/app/mod.rs
    - src/app/ui/workspace/mod.rs
    - src/app/ui/workspace/modal_dialogs/conflict.rs
    - src/app/ui/workspace/modal_dialogs.rs
    - src/app/ui/panels.rs
    - src/app/types.rs
    - src/settings.rs
    - locales/cs/ui.ftl
    - locales/en/ui.ftl

key-decisions:
  - "Sandbox apply se planuje pres pending_sandbox_apply a spousti se az po persistu settings."
  - "Persist failure resi toast s volbou revert/keep a runtime apply bezi jen po explicitnim keep."

patterns-established:
  - "Settings conflict: pri zmenach z jineho okna zobrazit modal s volbou reload/keep."

requirements-completed: [SANDBOX-01, SANDBOX-02]

# Metrics
duration: 29min
completed: 2026-03-05
---

# Phase 05 Plan 01: Save/Cancel + Runtime Apply Summary

**Okamzite apply sandbox rezimu po Save s potvrzenim OFF, multi-window propagaci a persist error handlingem.**

## Performance

- **Duration:** 29 min
- **Started:** 2026-03-05T05:43:09Z
- **Completed:** 2026-03-05T06:12:20Z
- **Tasks:** 3
- **Files modified:** 12

## Accomplishments
- Save/Cancel flow pro sandbox rezim s potvrzenim OFF a volbou apply now/defer.
- Runtime apply helper s multi-window propagaci pres settings_version.
- Persist failure handling s revert/keep temporary a navaznym runtime apply.

## Task Commits

Each task was committed atomically:

1. **Task 1: Save/Cancel flow pro sandbox rezim + potvrzeni OFF + odlozeni apply** - `f243774`, `3d3f4b6` (test/feat)
2. **Task 2: Runtime apply helper + multi-window dispatch** - `88b7a44`, `1d3cd72` (test/feat)
3. **Task 3: Persist -> apply poradi + error handling** - `e494bd3` (feat)

Additional: `1e04c8a` (chore) formatting after `cargo fmt`.

## Files Created/Modified
- `src/app/ui/workspace/modal_dialogs/settings.rs` - potvrzeni OFF, persist flow, toasty a akcni volby
- `src/app/ui/workspace/state/mod.rs` - pending struktury + runtime apply helper
- `src/app/mod.rs` - settings_version hook pro multi-window apply
- `src/app/ui/workspace/mod.rs` - zpracovani pending apply/persist rozhodnuti
- `src/app/ui/panels.rs` - akcni toasty
- `src/settings.rs` - try_save s chybami
- `locales/cs/ui.ftl`, `locales/en/ui.ftl` - nove texty/tooltipy

## Decisions Made
- Pouzit pending apply frontu pres settings_version pro multi-window synchronizaci.
- Persist failure resit toastem s explicitni volbou revert/keep.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `./check.sh` selhal na `cargo clippy` kvuli `sccache: Operation not permitted` (os error 1). Formatovani pres `cargo fmt --all` probehlo.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Runtime apply flow pripraven pro navazujici sandbox tasky.
- Doplnit CLI UAT (multi-window + persist failure) pri manualnim testu.

---
*Phase: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu*
*Completed: 2026-03-05*

## Self-Check: PASSED
