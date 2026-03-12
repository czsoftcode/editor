---
phase: 37
phase_slug: trash-preview-restore-mvp
status: PASS
verified_at: 2026-03-12
goal: "trash preview + single-item restore MVP with conflict-safe flow and UI consistency"
requirement_ids:
  - TRASHUI-01
  - RESTORE-01
  - RESTORE-02
  - RESTORE-03
quality_gates:
  cargo_check: PASS
  check_sh: PASS
---

# Phase 37 Verification Report

## Goal Verdict

Phase goal je splnen. Implementace a test evidence pokryva:

- trash preview list + restore trigger,
- single-item restore na puvodni cestu,
- konflikt-safe restore flow bez overwrite,
- konzistentni post-restore UI chovani (reload/expand/tab sync/toast).

## Requirement ID Cross-Reference (PLAN frontmatter vs REQUIREMENTS.md)

Zdroj PLAN frontmatter:

- `.planning/phases/37-trash-preview-restore-mvp/37-01-PLAN.md`
- `.planning/phases/37-trash-preview-restore-mvp/37-02-PLAN.md`
- `.planning/phases/37-trash-preview-restore-mvp/37-03-PLAN.md`
- `.planning/phases/37-trash-preview-restore-mvp/37-04-PLAN.md`

Unie ID z PLAN frontmatter: `TRASHUI-01`, `RESTORE-01`, `RESTORE-02`, `RESTORE-03`

Zdroj REQUIREMENTS:

- `.planning/REQUIREMENTS.md` obsahuje vsechna 4 ID jako aktivni a mapovana na Phase 37.

Verdikt cross-reference: PASS (kazde ID z PLAN frontmatter je zohledneno v REQUIREMENTS.md, zadne chybejici ID).

## Requirement Traceability

| Requirement | Behavior Contract | Evidence v kodu | Test/command evidence | Status |
| --- | --- | --- | --- | --- |
| TRASHUI-01 | Uzivatel ma nahled do `.polycredo/trash` a z nahledu spusti restore. | `src/app/ui/workspace/menubar/mod.rs`, `src/app/ui/widgets/command_palette.rs`, `src/app/ui/file_tree/preview.rs` (`show_trash_preview_dialog`) | `cargo test phase37 -- --nocapture` (phase37_trash_preview_ui + phase37_restore_ui_sync) | PASS |
| RESTORE-01 | Obnova jedne polozky vraci data na puvodni cestu, vcetne vytvoreni parent adresaru. | `src/app/trash.rs` (`restore_from_trash`, `fs::create_dir_all(parent)`) | `cargo test phase37 -- --nocapture` (phase37_restore_to_original_path, phase37_restore_creates_parent_dirs) | PASS |
| RESTORE-02 | Konflikt cilove cesty je nedestruktivni (copy/cancel), bez ticheho prepisu. | `src/app/trash.rs` (`RestoreConflictPolicy::RestoreAsCopy`, `resolve_restore_copy_destination`), `src/app/ui/file_tree/dialogs.rs` (`show_restore_conflict_dialog`) | `cargo test phase37 -- --nocapture` (phase37_restore_conflict_as_copy, phase37_conflict_has_no_overwrite_action) | PASS |
| RESTORE-03 | Po restore je UI konzistentni bez restartu (reload/expand/tab sync). | `src/app/ui/file_tree/mod.rs` (`pending_restored`, `request_reload_and_expand`), `src/app/ui/panels.rs`, `src/app/ui/editor/tabs.rs` (`sync_tabs_for_restored_path`) | `cargo test phase37 -- --nocapture` (phase37_restore_triggers_reload_highlight, phase37_restore_no_auto_open_tab, phase37_preview_restore_roundtrip) | PASS |

## must_haves Audit

| Plan | must_have (shrnutí) | Evidence | Status |
| --- | --- | --- | --- |
| 37-01 | Preview list cte trash pres engine API + metadata error contract + fail-closed restore | `src/app/trash.rs` (`list_trash_entries`, `TrashMetadataStatus`, `restore selhal:` prefix, rollback pri cleanup fail), `tests/phase37_restore_engine.rs` | PASS |
| 37-02 | Preview dostupny z menu/command, restore async, konflikt bez overwrite | `src/app/ui/workspace/menubar/mod.rs`, `src/app/ui/widgets/command_palette.rs`, `src/app/ui/file_tree/preview.rs` (`spawn_task`), `src/app/ui/file_tree/dialogs.rs` | PASS |
| 37-03 | Conflict policy restore-as-copy, UI refresh + highlight, bez auto-open tabu | `src/app/trash.rs`, `src/app/ui/file_tree/mod.rs`, `src/app/ui/panels.rs`, `src/app/ui/editor/tabs.rs` | PASS |
| 37-04 | i18n parity + finalni traceability + quality gate zaznam | `tests/phase37_i18n_restore_parity.rs`, `locales/*/ui.ftl`, tento report | PASS |

## Command-Level Evidence

Aktualne spustene commandy pri teto verifikaci:

- `cargo check` -> PASS
- `cargo test phase37 -- --nocapture` -> PASS (17/17 phase37 testu)
- `./check.sh` -> PASS (`cargo fmt`, `cargo clippy`, `cargo test` vse zelene)

Poznamka k presnosti evidence: pro phase-specific dokaz je autoritativni `cargo test phase37 -- --nocapture` (spousti vsechny phase37 test funkce); tento command je pouzit jako hlavni proof.

## Final Status

Phase 37 je overena jako **PASS** proti goal statementu i proti requirements `TRASHUI-01`, `RESTORE-01`, `RESTORE-02`, `RESTORE-03`.
