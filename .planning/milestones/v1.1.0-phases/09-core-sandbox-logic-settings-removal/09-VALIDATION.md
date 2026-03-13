---
phase: 9
slug: core-sandbox-logic-settings-removal
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-05
---

# Phase 9 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[cfg(test)]` + `#[test]` |
| **Config file** | Cargo.toml (standard) |
| **Quick run command** | `cargo check 2>&1` |
| **Full suite command** | `cargo test 2>&1` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check 2>&1`
- **After every plan wave:** Run `cargo test 2>&1`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 09-01-01 | 01 | 1 | CORE-01 | smoke | `test ! -f src/app/sandbox.rs && echo OK` | N/A | ⬜ pending |
| 09-01-02 | 01 | 1 | CORE-02 | smoke | `! grep -r 'struct Sandbox' src/ && echo OK` | N/A | ⬜ pending |
| 09-02-01 | 02 | 1 | SET-01 | unit | `cargo test settings::tests -- 2>&1` | Needs update | ⬜ pending |
| 09-02-02 | 02 | 1 | SET-02 | unit | `cargo test settings::tests -- 2>&1` | Needs update | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Delete `sandbox::tests` module (in sandbox.rs being deleted)
- [ ] Delete `state::tests::should_apply_sandbox_request` tests (3 tests)
- [ ] Delete `modal_dialogs/settings.rs` sandbox tests (6 tests, lines 732-826)
- [ ] Update `settings::tests::test_sett02_canonical_toml_persists_sandbox_mode` -- delete (tests removed feature)
- [ ] Update `settings::tests::test_sett05_legacy_project_read_only_maps_to_sandbox_mode` -- convert to migration test
- [ ] Add new test: settings migration strips sandbox_mode from TOML
- [ ] Add new test: settings migration strips project_read_only from JSON

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Settings file migration on app start | SET-01, SET-02 | Requires real settings file on disk | 1. Create settings.toml with `sandbox_mode = true`, 2. Start app, 3. Verify field removed from file |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
