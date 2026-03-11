---
phase: 31
slug: ai-terminal-runtime-migration
plan: 04
status: in-progress
updated: 2026-03-11
---

# Phase 31 - Final Verification Matrix

## Requirement Traceability Matrix

| Requirement | Status | Evidence Type | Evidence | Result | Date |
|-------------|--------|---------------|----------|--------|------|
| TERM-01 | PASS | Automated + code trace | `cargo check` PASS; `cargo test ai_chat -- --nocapture` PASS (includes `normalize_prompt_input_*`, provider config guards); send path in `logic.rs` (`send_query_to_agent`, `stream_chat`) | PASS | 2026-03-11 |
| TERM-02 | PASS | Automated + code trace | `cargo check` PASS; non-blocking event loop confirmed by `process_background_events` + `try_recv` usage in `background.rs` | PASS | 2026-03-11 |
| TERM-03 | PASS | Automated + code trace | `cargo test gsd -- --nocapture` PASS; model wiring and slash dispatch paths in `ai_bar.rs` (`selected_model`) + `slash.rs` (`dispatch`) | PASS | 2026-03-11 |
| SAFE-01 | PASS | Automated + code trace | `cargo test approval -- --nocapture` PASS (11/11), coverage `NeedsApproval`, approve/deny/resume branches v `ai_core/executor.rs` | PASS | 2026-03-11 |
| SAFE-02 | PASS | Automated + code trace | `cargo test security -- --nocapture` PASS (28/28), coverage `PathSandbox`, `FileBlacklist`, `CommandBlacklist`, `RateLimiter` v `ai_core/security.rs` + enforcement v `ai_core/executor.rs` | PASS | 2026-03-11 |
| SAFE-03 | PASS | Automated + code trace | `cargo test security -- --nocapture` includes `audit_logger_security_event` + secrets filter tests; error/audit handling remains in `ai_core/audit.rs` and background tool flow | PASS | 2026-03-11 |

## Repro Commands

```bash
cargo check
cargo test ai_chat -- --nocapture
cargo test gsd -- --nocapture
cargo test approval -- --nocapture
cargo test security -- --nocapture
```

## Notes

- Scope boundary validated against phase context: AI terminal-only runtime, external slash/GSD workflow, bez PolyCredo CLI fallback.
- SAFE sekce je finalizovaná na základě approval/security test runs.
