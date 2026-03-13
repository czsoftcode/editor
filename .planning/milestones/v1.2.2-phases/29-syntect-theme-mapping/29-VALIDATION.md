---
phase: 29
slug: syntect-theme-mapping
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-10
---

# Phase 29 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) |
| **Config file** | none — standard Cargo workspace setup |
| **Quick run command** | `cargo test -q <single_test_name> --lib` |
| **Full suite command** | `./check.sh` |
| **Estimated runtime** | smoke ~5-30s, full ~60-120s |

---

## Sampling Rate

- **After every task commit:** Run one targeted smoke test (`cargo test -q <single_test_name> --lib`)
- **After every plan wave:** Run `cargo check` and then `./check.sh` as heavy informational gate (`./check.sh` informational if repo-wide fmt drift persists)
- **Before `$gsd-verify-work`:** Full suite should be green; known out-of-scope fmt drift may be informational
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 29-01-01 | 01 | 1 | SYNTAX-01 | smoke/unit | `cargo test -q syntax01_light_mapping_matrix_complete --lib` | ✅ / ❌ W0 | ⬜ pending |
| 29-01-02 | 01 | 1 | SYNTAX-01,SYNTAX-02 | smoke/unit | `cargo test -q syntect_theme_fallback_contract --lib` | ✅ / ❌ W0 | ⬜ pending |
| 29-01-03 | 01 | 1 | SYNTAX-01,SYNTAX-02 | smoke/unit | `cargo test -q syntax02_dark_mapping_matrix_complete --lib` | ✅ / ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/settings.rs` — test stubs for explicit variant-to-theme mapping matrix
- [ ] `src/settings.rs` — tests for fallback when mapped theme name is unavailable

## Wave/Final Heavy Gates

- Po dokončení wave: `cargo check`
- Po dokončení wave/fáze: `./check.sh` (informational, pokud selže jen na známém out-of-scope fmt driftu)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Vizuální soulad syntax barev s vybranou variantou | SYNTAX-01,SYNTAX-02 | Subjektivní vizuální charakter nelze plně pokrýt unit testy | Přepni všechny 4 light + 2 dark varianty a ověř, že syntax tón odpovídá očekávání (WarmTan teplejší, Midnight chladnější) |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency <= 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
