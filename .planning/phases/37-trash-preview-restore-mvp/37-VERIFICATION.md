# Phase 37 Verification Report

## Requirement Traceability

| Requirement | Behavior Contract | Evidence Hooks | Status |
| --- | --- | --- | --- |
| TRASHUI-01 | Trash preview modal zobrazi obsah `.polycredo/trash` a umozni restore vybrane polozky. | `cargo test phase37_trash_preview_ui -- --nocapture`, `cargo test phase37_i18n_restore_preview_parity -- --nocapture` | PASS |
| RESTORE-01 | Obnova jedne polozky vraci data na puvodni cestu bez ztraty dat. | `cargo test phase37_restore_engine -- --nocapture` (happy-path restore scenare) | PASS |
| RESTORE-02 | Pri konfliktu cesty je pouzita nedestruktivni policy (restore-as-copy), bez ticheho prepisu. | `cargo test phase37_restore_engine -- --nocapture`, `cargo test phase37_restore_ui_sync -- --nocapture` | PASS |
| RESTORE-03 | Po restore se UI synchronizuje (reload/expand/tab sync) bez auto-open noveho tabu. | `cargo test phase37_restore_ui_sync -- --nocapture`, `cargo test phase37_trash_preview_ui -- --nocapture` | PASS |

## i18n Parity Evidence

Task 37-04 doplnil parity-safe klice pro restore flow ve vsech locale (`cs/en/de/ru/sk`):

- `file-tree-trash-preview-*`
- `file-tree-restore-conflict-*`
- `file-tree-restore-as-copy`
- `file-tree-restore-success`
- `file-tree-restore-error`

Parity gate:

- `RUSTC_WRAPPER= cargo test all_lang_keys_match_english -- --nocapture` -> PASS
- `RUSTC_WRAPPER= cargo test phase37_i18n_restore_preview_parity -- --nocapture` -> PASS

## Command-Level Quality Gate

| Command | Result | Notes |
| --- | --- | --- |
| `RUSTC_WRAPPER= cargo check` | PASS | Build bez chyb |
| `RUSTC_WRAPPER= ./check.sh` | PASS | `cargo fmt`, `cargo clippy`, `cargo test` zeleny |
| `RUSTC_WRAPPER= cargo test phase37_restore_engine -- --nocapture` | PASS | Engine restore kontrakt |
| `RUSTC_WRAPPER= cargo test phase37_restore_ui_sync -- --nocapture` | PASS | UI sync + no-auto-open kontrakt |
| `RUSTC_WRAPPER= cargo test phase37_trash_preview_ui -- --nocapture` | PASS | Preview + conflict modal flow |
| `RUSTC_WRAPPER= cargo test phase37_i18n_restore_preview_parity -- --nocapture` | PASS | Lokalizacni parity guard |

## Conclusion

Phase 37 ma auditovatelnou traceability mezi TRASHUI-01 a RESTORE-01/02/03, vcetne explicitnich quality-gate zaznamu pro `cargo check` a `./check.sh`.
