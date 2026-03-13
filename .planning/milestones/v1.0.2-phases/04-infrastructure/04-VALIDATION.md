---
phase: 4
slug: infrastructure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-05
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | SETT-01 | integration | `cargo test` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | SETT-02 | integration | `cargo test` | ❌ W0 | ⬜ pending |
| 04-01-03 | 01 | 1 | SETT-03 | integration | `cargo test` | ❌ W0 | ⬜ pending |
| 04-01-04 | 01 | 1 | SETT-04 | integration | `cargo test` | ❌ W0 | ⬜ pending |
| 04-01-05 | 01 | 1 | SETT-05 | integration | `cargo test` | ❌ W0 | ⬜ pending |
| 04-01-06 | 01 | 1 | TERM-01 | integration | `cargo test` | ❌ W0 | ⬜ pending |
| 04-01-07 | 01 | 1 | TERM-02 | integration | `cargo test` | ❌ W0 | ⬜ pending |
| 04-01-08 | 01 | 1 | TERM-03 | integration | `cargo test` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Tests not present yet — create minimal harness or mark manual-only where automation isn't feasible

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Settings toggle changes persisted in settings.toml | SETT-02 | UI persistence not covered by tests | Open Settings, toggle, Save, reopen project, verify settings.toml |
| Sandbox OFF uses root for terminals and label changes | TERM-01, TERM-03 | PTY working dir/label is UI-driven | Switch OFF, reopen project, open terminal, verify label and cwd |
| Sandbox UI elements hidden when OFF | UI-01..UI-03 | Visual UI state | Switch OFF, reopen project, verify hidden bars/buttons |
| Sync disabled when OFF | UI-04 | Behavioral + UI | Switch OFF, verify sync actions disabled |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
