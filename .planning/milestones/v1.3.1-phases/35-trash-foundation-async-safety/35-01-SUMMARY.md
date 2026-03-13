---
phase: 35-trash-foundation-async-safety
plan: 01
subsystem: file-tree
tags: [trash, async, foundation]
provides:
  - On-demand `.polycredo/trash` foundation helpery
  - Async delete tok s fail-closed chovanim
  - Metadata kontrakt pro navaznou restore fazi
key-files:
  created:
    - .planning/phases/35-trash-foundation-async-safety/35-01-SUMMARY.md
  modified:
    - src/app/project_config.rs
    - src/app/mod.rs
    - src/app/trash.rs
    - src/app/ui/file_tree/dialogs.rs
    - src/app/ui/file_tree/mod.rs
requirements-completed: [TRASH-03, RELIAB-01]
completed: 2026-03-11
---

# Phase 35 Plan 01 Summary

## Co bylo dodano
- `project_config` byl rozsireny o `trash_dir_path()` pro deterministickou cestu `.polycredo/trash`.
- Pridan modul `src/app/trash.rs`:
  - `ensure_trash_dir()` (on-demand create v fail-closed rezimu),
  - `move_path_to_trash()` (presun do trash bez hard-delete fallbacku),
  - metadata kontrakt `TrashEntryMeta` (`trash_name`, `original_relative_path`, `deleted_at`, `entry_kind`).
- `file_tree` delete dialog byl preveden na async tok pres `spawn_task`; vysledky se sbiraji v `FileTree` a chyby jdou do `pending_error` (toast pipeline).

## Ověření
- `cargo check` PASS
- `./check.sh` PASS

## Commit
- `3970d29` — `feat(35-01): add trash foundation and async delete flow`

## Self-Check: PASSED
