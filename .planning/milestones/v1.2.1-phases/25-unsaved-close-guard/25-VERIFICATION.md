---
phase: 25-unsaved-close-guard
phase_number: 25
phase_name: unsaved-close-guard
status: passed
verified_at: 2026-03-10
verifier: codex
requirements: [GUARD-01, GUARD-02, GUARD-03, GUARD-04]
artifacts_reviewed:
  - .planning/phases/25-unsaved-close-guard/25-08-PLAN.md
  - .planning/phases/25-unsaved-close-guard/25-09-PLAN.md
  - .planning/phases/25-unsaved-close-guard/25-10-PLAN.md
  - .planning/phases/25-unsaved-close-guard/25-08-SUMMARY.md
  - .planning/phases/25-unsaved-close-guard/25-09-SUMMARY.md
  - .planning/phases/25-unsaved-close-guard/25-10-SUMMARY.md
  - .planning/phases/25-unsaved-close-guard/25-UAT.md
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - .planning/STATE.md
verification_commands:
  - "RUSTC_WRAPPER= cargo check"
  - "RUSTC_WRAPPER= cargo test unsaved_close_guard -- --nocapture"
  - "RUSTC_WRAPPER= ./check.sh"
---

# Phase 25 Verification

## Verdict

Fáze 25 dosahuje cíle: UAT gapy pro `Ctrl+W` mutaci, `Esc` cancel/focus a `SingleTab` cílení fronty jsou v kódu implementované a kryté testy.

`cargo check` prošel.
`cargo test unsaved_close_guard -- --nocapture` prošel (`12 passed, 0 failed`).
`./check.sh` neprošel na `cargo fmt --check` kvůli preexistujícím formátovacím odchylkám mimo scope této fáze (viz už dříve evidované deferred položky); bez nového funkčního gapu pro phase 25.

## Must-have ověření (25-08, 25-09, 25-10)

### Plan 25-08 (Ctrl+W consume + input lock)

- Pravda: `Ctrl+W` je spotřebován na workspace vrstvě a nepropadá do editoru.
  - Evidence: `consume_shortcut(Ctrl+W)` v [src/app/ui/workspace/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/mod.rs:49).
  - Test: `unsaved_close_guard_ctrl_w_consumes_shortcut` v [src/app/ui/workspace/tests/unsaved_close_guard.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/tests/unsaved_close_guard.rs:101).
- Pravda: během aktivního guard flow je editor input locknutý.
  - Evidence: `editor_input_locked(dialog_open_base, pending_close_flow)` v [src/app/ui/workspace/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/mod.rs:58) a použití locku v [src/app/ui/workspace/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/mod.rs:457).
  - Evidence renderu: `TextEdit::interactive(!dialog_open)` v [src/app/ui/editor/render/normal.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/editor/render/normal.rs:134).
  - Test: `unsaved_close_guard_input_lock` v [src/app/ui/workspace/tests/unsaved_close_guard.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/tests/unsaved_close_guard.rs:124).
- Pravda: flow není re-entrantní.
  - Evidence: guard `if ws.pending_close_flow.is_some() { return; }` v [src/app/ui/workspace/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/mod.rs:115).

### Plan 25-09 (Esc->Cancel + focus handoff)

- Pravda: `Esc` v unsaved guard dialogu je explicitně mapováno na `Cancel`.
  - Evidence: `consume_key(Escape)` a `resolve_unsaved_guard_decision(...)->Cancel` v [src/app/ui/dialogs/confirm.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/dialogs/confirm.rs:58).
  - Test: `unsaved_close_guard_esc_cancel` v [src/app/ui/dialogs/confirm.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/dialogs/confirm.rs:188).
- Pravda: během guard flow se fokus nepředává předčasně editoru.
  - Evidence: queue handoff přes `open_file_without_focus` v [src/app/ui/workspace/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/mod.rs:255) a implementace v [src/app/ui/editor/tabs.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/editor/tabs.rs:153).
  - Evidence: globální focus reset je blokován při `guard_active` v [src/app/ui/workspace/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/mod.rs:662).
  - Test: `unsaved_close_guard_focus_handoff` v [src/app/ui/workspace/tests/unsaved_close_guard.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/tests/unsaved_close_guard.rs:172).
- Pravda: `Cancel` neuzavírá tab/projekt bez rozhodnutí.
  - Evidence: při `Cancel` outcome `Cancelled`, bez `close_tabs_for_path` větve v [src/app/ui/workspace/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/mod.rs:297).

### Plan 25-10 (SingleTab cílení fronty)

- Pravda: `SingleTab` flow řeší pouze cílový tab.
  - Evidence: `DirtyCloseQueueMode::SingleTab(target)` vrací max. jeden dirty target v [src/app/ui/workspace/state/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/state/mod.rs:230).
  - Test: `unsaved_close_guard_queue_single_tab_target` v [src/app/ui/workspace/state/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/state/mod.rs:282) a `unsaved_close_guard_single_tab_regressions` v [src/app/ui/workspace/tests/unsaved_close_guard.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/tests/unsaved_close_guard.rs:143).
- Pravda: `WorkspaceClose` iteruje všechny dirty taby deterministicky.
  - Evidence: `DirtyCloseQueueMode::WorkspaceClose` sort + active-first v [src/app/ui/workspace/state/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/state/mod.rs:240).
- Pravda: `TabBarAction::Close(idx)` řeší explicitní target tab bez vazby na pozdější `active_tab`.
  - Evidence: snapshot + `tabbar_close_target_path` + `request_close_tab_target` v [src/app/ui/workspace/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/mod.rs:598).
  - Test: `unsaved_close_guard_target_tab_from_tabbar_close` v [src/app/ui/workspace/tests/unsaved_close_guard.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/tests/unsaved_close_guard.rs:132).

## Traceability na requirement IDs

- `GUARD-01` (close dirty tab -> guard): splněno.
  - Kód: [src/app/ui/workspace/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/mod.rs:113), [src/app/ui/dialogs/confirm.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/dialogs/confirm.rs:78).
  - Testy: `unsaved_close_guard_ctrl_w_consumes_shortcut`, `unsaved_close_guard_target_tab_from_tabbar_close`, `unsaved_close_guard_single_tab_regressions`.
- `GUARD-02` (close app/project -> guard): splněno.
  - Kód: `start_global_close_guard` + `PendingCloseMode::WorkspaceClose` v [src/app/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/mod.rs:545).
  - Test: `unsaved_close_guard_root_flow` v [src/app/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/mod.rs:909).
- `GUARD-03` (Save/Discard/Cancel): splněno.
  - Kód: reducer `apply_unsaved_close_decision` a modal decision flow v [src/app/ui/workspace/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/mod.rs:171).
  - Testy: `unsaved_close_guard_modal_actions`, `unsaved_close_guard_tab_triggers`, `unsaved_close_guard_esc_cancel`.
- `GUARD-04` (save fail během close): splněno.
  - Kód: `save_result` fail větev, inline error + toast + neuzavření tabu v [src/app/ui/workspace/mod.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/mod.rs:275).
  - Test: `unsaved_close_guard_save_fail` v [src/app/ui/workspace/tests/unsaved_close_guard.rs](/home/stkremen/MyProject/Rust/polycredo_editor/src/app/ui/workspace/tests/unsaved_close_guard.rs:44).

## Gaps

- Žádný funkční gap v rozsahu phase 25 nebyl nalezen.
- Otevřená technická položka mimo scope: globální `cargo fmt --check` drift v jiných modulech (ovlivňuje `./check.sh`, neovlivňuje verifikovaný cíl fáze).

## Status Marker

`[STATUS: PASSED]`
