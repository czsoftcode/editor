---
phase: 37
slug: trash-preview-restore-mvp
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-12
---

# Phase 37 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | other (Rust cargo test) |
| **Config file** | none |
| **Quick run command** | `RUSTC_WRAPPER= cargo test phase37_ -- --nocapture` |
| **Full suite command** | `RUSTC_WRAPPER= ./check.sh` |
| **Estimated runtime** | ~240 seconds |

---

## Sampling Rate

- **After every task commit:** Run `RUSTC_WRAPPER= cargo test phase37_ -- --nocapture`
- **After every plan wave:** Run `RUSTC_WRAPPER= ./check.sh`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 300 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 37-01-01 | 01 | 1 | RESTORE-01 | unit | `cargo test phase37_restore_happy_path -- --nocapture` | ❌ W0 | ⬜ pending |
| 37-01-02 | 01 | 1 | RESTORE-02 | unit | `cargo test phase37_restore_conflict_no_overwrite -- --nocapture` | ❌ W0 | ⬜ pending |
| 37-02-01 | 02 | 2 | TRASHUI-01 | integration | `cargo test phase37_trash_preview_modal -- --nocapture` | ❌ W0 | ⬜ pending |
| 37-03-01 | 03 | 3 | RESTORE-03 | integration | `cargo test phase37_restore_ui_sync -- --nocapture` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/phase37_restore_preview.rs` — focused tests for TRASHUI-01/RESTORE-01/02/03
- [ ] `tests/common_fs.rs` (nebo lokální helper modul) — shared temp-dir fixtures pro restore scénáře
- [ ] Existing infrastructure covers framework install (no additional install step)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Highlight obnovené položky ve file tree po restore | RESTORE-03 | Vizualní UX stav | Spusť restore v preview, ověř zvýraznění/expand na obnovené cestě a že soubor není auto-open v tabu. |
| Lokalizační čitelnost conflict/success toastu | RESTORE-02, RESTORE-03 | Jazyková kvalita | Ověř toast texty v aktivním locale při konfliktu (`restore as copy`) a při úspěšném restore. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 300s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
