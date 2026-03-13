---
phase: 26
slug: save-ux-polish-regression-hardening
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-10
---

# Phase 26 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) |
| **Config file** | none — standard Cargo workspace setup |
| **Quick run command** | `cargo check && cargo test -q <TARGETED_TEST> -- --nocapture` |
| **Informational sweep** | `./check.sh` (neblokující; pouze informativní) |
| **Estimated runtime** | ~45-90 seconds (per-task gate), ~180 seconds (informational sweep) |

---

## Sampling Rate

- **After every task commit (hard gate):** Run `cargo check` + targeted test name(s) pro právě měněnou větev (`cargo test <TESTNAME> -- --nocapture`).
- **After every plan wave (hard gate):** Re-run `cargo check` + targeted tests mapované na dokončené tasky v dané wave.
- **Informational sweep (non-blocking):** `./check.sh` lze spustit po wave kvůli repo-wide driftu; fail neblokuje fázi 26.
- **Before `$gsd-verify-work`:** Hard gate je `cargo check` + všechny relevantní cílené testy z mapy níže.
- **Max feedback latency:** 90 seconds (per-task hard gate), 180 seconds (informational sweep)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 26-01-01 | 01 | 1 | MODE-04 | unit | `cargo test -q save_mode_status -- --nocapture` | ✅ | ⬜ pending |
| 26-01-02 | 01 | 1 | MODE-04 | ui | `cargo test -q tab_save_mode_indicator -- --nocapture` | ✅ | ⬜ pending |
| 26-01-03 | 01 | 1 | MODE-04 | regression | `cargo test -q mode_04_runtime_visibility -- --nocapture` | ✅ | ⬜ pending |
| 26-02-01 | 02 | 2 | MODE-04 | ui | `cargo test -q dirty_state_visual_priority -- --nocapture` | ✅ | ⬜ pending |
| 26-02-02 | 02 | 2 | MODE-04 | regression | `cargo test -q save_ux_contrast_regression -- --nocapture` | ✅ | ⬜ pending |
| 26-03-01 | 03 | 2 | MODE-04 | unit | `cargo test -q manual_save_request -- --nocapture` | ✅ | ⬜ pending |
| 26-03-02 | 03 | 2 | MODE-04 | regression | `cargo test -q unsaved_close_guard_save_failure_feedback -- --nocapture` | ✅ | ⬜ pending |
| 26-03-03 | 03 | 2 | MODE-04 | unit | `cargo test -q save_error_dedupe -- --nocapture` | ✅ | ⬜ pending |
| 26-04-01 | 04 | 3 | MODE-04 | i18n | `cargo test -q all_lang_keys_match_english -- --nocapture && cargo test -q save_ux_i18n_smoke -- --nocapture` | ✅ | ⬜ pending |
| 26-04-02 | 04 | 3 | MODE-04 | regression | `cargo test -q save_ux_idle_safety_guard -- --nocapture` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Žádné otevřené Wave 0 závislosti; všechny tasky mají explicitní `<automated>` verify v plánech 26-01..26-04.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Light/dark čitelnost dirty/clean indikace | MODE-04 | Kontrast a vizuální hierarchie je potřeba ověřit lidským pohledem | Otevři editor v light + dark mode, vytvoř změnu, porovnej čitelnost badgů a stavové lišty při dirty/clean přepnutí |

---

## Validation Sign-Off

- [ ] All 10 tasks have explicit `<automated>` verify commands
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Verification map matches plans 26-01..26-04 (10/10 tasks)
- [ ] No watch-mode flags
- [ ] Per-task feedback latency <= 90s (smoke/unit gate)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
