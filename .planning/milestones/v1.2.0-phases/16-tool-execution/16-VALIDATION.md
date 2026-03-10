---
phase: 16
slug: tool-execution
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-06
updated: 2026-03-06
---

# Phase 16 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 16-01-01 | 01 | 1 | TOOL-02,03,04 | unit+tdd | `cargo test ai::security::tests -x` | pending |
| 16-01-02 | 01 | 1 | TOOL-02,03,04 | unit+tdd | `cargo test ai::audit::tests -x && cargo check` | pending |
| 16-02-01 | 02 | 1 | TOOL-01,02 | unit+tdd | `cargo test ai::types::tests -x && cargo check` | pending |
| 16-02-02 | 02 | 1 | TOOL-01,02 | unit+tdd | `cargo test ai::ollama::tests -x && cargo check` | pending |
| 16-03-01 | 03 | 2 | TOOL-02,03,04,06 | unit+tdd | `cargo test ai::executor::tests -x` | pending |
| 16-03-02 | 03 | 2 | TOOL-02,03,04,06 | unit+tdd | `cargo test ai::executor::tests -x` | pending |
| 16-04-01 | 04 | 3 | TOOL-05 | unit+tdd | `cargo test ai::executor::tests::test_approval -x` | pending |
| 16-04-02 | 04 | 3 | TOOL-01,05 | compile | `cargo check` | pending |
| 16-04-03 | 04 | 3 | TOOL-05,06 | compile | `cargo check` | pending |
| 16-04-04 | 04 | 3 | TOOL-05 | compile | `cargo check` | pending |
| 16-04-05 | 04 | 3 | TOOL-05,06 | manual+compile | `cargo test --lib` | pending |

*Status: pending / green / red / flaky*

---

## TDD Coverage Summary

Plans 01, 02, 03 use TDD (`tdd="true"`) for all auto tasks — tests are written before implementation. No separate Wave 0 plan is needed because each TDD task creates its own tests inline as the first step of implementation.

Plan 04 Task 1 adds TDD-tested approval response logic (`process_approval_response`, `build_approval_messages`) in executor.rs, providing automated unit test coverage for TOOL-05 approval workflow decision routing.

Plan 04 Tasks 2-4 are UI wiring/integration — verified by `cargo check` (compile correctness) and human checkpoint (Task 5).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Diff preview renders correctly in egui | TOOL-04 | GUI rendering | Open editor, trigger AI file edit, verify diff colors and layout |
| Approval dialog appears and blocks execution | TOOL-05 | GUI interaction | Trigger tool call, verify approve/deny/always buttons work |
| Tool call block renders in chat | TOOL-02 | GUI rendering | Send message triggering tool, verify compact block with icon |
| Terminal output captured in context | TOOL-01 | Runtime integration | Run command in terminal, send AI message, verify context includes output |
| Always-approved tools skip dialog on repeat | TOOL-05 | GUI state | Approve with "Always", trigger same tool again, verify no dialog |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify commands
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] TDD plans (01, 02, 03) create tests inline — no separate Wave 0 needed
- [x] Plan 04 Task 1 provides unit-tested approval logic for TOOL-05
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter
- [x] Task map matches actual 4-plan structure (11 task entries)

**Approval:** ready
