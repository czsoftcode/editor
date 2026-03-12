---
phase: 36
slug: safe-move-to-trash-engine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-12
---

# Phase 36 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) |
| **Config file** | none — existing Cargo workspace |
| **Quick run command** | `RUSTC_WRAPPER= cargo test phase36 -- --nocapture` |
| **Full suite command** | `RUSTC_WRAPPER= cargo check && RUSTC_WRAPPER= ./check.sh` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `RUSTC_WRAPPER= cargo test phase36 -- --nocapture`
- **After every plan wave:** Run `RUSTC_WRAPPER= cargo check && RUSTC_WRAPPER= ./check.sh`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds (task-level), full gate at wave-end

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 36-01-01 | 01 | 1 | TRASH-01 | unit/regression | `RUSTC_WRAPPER= cargo test phase36_move_file_to_trash -- --nocapture` | ❌ W0 | ⬜ pending |
| 36-01-02 | 01 | 1 | TRASH-02 | unit/regression | `RUSTC_WRAPPER= cargo test phase36_move_dir_to_trash -- --nocapture` | ❌ W0 | ⬜ pending |
| 36-02-01 | 02 | 2 | TRASH-04 | failure-path | `RUSTC_WRAPPER= cargo test phase36_fail_closed -- --nocapture` | ❌ W0 | ⬜ pending |
| 36-02-02 | 02 | 2 | RELIAB-02 | integration/regression | `RUSTC_WRAPPER= cargo test phase36_error_toast -- --nocapture` | ❌ W0 | ⬜ pending |
| 36-03-01 | 03 | 3 | TRASH-01,TRASH-02,TRASH-04,RELIAB-02 | quality-gate | `RUSTC_WRAPPER= cargo check && RUSTC_WRAPPER= ./check.sh` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/phase36_move_to_trash.rs` — coverage for TRASH-01/TRASH-02 contracts
- [ ] `tests/phase36_fail_safe.rs` — failure-path coverage for TRASH-04/RELIAB-02
- [ ] `tests/phase36_toast_propagation.rs` — explicit toast surfacing checks

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Toast wording je citelny a obsahuje doporuceni | RELIAB-02 | jazykova kvalita UX | Vyvolat delete failure a overit text toastu v UI |
| UI nezamrza pri delete adresare | RELIAB-02 | real-time interakce | Behem delete velkeho adresare zkusit interakce v editoru |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s (task checks)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
