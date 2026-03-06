---
phase: 15
slug: streaming-chat-ui
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-06
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in #[cfg(test)] + cargo test |
| **Config file** | Cargo.toml (existing) |
| **Quick run command** | `cargo test --lib -- ai` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib -- ai`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 15-00-01 | 00 | 0 | CHAT-02, PROV-04 | unit | `cargo test --lib -- ai::state::tests settings::tests` | created in W0 | pending |
| 15-01-01 | 01 | 1 | CHAT-02, CHAT-05 | compile+unit | `cargo check && cargo test --lib -- ai` | W0 | pending |
| 15-01-02 | 01 | 1 | CHAT-07 | compile | `cargo check` | N/A | pending |
| 15-02-01 | 02 | 2 | CHAT-01, CHAT-03 | compile | `cargo check` | N/A | pending |
| 15-02-02 | 02 | 2 | CHAT-04, CHAT-07 | compile+grep | `cargo check && grep -c "Color32::from_rgb" src/app/ui/terminal/ai_chat/render.rs` | N/A | pending |
| 15-02-03 | 02 | 2 | CHAT-07, PROV-04 | compile+grep | `cargo check && grep -c "Color32::from_rgb" src/app/ui/terminal/ai_chat/render.rs` | N/A | pending |
| 15-02-04 | 02 | 2 | CHAT-01-07 | manual | checkpoint:human-verify | N/A | pending |
| 15-03-01 | 03 | 2 | PROV-04 | compile | `cargo check` | N/A | pending |
| 15-03-02 | 03 | 2 | PROV-04 | compile+unit | `cargo check && cargo test --lib -- settings::tests` | W0 | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

- [x] `src/app/ai/state.rs` tests — streaming_buffer defaults, auto_scroll reset logic (CHAT-02) — Plan 00 Task 1
- [x] `src/settings.rs` tests — AI settings migration from plugin settings (PROV-04) — Plan 00 Task 1
- [x] `src/settings.rs` tests — serde roundtrip for new AI fields (PROV-04) — Plan 00 Task 1

*Wave 0 plan (15-00-PLAN.md) creates all required test scaffolds.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CLI layout (prompt bottom, responses top) | CHAT-01 | UI visual layout — no programmatic assertion possible | Open chat, verify prompt is at bottom, responses scroll above |
| Theme-aware colors | CHAT-03 | Visual color correctness requires human eye | Switch between dark/light mode, verify colors adapt |
| Markdown rendering (code blocks, bold/italic) | CHAT-04 | Rendered output is visual egui content | Send prompt with code/bold/italic response, verify formatting |
| Stop/Send + auto-scroll + model picker | CHAT-07, PROV-04 | Interactive UI behavior | Plan 02 Task 4 checkpoint |

---

## CHAT-06 Note

CHAT-06 (Input with prompt history, up/down arrows) is already fully implemented in `src/app/ui/widgets/ai/chat/input.rs` (history/history_index logic). Pre-existing functionality — no new work needed in Phase 15. Confirmed by RESEARCH.md: "Already implemented in input.rs with history/history_index".

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
