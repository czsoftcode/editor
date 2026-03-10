---
phase: 20
slug: gsd-core-state-engine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-07
---

# Phase 20 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[cfg(test)]` + `cargo test` |
| **Config file** | Cargo.toml (existing) |
| **Quick run command** | `cargo test --lib gsd -- --nocapture` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib gsd -- --nocapture`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 20-01-01 | 01 | 1 | CORE-01 | unit | `cargo test --lib frontmatter -- --nocapture` | ❌ W0 | ⬜ pending |
| 20-01-02 | 01 | 1 | CORE-02 | unit | `cargo test --lib frontmatter::tests::round_trip -- --nocapture` | ❌ W0 | ⬜ pending |
| 20-01-03 | 01 | 1 | CORE-03 | unit | `cargo test --lib config -- --nocapture` | ❌ W0 | ⬜ pending |
| 20-01-04 | 01 | 1 | CORE-04 | unit | `cargo test --lib paths -- --nocapture` | ❌ W0 | ⬜ pending |
| 20-01-05 | 01 | 1 | CORE-05 | unit | `cargo test --lib gsd::tests::missing_planning -- --nocapture` | ❌ W0 | ⬜ pending |
| 20-02-01 | 02 | 2 | STATE-01 | unit | `cargo test --lib state::tests::state_display -- --nocapture` | ❌ W0 | ⬜ pending |
| 20-02-02 | 02 | 2 | STATE-02 | unit | `cargo test --lib state::tests::state_update -- --nocapture` | ❌ W0 | ⬜ pending |
| 20-02-03 | 02 | 2 | STATE-03 | unit | `cargo test --lib state::tests::state_patch -- --nocapture` | ❌ W0 | ⬜ pending |
| 20-02-04 | 02 | 2 | STATE-04 | unit | `cargo test --lib state::tests::progress_display -- --nocapture` | ❌ W0 | ⬜ pending |
| 20-02-05 | 02 | 2 | STATE-05 | unit | `cargo test --lib state::tests::append_section -- --nocapture` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/app/ui/terminal/ai_chat/gsd/frontmatter.rs` — `#[cfg(test)] mod tests` with parse, round-trip, tolerant parsing tests
- [ ] `src/app/ui/terminal/ai_chat/gsd/config.rs` — `#[cfg(test)] mod tests` with get/set/dot-notation tests
- [ ] `src/app/ui/terminal/ai_chat/gsd/paths.rs` — `#[cfg(test)] mod tests` with slug, numbering tests
- [ ] `src/app/ui/terminal/ai_chat/gsd/state.rs` — `#[cfg(test)] mod tests` with state display, update, progress tests
- [ ] `src/app/ui/terminal/ai_chat/gsd/mod.rs` — `#[cfg(test)] mod tests` with dispatch routing, missing .planning/ tests

*Existing infrastructure covers framework setup — only test modules need creation.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `/gsd state` renders formatted markdown in chat panel | STATE-01 | Visual rendering in egui_commonmark | Type `/gsd state` in chat, verify markdown renders correctly |
| `/gsd progress` shows visual progress bar | STATE-04 | Unicode block rendering in egui | Type `/gsd progress`, verify progress bar displays with correct fill |
| Missing `.planning/` shows friendly message | CORE-05 | Requires opening project without `.planning/` | Open a non-GSD project, type `/gsd state`, verify helpful message appears |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
