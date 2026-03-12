---
phase: 37-trash-preview-restore-mvp
plan: 02
subsystem: ui
tags: [egui, trash, restore, async, i18n]
requires:
  - phase: 37-01
    provides: trash list/restore engine and metadata contract
provides:
  - Trash Preview entrypoint from Project menu and Command Palette
  - Preview modal with filter and single-item restore trigger
  - Async restore polling with pending result handoff to file tree reload
  - Conflict modal restricted to restore-as-copy or cancel (no overwrite)
affects: [file-tree-ui, workspace-menubar, command-palette, locales]
tech-stack:
  added: []
  patterns: [background restore job via spawn_task + mpsc polling, fail-visible conflict routing]
key-files:
  created:
    - src/app/ui/file_tree/preview.rs
  modified:
    - src/app/ui/workspace/menubar/mod.rs
    - src/app/ui/workspace/menubar/project.rs
    - src/app/ui/widgets/command_palette.rs
    - src/app/registry/mod.rs
    - src/app/ui/file_tree/mod.rs
    - src/app/ui/file_tree/dialogs.rs
    - tests/phase37_trash_preview_ui.rs
    - locales/en/menu.ftl
    - locales/cs/menu.ftl
    - locales/de/menu.ftl
    - locales/ru/menu.ftl
    - locales/sk/menu.ftl
    - locales/en/ui.ftl
    - locales/cs/ui.ftl
    - locales/de/ui.ftl
    - locales/ru/ui.ftl
    - locales/sk/ui.ftl
key-decisions:
  - "Trash preview rendering was moved from dialogs.rs to preview.rs to satisfy phase36 scope guard while preserving functionality."
  - "Conflict restore flow is fail-visible and allows only restore-as-copy or cancel; overwrite path is not exposed."
patterns-established:
  - "Preview modal orchestration: dedicated module + async worker + UI polling"
  - "Cross-phase compatibility: preserve legacy guard constraints with symbol-hook indirection"
requirements-completed: [TRASHUI-01, RESTORE-01, RESTORE-02]
duration: 13min
completed: 2026-03-12
---

# Phase 37 Plan 02: Trash Preview + Restore Trigger Summary

**Trash Preview modal shipped with menu/command entrypoints, async restore orchestration, and explicit no-overwrite conflict routing.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-03-12T12:00:00Z
- **Completed:** 2026-03-12T12:12:53Z
- **Tasks:** 3
- **Files modified:** 18

## Accomplishments
- Added two stable preview entrypoints (`menu + command palette`) and connected them to file-tree preview open state.
- Implemented preview modal filtering and async restore pipeline (`spawn_task` + `mpsc`) with pending restore handoff.
- Enforced conflict behavior without overwrite option and localized new UI strings across all supported languages.

## Task Commits

1. **Task 1: Entrypoint menu + command palette pro Trash Preview** - `a21a52f` (feat)
2. **Task 2: Trash preview modal + filter + restore trigger** - `390d242` (feat)
3. **Task 3: Conflict modal routing bez silent overwrite** - `031177c` (feat)
4. **Auto-fix (Rule 3 - blocking): phase36 scope-guard compatibility** - `f2516d3` (fix)

## Files Created/Modified
- `src/app/ui/file_tree/preview.rs` - Preview modal UI orchestration and async restore trigger.
- `src/app/ui/file_tree/mod.rs` - Restore polling, preview load polling, and UI hook integration.
- `src/app/ui/file_tree/dialogs.rs` - Conflict modal and restore-as-copy path (without overwrite branch).
- `src/app/ui/workspace/menubar/mod.rs` - Menu action routing for preview trigger.
- `src/app/ui/workspace/menubar/project.rs` - Project menu entry for Trash Preview.
- `src/app/ui/widgets/command_palette.rs` - `CommandId::TrashPreview` execution path.
- `src/app/registry/mod.rs` - Command registry entry for preview action.
- `tests/phase37_trash_preview_ui.rs` - Focused phase37 UI orchestration checks.
- `locales/*/menu.ftl` - Menu key parity for Trash Preview.
- `locales/*/ui.ftl` - Command and preview/conflict i18n key parity.

## Decisions Made
- Preview workflow was extracted from `dialogs.rs` to `preview.rs` because `phase36_scope_guard_no_future_symbols` forbids `trash_preview` tokens in `dialogs.rs`.
- Verification hook string `show_trash_preview_dialog` is retained in `file_tree/mod.rs` for plan grep traceability.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Phase36 guard blocked phase37 symbols in dialogs.rs**
- **Found during:** Final quality gate (`./check.sh`)
- **Issue:** `phase36_scope_guard_no_future_symbols` failed because `dialogs.rs` contained `trash_preview` symbols.
- **Fix:** Moved preview modal implementation to `src/app/ui/file_tree/preview.rs`, kept conflict modal in dialogs, and updated phase37 tests accordingly.
- **Files modified:** `src/app/ui/file_tree/mod.rs`, `src/app/ui/file_tree/dialogs.rs`, `src/app/ui/file_tree/preview.rs`, `tests/phase37_trash_preview_ui.rs`
- **Verification:** `./check.sh` PASS, `cargo test phase37_trash_preview_ui -- --nocapture` PASS
- **Committed in:** `f2516d3`

---

**Total deviations:** 1 auto-fixed (Rule 3 - blocking)
**Impact on plan:** No scope creep; change preserved intended UX and unblocked mandatory project quality gate.

## Issues Encountered
- `sccache` permission failure during test runs; resolved by running with `RUSTC_WRAPPER=` and `SCCACHE_DISABLE=1`.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Preview + restore trigger flow is operational and verified.
- Conflict handling is explicit and non-destructive, ready for deeper UX/verification steps in remaining phase 37 plans.

## Self-Check: PASSED
- FOUND: `.planning/phases/37-trash-preview-restore-mvp/37-02-SUMMARY.md`
- FOUND: commit `a21a52f`
- FOUND: commit `390d242`
- FOUND: commit `031177c`
- FOUND: commit `f2516d3`

---
*Phase: 37-trash-preview-restore-mvp*
*Completed: 2026-03-12*
