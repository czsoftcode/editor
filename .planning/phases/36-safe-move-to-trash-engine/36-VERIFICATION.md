# Phase 36 Verification Report

Datum: 2026-03-12
Plan: 36-03
Scope: safe-move-to-trash-engine final quality gate evidence

## Requirement -> Test Map

| Requirement | Test hook | Command | Stav |
| --- | --- | --- | --- |
| TRASH-01 | `phase36_move_file_to_trash` | `RUSTC_WRAPPER= cargo test phase36_move_file_to_trash -- --nocapture` | PASS |
| TRASH-02 | `phase36_move_dir_to_trash` | `RUSTC_WRAPPER= cargo test phase36_move_dir_to_trash -- --nocapture` | PASS |
| TRASH-04 | `phase36_fail_closed` | `RUSTC_WRAPPER= cargo test phase36_fail_closed -- --nocapture` | PASS |
| RELIAB-02 | `phase36_error_toast` + `phase36_disconnected_channel_toast` | `RUSTC_WRAPPER= cargo test phase36_error_toast -- --nocapture` + `RUSTC_WRAPPER= cargo test phase36_disconnected_channel_toast -- --nocapture` | PASS |

## Task-Checkpoint Evidence

| Checkpoint | Command | Výsledek |
| --- | --- | --- |
| Hook coverage: file move | `RUSTC_WRAPPER= cargo test phase36_move_file_to_trash -- --nocapture` | PASS |
| Hook coverage: dir move | `RUSTC_WRAPPER= cargo test phase36_move_dir_to_trash -- --nocapture` | PASS |
| Hook coverage: fail-closed | `RUSTC_WRAPPER= cargo test phase36_fail_closed -- --nocapture` | PASS |
| Hook coverage: error toast | `RUSTC_WRAPPER= cargo test phase36_error_toast -- --nocapture` | PASS |

## Wave-End Quality Gate

| Command | Důkaz |
| --- | --- |
| `RUSTC_WRAPPER= cargo test phase36 -- --nocapture` | PASS (spuštěno v rámci gate) |
| `RUSTC_WRAPPER= cargo check` | PASS (spuštěno v rámci gate) |
| `RUSTC_WRAPPER= ./check.sh` | PASS (spuštěno v rámci gate) |
| `! rg -n "remove_file\\(|remove_dir_all\\(" src/app/trash.rs src/app/ui/file_tree/dialogs.rs -S` | PASS (žádný hard-delete fallback) |
| `rg -n "TRASH-01|TRASH-02|TRASH-04|RELIAB-02" .planning/phases/36-safe-move-to-trash-engine/36-VERIFICATION.md -S` | PASS |

## Manual UX Verification (RELIAB-02)

1. Otevři projekt se souborem a adresářem určeným ke smazání.
2. Spusť delete nad položkou, která vyvolá chybu přesunu do trash (např. nedostatečná práva).
3. Ověř, že toast je čitelný, obsahuje důvod i doporučení dalšího kroku.
4. Současně během delete interaguj s UI (klikání mezi soubory, scroll), UI nesmí zamrznout.
5. Ověř, že při chybě zůstávají data na původním místě (fail-closed).

Výsledek manuálního ověření: PASS (toast je čitelný, tok zůstává responsivní, fail-closed zachován).

