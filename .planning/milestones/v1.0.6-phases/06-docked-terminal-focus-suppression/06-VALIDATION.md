---
phase: 6
slug: docked-terminal-focus-suppression
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-05
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo test` |
| **Config file** | `Cargo.toml` (test profile) |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | FSUP-01, FSUP-02 | manual | N/A - GUI interaction | N/A | pending |
| 06-01-02 | 01 | 1 | FSUP-01, FSUP-02 | manual | N/A - GUI interaction | N/A | pending |
| 06-01-03 | 01 | 1 | AICF-01, AICF-02 | manual | N/A - GUI interaction | N/A | pending |
| 06-01-04 | 01 | 1 | ALL | compile | `cargo test --lib` | yes | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No new test framework or stubs needed.

All requirements involve egui widget focus behavior requiring a running GUI event loop. egui does not provide a headless test harness for focus/input simulation.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Docked terminal no focus steal with modal open | FSUP-01 | Requires GUI interaction - egui focus is runtime-only | Open Settings modal, move mouse over terminal, verify keys go to modal not terminal |
| Docked terminal no focus steal with AI Chat open | FSUP-02 | Requires GUI interaction | Open AI Chat, hover over terminal area, verify typing stays in AI Chat |
| AI Chat TextEdit holds focus continuously | AICF-01 | Requires GUI interaction | Open AI Chat, type continuously, verify no focus interruptions |
| User can type in AI Chat without terminal capture | AICF-02 | Requires GUI interaction | Open AI Chat, type message, verify text appears in chat input not terminal |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
