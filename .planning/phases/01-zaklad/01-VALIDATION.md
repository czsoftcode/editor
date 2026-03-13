---
phase: 1
slug: zaklad
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-04
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (existující) |
| **Quick run command** | `cargo test --lib 2>&1 | tail -5` |
| **Full suite command** | `cargo test 2>&1` |
| **Estimated runtime** | ~10–30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib 2>&1 | tail -5`
- **After every plan wave:** Run `cargo test 2>&1`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| LightVariant enum | 01 | 1 | THEME-01 | unit | `cargo test settings` | ⬜ pending |
| serde default | 01 | 1 | SETT-04 | unit | `cargo test serde_compat` | ⬜ pending |
| to_egui_visuals() | 01 | 1 | THEME-02 | unit | `cargo test theme` | ⬜ pending |
| syntect_theme_name() | 01 | 1 | THEME-02 | unit | `cargo test theme` | ⬜ pending |
| Highlighter parametrizace | 02 | 1 | EDIT-01 | unit | `cargo test highlighter` | ⬜ pending |
| Solarized Light | 02 | 1 | EDIT-02 | unit | `cargo test highlighter` | ⬜ pending |
| cache hash s theme | 02 | 1 | EDIT-04 | unit | `cargo test highlighter` | ⬜ pending |
| startup apply | 03 | 2 | THEME-03 | manual | Vizuální kontrola | ⬜ pending |
| settings_version propagace | 03 | 2 | THEME-04 | manual | Přepnutí + kontrola | ⬜ pending |
| UI barvy light mode | 03 | 2 | UI-01, UI-02, UI-03 | manual | Vizuální kontrola | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/settings.rs` — přidat unit test modul pro `LightVariant` default a serde round-trip
- [ ] `src/highlighter.rs` — rozšířit existující test modul o parametrizaci tématu

*Existující test infrastruktura (cargo test) pokrývá vše — žádný nový framework.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Startup bez flash | THEME-03 | Vyžaduje vizuální pozorování prvního frame | Spustit app s light mode v settings.json; ověřit že první frame je světlý |
| settings_version propagace | THEME-04 | Vyžaduje běžící UI | Přepnout téma v Settings, ověřit okamžitou změnu bez restartu |
| Čitelnost UI v light mode | UI-01, UI-02, UI-03 | Vizuální kontrast | Přepnout do light mode, zkontrolovat menu, záložky, status bar |
| Syntax highlighting v light | EDIT-02, EDIT-03 | Vizuální kontrola barev | Otevřít .rs soubor v obou módech, ověřit čitelnost |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
