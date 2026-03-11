---
phase: 34
slug: milestone-gap-closure-and-traceability-rebaseline
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 34 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) + shell grep gates |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `bash tests/phase33_removal_checks.sh all` |
| **Full suite command** | `RUSTC_WRAPPER= cargo check && RUSTC_WRAPPER= ./check.sh` |
| **Estimated runtime** | ~180 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bash tests/phase33_removal_checks.sh all`
- **After every plan wave:** Run `RUSTC_WRAPPER= cargo check && RUSTC_WRAPPER= ./check.sh`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 34-01-01 | 01 | 1 | R33-D | audit consistency | `! rg -n "status:\\s*gaps_found" .planning/phases/33-*/33-VERIFICATION.md .planning/v1.3.0-v1.3.0-MILESTONE-AUDIT.md -S` | ✅ | ⬜ pending |
| 34-01-02 | 01 | 1 | R33-A,R33-B,R33-C | grep + structure | `bash tests/phase33_removal_checks.sh all` | ✅ | ⬜ pending |
| 34-02-01 | 02 | 2 | R33-A..R33-D | traceability | `rg -n "R33-A|R33-B|R33-C|R33-D" .planning/REQUIREMENTS.md .planning/ROADMAP.md .planning/phases/33-*/33-VERIFICATION.md -S` | ✅ | ⬜ pending |
| 34-02-02 | 02 | 2 | R33-D | full gate | `RUSTC_WRAPPER= cargo check && RUSTC_WRAPPER= ./check.sh` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Milestone closure readiness after traceability sync | R33-D | Requires human audit sign-off over planning evidence | Run `$gsd-audit-milestone v1.3.0` and verify final status is `passed`. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or existing infra
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
