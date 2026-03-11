---
phase: 25
slug: unsaved-close-guard
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-09
---

# Phase 25 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | none |
| **Quick run command** | `cargo test unsaved_close_guard -- --nocapture` |
| **Full suite command** | `./check.sh` |
| **Estimated runtime** | ~90 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test unsaved_close_guard -- --nocapture`
- **After every plan wave:** Run `./check.sh`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID  | Plan | Wave | Requirement | Test Type        | Automated Command                                  | File Exists | Status   |
|----------|------|------|-------------|------------------|----------------------------------------------------|-------------|----------|
| 25-01-01 | 01   | 1    | GUARD-01    | unit             | `cargo test unsaved_close_guard_queue`            | ✅          | ✅ green |
| 25-02-01 | 02   | 1    | GUARD-01    | integration-lite | `cargo test unsaved_close_guard_tab_triggers`     | ✅          | ✅ green |
| 25-03-01 | 03   | 2    | GUARD-03    | integration-lite | `cargo test unsaved_close_guard_modal_actions`    | ✅          | ✅ green |
| 25-03-02 | 03   | 2    | GUARD-04    | integration-lite | `cargo test unsaved_close_guard_save_fail`        | ✅          | ✅ green |
| 25-04-01 | 04   | 3    | GUARD-02    | integration-lite | `cargo test unsaved_close_guard_root_flow`        | ✅          | ✅ green |
| 25-05-01 | 05   | 3    | GUARD-01    | regression       | `cargo test unsaved_close_guard -- --nocapture`   | ✅          | ✅ green |
| 25-05-02 | 05   | 3    | GUARD-04    | regression       | `cargo test unsaved_close_guard_save_fail`        | ✅          | ✅ green |
| 25-06-01 | 06   | 4    | GUARD-03    | i18n-check       | `./check.sh`                                      | ✅          | ⚠️ see notes |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `src/app/ui/workspace/tests/unsaved_close_guard.rs` — unit/integration-lite scénáře pro GUARD-01..04
- [x] test helper / primitives pro guard reducer, queue builder a root-flow orchestrace

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Quit aplikace s dirty taby ve více oknech | GUARD-02 | více viewportů + modal orchestrace | Otevři dirty taby ve 2 oknech, spusť Quit, ověř sekvenční guard flow a že `Cancel` stopne celý quit |
| Save fail během close dialogu ukáže inline + toast a zůstane ve flow | GUARD-04 | je potřeba realistická IO chyba | Udělej soubor readonly/invalidní path, dej `Save`, ověř inline chybu, toast a že tab/aplikace zůstává otevřená |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** PASS (Phase 25 — Unsaved Close Guard)
