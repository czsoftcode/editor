---
phase: 18
slug: phase-16-verification-i18n-fixes
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-06
---

# Phase 18 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
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
| 18-01-01 | 01 | 1 | TOOL-01..06 | doc review | `test -f .planning/phases/16-tool-execution/16-VERIFICATION.md` | W0 | pending |
| 18-01-02 | 01 | 1 | TOOL-01..06 | doc review | `test -f .planning/phases/16-tool-execution/16-04-SUMMARY.md` | W0 | pending |
| 18-01-03 | 01 | 1 | CLEN-03 | grep | `grep -c '\[x\]' .planning/ROADMAP.md` | N/A | pending |
| 18-01-04 | 01 | 1 | CLEN-03 | grep | `grep -c '\[x\]' .planning/REQUIREMENTS.md` | N/A | pending |
| 18-02-01 | 02 | 1 | TOOL-06 | grep | `grep -r 'get_args.*cli-tool-ask-heading' src/` | Exists | pending |
| 18-02-02 | 02 | 1 | CLEN-03 | unit | `cargo test all_lang_keys` | Exists | pending |
| 18-02-03 | 02 | 1 | CLEN-03 | grep | `grep -r 'cli-tool-approval-heading' src/` returns nothing | N/A | pending |
| 18-02-04 | 02 | 1 | CLEN-03 | grep | `grep -r '"Generating\|"Family:\|"Parameters:\|"Quantization:\|"Context:\|"In:\|"Unexpected\|"Ollama is not' src/` returns nothing | N/A | pending |

*Status: pending · green · red · flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements. The `all_lang_keys_match_english` test validates i18n key parity across all 5 locales.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| cli-tool-ask-heading shows agent name | TOOL-06 | UI rendering | Open AI chat, trigger tool ask, verify heading shows agent name instead of literal `{$agent}` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
