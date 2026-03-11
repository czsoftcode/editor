---
phase: 27
slug: 4th-light-theme
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 27 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) |
| **Config file** | none — standard Cargo workspace setup |
| **Quick run command** | `cargo test -q light_variant --bins --tests` |
| **Full suite command** | `./check.sh` |
| **Estimated runtime** | ~60-120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -q light_variant --bins --tests`
- **After every plan wave:** Run `cargo check && ./check.sh` (`./check.sh` informational if known fmt drift persists)
- **Before `$gsd-verify-work`:** Full suite should be green; known out-of-scope fmt drift may be informational
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 27-02-01 | 02 | 2 | THEME-01,THEME-03 | regression | `cargo test -q settings_light_variant_picker_includes_warmtan` | ✅ / ❌ W0 | ⬜ pending |
| 27-02-02 | 02 | 2 | THEME-01,THEME-02 | regression | `cargo test -q settings_light_variant_switch_to_warmtan` | ✅ / ❌ W0 | ⬜ pending |
| 27-02-03 | 02 | 2 | THEME-04 | regression | `cargo test -q settings_light_variant_label_warmtan_localized` | ✅ / ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/app/ui/workspace/modal_dialogs/settings.rs` — test stubs for picker option visibility and switching path
- [ ] `src/settings.rs` — guard tests for enum/list parity used by UI picker

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| WarmTan je viditelný a přepínatelný v Settings UI | THEME-01,THEME-03 | UX tok v modalu je potřeba ověřit interaktivně | Otevři Settings → Theme picker, ověř přítomnost WarmTan swatche, přepni na něj a zkontroluj okamžitý vizuální apply |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency <= 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
