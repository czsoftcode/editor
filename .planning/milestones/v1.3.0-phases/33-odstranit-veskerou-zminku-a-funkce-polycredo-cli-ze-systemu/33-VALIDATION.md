---
phase: 33
slug: odstranit-veskerou-zminku-a-funkce-polycredo-cli-ze-systemu
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 33 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) |
| **Config file** | none |
| **Quick run command** | `cargo check` |
| **Full suite command** | `./check.sh` |
| **Estimated runtime** | ~180 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check`
- **After every plan wave:** Run `./check.sh`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 33-01-01 | 01 | 1 | R33-B | compile+grep | `cargo check && ! rg -n "runtime-ai-modul|ui/terminal/runtime-chat-modul" src/app` | ✅ | ⬜ pending |
| 33-01-02 | 01 | 1 | R33-A | smoke | `cargo test phase33_launcher_only -- --nocapture` | ❌ W0 | ⬜ pending |
| 33-02-01 | 02 | 2 | R33-D | audit | `! rg -n "legacy CLI vrstva|runtime-ai-modul|runtime-chat-modul|app::legacy-cli" .planning` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/phase33_launcher_only.rs` — verifies ai_bar launcher path remains and chat/runtime modules are gone

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Agent start from ai_bar sends command to active terminal tab | R33-A | UI-level interaction with terminal focus is timing/visual | Open app, select agent in ai_bar, click Start, verify command appears in active terminal tab |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
