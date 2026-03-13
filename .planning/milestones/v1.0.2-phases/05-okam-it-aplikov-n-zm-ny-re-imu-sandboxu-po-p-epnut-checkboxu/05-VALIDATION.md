---
phase: 05
slug: okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-05
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) |
| **Config file** | Cargo.toml |
| **Quick run command** | `RUSTC_WRAPPER= cargo check` |
| **Full suite command** | `./check.sh` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `RUSTC_WRAPPER= cargo check`
- **After every plan wave:** Run `./check.sh`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | N/A | build | `RUSTC_WRAPPER= cargo check` | ✅ | ⬜ pending |
| 05-01-02 | 01 | 1 | N/A | build | `RUSTC_WRAPPER= cargo check` | ✅ | ⬜ pending |
| 05-01-03 | 01 | 1 | N/A | build | `RUSTC_WRAPPER= cargo check` | ✅ | ⬜ pending |
| 05-02-01 | 02 | 2 | N/A | build | `RUSTC_WRAPPER= cargo check` | ✅ | ⬜ pending |
| 05-02-02 | 02 | 2 | N/A | build | `RUSTC_WRAPPER= cargo check` | ✅ | ⬜ pending |
| 05-03-01 | 03 | 2 | N/A | build | `RUSTC_WRAPPER= cargo check` | ✅ | ⬜ pending |
| 05-03-02 | 03 | 2 | N/A | build | `RUSTC_WRAPPER= cargo check` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Multi-window změna režimu | N/A | Vyžaduje dvě okna | Otevři dvě okna stejného projektu, změň režim v jednom a ověř upozornění + apply v druhém |
| Potvrzení OFF + odložení apply | N/A | UX flow s dialogy/toasty | Změň režim na OFF, potvrď, otestuj Save/Cancel a „apply now/defer“ |
| Staged blokace OFF + sync ON | N/A | Interakce se sandbox stavem | Vytvoř staged soubory, zkus OFF, pak ON a potvrď sync |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** {pending / approved YYYY-MM-DD}
