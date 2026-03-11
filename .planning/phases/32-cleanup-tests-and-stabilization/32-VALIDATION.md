---
phase: 32
slug: cleanup-tests-and-stabilization
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 32 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) |
| **Config file** | none |
| **Quick run command** | `cargo test phase30_plan -- --nocapture` |
| **Full suite command** | `./check.sh` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test phase30_plan -- --nocapture`
- **After every plan wave:** Run `./check.sh`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 32-01-01 | 01 | 1 | STAB-01 | compile/gate | `cargo check && ./check.sh` | ✅ | ⬜ pending |
| 32-01-02 | 01 | 1 | STAB-02 | regression | `cargo test phase30_plan -- --nocapture` | ✅ | ⬜ pending |
| 32-02-01 | 02 | 2 | STAB-02 | targeted runtime | `cargo test approval -- --nocapture && cargo test slash::tests -- --nocapture` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/phase32_namespace_guard.rs` — explicit STAB namespace guard coverage
- [ ] `tests/phase32_runtime_stability.rs` — targeted prompt/stream/slash/approval smoke assertions

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Assistant-only terminal UX sanity (open panel + send prompt + visual stream progression) | STAB-02 | UI integration path is partially visual and timing-sensitive | Run app, open AI terminal, send prompt, confirm stream updates and no UI freeze/toast flood |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
