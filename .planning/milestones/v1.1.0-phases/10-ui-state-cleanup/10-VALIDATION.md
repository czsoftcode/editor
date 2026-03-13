---
phase: 10
slug: ui-state-cleanup
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-05
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + cargo test |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo check` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 1 | UI-02 | compile | `cargo check` | N/A (deletion) | pending |
| 10-01-02 | 01 | 1 | UI-01, UI-03 | compile | `cargo check` | N/A (deletion) | pending |
| 10-01-03 | 01 | 1 | UI-05 | compile | `cargo check` | N/A (deletion) | pending |
| 10-01-04 | 01 | 1 | UI-04 | compile | `cargo check` | N/A (deletion) | pending |
| 10-01-05 | 01 | 1 | STATE-01 | compile | `cargo check` | N/A (deletion) | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. This is a deletion phase — successful `cargo check` after all removals is the primary validation.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Toast akce removed | UI-06 | Already done in Phase 9, runtime-only | Verify no sandbox toast actions appear |
| State fields removed | STATE-01-04 | Already done in Phase 9 | Confirmed by code inspection |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
