---
phase: 13
slug: provider-foundation
status: validated
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-06
validated: 2026-03-06
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in #[test] + cargo test |
| **Config file** | Cargo.toml (default) |
| **Quick run command** | `cargo test cli::` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |
| **Test count** | 124 (cli module) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test cli::`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 13-01-01 | 01 | 1 | PROV-01 | unit | `cargo test cli::ollama::tests::stream_event` | Yes | green |
| 13-01-02 | 01 | 1 | PROV-02 | unit | `cargo test cli::ollama::tests::parse_` | Yes | green |
| 13-01-03 | 01 | 1 | PROV-02 | unit | `cargo test cli::ollama::tests::ollama_provider` | Yes | green |
| 13-02-01 | 02 | 2 | PROV-03 | compile | `cargo check` | N/A | green |
| 13-03-01 | 03 | 1 | PROV-03 | unit | `cargo test cli::ollama::tests::validate_ollama_url` | Yes | green |

*Status: pending / green / red / flaky*

---

## Test Coverage Summary

| Requirement | Tests | Coverage |
|-------------|-------|----------|
| PROV-01 (AiProvider trait + StreamEvent) | `stream_event_token`, `stream_event_done`, `stream_event_error` | COVERED |
| PROV-02 (OllamaProvider + NDJSON) | `parse_tags_*` (3), `parse_ndjson_*` (4), `ollama_provider_*` (3), `serialize_message_*` (3), `parse_raw_tool_calls_*` (12), `strip_thinking_*` (3) | COVERED |
| PROV-03 (URL validation) | `validate_ollama_url_*` (10) | COVERED |
| PROV-03 (UI: status icon, ComboBox) | N/A — visual UI | MANUAL-ONLY |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Status icon color in AI bar | PROV-03 | Visual UI element | Start editor, verify green icon when Ollama running, red when stopped |
| Streaming doesn't block UI | PROV-02 | UI responsiveness | Start stream, interact with editor tabs/panels during generation |
| ComboBox model display | PROV-03 | Visual UI element | Verify model names without :latest suffix in dropdown |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** validated

---

## Validation Audit 2026-03-06

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Total automated tests | 124 |
| Manual-only items | 3 |

*Note: Module renamed from `ai::` to `cli::` — all test paths updated accordingly.*
