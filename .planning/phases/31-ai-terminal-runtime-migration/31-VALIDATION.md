---
phase: 31
slug: ai-terminal-runtime-migration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 31 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) + shell smoke checks |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `rg -n "NeedsApproval|validate_path|dispatch|send_query_to_agent" src/app/ai_core src/app/ui/terminal/ai_chat src/app/ui/background.rs` |
| **Full suite command** | `cargo check && ./check.sh` |
| **Estimated runtime** | ~25 seconds (quick), full suite dle prostředí |

---

## Sampling Rate

- **After every task commit:** Run `rg -n "NeedsApproval|validate_path|dispatch|send_query_to_agent" src/app/ai_core src/app/ui/terminal/ai_chat src/app/ui/background.rs`
- **After every plan wave:** Run `cargo check && ./check.sh`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 25 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 31-03-01 | 03 | 3 | SAFE-01 | unit | `RUSTC_WRAPPER= cargo test approval -- --nocapture` | ✅ | ✅ green |
| 31-03-02 | 03 | 3 | SAFE-02 | unit | `RUSTC_WRAPPER= cargo test security -- --nocapture` | ✅ | ✅ green |
| 31-03-03 | 03 | 3 | SAFE-03 | build + targeted unit | `RUSTC_WRAPPER= cargo test audit_logger_sanitizes_multiline_details -- --nocapture && RUSTC_WRAPPER= cargo check` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Approval modal rendering (approve/deny/always UI parity) | SAFE-01 | egui widget layout/interakce není plně unit-testovatelná | Spustit editor, vyvolat `write_file` tool call, ověřit modal a následné resume konverzace pro approve i deny větev |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
