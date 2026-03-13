---
phase: 19
slug: slash-command-infrastructure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-07
---

# Phase 19 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[cfg(test)]` + `#[test]` |
| **Config file** | Cargo.toml (standard) |
| **Quick run command** | `cargo test slash` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test slash`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 19-01-01 | 01 | 1 | SLASH-08 | unit | `cargo test slash::tests::dispatch_intercept` | ❌ W0 | ⬜ pending |
| 19-01-02 | 01 | 1 | SLASH-09 | unit | `cargo test slash::tests::fuzzy_suggest` | ❌ W0 | ⬜ pending |
| 19-02-01 | 02 | 1 | SLASH-01 | unit | `cargo test slash::tests::help_output` | ❌ W0 | ⬜ pending |
| 19-02-02 | 02 | 1 | SLASH-02 | unit | `cargo test slash::tests::clear_resets` | ❌ W0 | ⬜ pending |
| 19-02-03 | 02 | 1 | SLASH-03 | unit | `cargo test slash::tests::new_shows_logo` | ❌ W0 | ⬜ pending |
| 19-02-04 | 02 | 1 | SLASH-07 | unit | `cargo test slash::tests::settings_opens` | ❌ W0 | ⬜ pending |
| 19-03-01 | 03 | 2 | SLASH-04 | unit | `cargo test slash::tests::model_list` | ❌ W0 | ⬜ pending |
| 19-03-02 | 03 | 2 | SLASH-05 | integration | `cargo test slash::tests::git_output` | ❌ W0 | ⬜ pending |
| 19-03-03 | 03 | 2 | SLASH-06 | unit | `cargo test slash::tests::build_starts` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/app/ui/terminal/ai_chat/slash.rs` — test module with stubs for SLASH-01 through SLASH-09
- [ ] Levenshtein distance unit tests (empty strings, identical strings, single char diff)

*Existing infrastructure covers framework setup (Cargo.toml already configured).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| System message distinct background color | SLASH-01..09 | Visual rendering in egui | Type `/help`, verify response has different background than AI responses in both dark/light mode |
| /build async "Building..." placeholder | SLASH-06 | Requires real cargo build + UI timing | Type `/build`, verify "Building..." appears immediately, then updates to result |
| /new shows ASCII logo | SLASH-03 | Visual rendering verification | Type `/new`, verify PolyCredo logo with version/model/rank appears |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
