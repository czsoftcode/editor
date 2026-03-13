---
phase: 03
slug: light-varianty-settings-ui
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-04
---

# Phase 03 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo check` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~90 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check`
- **After every plan wave:** Run `cargo test`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 03-TBD-LITE-01 | TBD | TBD | LITE-01 | visual + unit | `cargo check` | ✅ | ⬜ pending |
| 03-TBD-LITE-02 | TBD | TBD | LITE-02 | integration + manual | `cargo check` | ✅ | ⬜ pending |
| 03-TBD-LITE-03 | TBD | TBD | LITE-03 | integration + restart | `cargo check` | ✅ | ⬜ pending |
| 03-TBD-LITE-04 | TBD | TBD | LITE-04 | visual + manual | `cargo check` | ✅ | ⬜ pending |
| 03-TBD-SETT-01 | TBD | TBD | SETT-01 | UI rendering | `cargo check` | ✅ | ⬜ pending |
| 03-TBD-SETT-02 | TBD | TBD | SETT-02 | runtime behavior | `cargo check` | ✅ | ⬜ pending |
| 03-TBD-SETT-03 | TBD | TBD | SETT-03 | persistence | `cargo check` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Existing infrastructure covers all phase requirements.
- [ ] Po vytvoření konkrétních `03-0X-PLAN.md` doplnit mapu `Task ID -> Plan/Wave`.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live preview změna varianty ve všech oknech | SETT-02 | Multi-viewport UI flow | Otevřít root + secondary viewport, měnit varianty v Settings a ověřit okamžitou propagaci bez restartu |
| Revert při Cancel po preview | SETT-02 | UX semantika modalu | Otevřít Settings, změnit theme/variantu, stisknout Cancel, ověřit návrat na původní stav |
| Viditelnost pickeru jen v light mode | SETT-01 | Podmíněné renderování UI | Přepnout Dark/Light a ověřit, že picker je skrytý v dark a zobrazený v light |
| Kontrast panelů (`faint_bg_color`) | LITE-04 | Subjektivní vizuální kvalita | Porovnat editor/file tree/side panel v každé variantě, zkontrolovat nesplývání při běžném zoomu |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
