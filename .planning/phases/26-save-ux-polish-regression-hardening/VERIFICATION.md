---
phase: 26-save-ux-polish-regression-hardening
status: passed
verified_at: 2026-03-10
verifier: codex
requirement_ids_from_plans:
  - MODE-04
requirements_cross_reference:
  MODE-04: accounted_in_.planning/REQUIREMENTS.md
---

# Phase 26 Verification

## Goal
Uživatel jasně vidí aktivní save režim a save flow je pokrytý regresními testy.

## Requirement ID Cross-reference
- `26-01-PLAN.md`, `26-02-PLAN.md`, `26-03-PLAN.md`, `26-04-PLAN.md` všechny obsahují pouze `MODE-04`.
- `.planning/REQUIREMENTS.md` obsahuje `MODE-04` (`UI clearly indicates active save mode...`) a je mapován na Phase 26.
- Výsledek: všechny requirement IDs z PLAN frontmatter jsou accounted for.

## Must-Haves vs Codebase

### 26-01 (status bar + tab indikace + runtime kontrakt)
- PASS: Aktivní režim je mapován explicitně (`statusbar-save-mode-automatic` / `statusbar-save-mode-manual`) přes runtime helpery v `src/app/ui/workspace/mod.rs`.
- PASS: Status bar renderuje save mode label z runtime režimu, draft se mimo apply nepromítá (`status_bar_save_mode_key_for_runtime`).
- PASS: Tab indikace je doplňková (`·A`/`·M`) a dirty zůstává primární (`●` před markerem) v `src/app/ui/editor/render/tabs.rs`.
- PASS: Regression testy pro runtime visibility jsou v `src/app/ui/workspace/tests/save_mode.rs`.

### 26-02 (dirty-first priorita a čitelnost)
- PASS: `SaveStatus::Modified` je explicitně primární (`is_primary: true`) v `src/app/ui/editor/ui.rs`.
- PASS: Light/dark větve barev statusu existují bez redesignu.
- PASS: Regression testy priorit a badge chování jsou v `src/app/ui/workspace/tests/save_mode.rs` (`save_ux_contrast_regression*`).

### 26-03 (Ctrl+S větve, guard save-fail, dedupe 1.5s)
- PASS: Deterministické Ctrl+S větve přes `manual_save_request_for_shortcut` + testy `manual_save_request_*`.
- PASS: Save fail v guard flow drží inline chybu, toast a nezavírá flow (`process_guard_save_failure_feedback`, `should_close_tabs_after_guard_decision`) + test `unsaved_close_guard_save_failure_feedback`.
- PASS: Dedupe kontrakt drží 1.5s (`SAVE_ERROR_DEDUPE_WINDOW = 1500ms`) + testy v `src/app/types.rs`.

### 26-04 (i18n coverage + idle safety)
- PASS: Save UX keyset je centralizovaný (`PHASE_26_SAVE_UX_KEYS`) v `src/i18n.rs`.
- PASS: i18n smoke přes všech 5 jazyků je pokryt (`save_ux_i18n_smoke`) a EN parity test (`all_lang_keys_match_english`) prochází.
- PASS: Idle safety guard test (`save_ux_idle_safety_guard`) potvrzuje absence periodických repaint/timer triggerů v save/guard cestách.

## Verification Commands
- PASS: `RUSTC_WRAPPER= cargo check`
- PASS: `RUSTC_WRAPPER= cargo test save_mode -- --nocapture`
- PASS: `RUSTC_WRAPPER= cargo test save_ux -- --nocapture`
- PASS: `RUSTC_WRAPPER= cargo test manual_save_request -- --nocapture`
- PASS: `RUSTC_WRAPPER= cargo test unsaved_close_guard -- --nocapture`
- PASS: `RUSTC_WRAPPER= cargo test save_error_dedupe -- --nocapture`
- PASS: `RUSTC_WRAPPER= cargo test all_lang_keys_match_english -- --nocapture`
- INFO: `./check.sh` FAIL na pre-existing repo-wide `cargo fmt --check` drift mimo scope fáze 26 (informational check v plánech).

## Verdict
Phase 26 goal je dosažen: aktivní save režim je v UI jasně viditelný a save flow má regresní pokrytí odpovídající `MODE-04`.
