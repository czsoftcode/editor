---
phase: 38
slug: watcher-stability-verification
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-12
---

# Phase 38 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) |
| **Config file** | none — standard Cargo workflow |
| **Quick run command** | `cargo test phase38 -- --nocapture` |
| **Full suite command** | `./check.sh` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test phase38 -- --nocapture`
- **After every plan wave:** Run `./check.sh`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 38-01-01 | 01 | 1 | RELIAB-03 | unit/integration | `cargo test phase38 -- --nocapture` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/phase38_watcher_stability.rs` — skeleton test matrix pro RELIAB-03
- [ ] `tests/phase38_watcher_stability.rs` — overflow fallback contract test
- [ ] Existing infrastructure covers framework setup (žádná instalace navíc)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Delete -> restore bez viditelného UI lagu při burstu eventů | RELIAB-03 | "No visible lag" je UX metrika závislá na interakci | Proveď sekvenci delete -> immediate restore přes Trash Preview, sleduj file tree refresh bez freeze/blink loop |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
