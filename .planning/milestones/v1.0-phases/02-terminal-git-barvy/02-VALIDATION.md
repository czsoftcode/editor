---
phase: 2
slug: terminal-git-barvy
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-04
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `RUSTC_WRAPPER= cargo check` |
| **Full suite command** | `RUSTC_WRAPPER= cargo test` |
| **Estimated runtime** | ~20–90 seconds |

---

## Sampling Rate

- **After every task commit:** Run `RUSTC_WRAPPER= cargo check`
- **After every plan wave:** Run `RUSTC_WRAPPER= cargo test`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | TERM-01, TERM-02 | integration | `RUSTC_WRAPPER= cargo check` | ✅ | ⬜ pending |
| 02-01-02 | 01 | 1 | TERM-04 | unit/manual hybrid | `RUSTC_WRAPPER= cargo test terminal` | ❌ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | TERM-03 | unit | `RUSTC_WRAPPER= cargo test terminal_scrollbar` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 2 | TREE-01 | unit | `RUSTC_WRAPPER= cargo test file_tree_git` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02 | 2 | TREE-02 | unit/manual hybrid | `RUSTC_WRAPPER= cargo test file_tree_git` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/app/ui/terminal/instance/mod.rs` — přidat testovatelný theme resolver/factory (dark/light) pro terminál
- [ ] `src/app/ui/terminal/instance/render.rs` — extrahovat scrollbar color helper a pokrýt unit testy
- [ ] `src/app/ui/background.rs` nebo nový helper modul — oddělit git status -> semantic mapping pro testy
- [ ] `src/app/ui/file_tree/render.rs` — oddělit light/dark git palette resolver pro unit testy

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Claude panel light appearance | TERM-01 | vizuální kontrast | Přepnout na light mode, otevřít Claude panel, ověřit background/text čitelnost |
| Build terminál light appearance | TERM-02 | vizuální kontrast | Spustit build/output v light mode, ověřit konzistenci s Claude panelem |
| Runtime switch bez přerušení | TERM-01..04 | vyžaduje běžící proces | Spustit dlouhý příkaz, přepnout dark/light, ověřit že proces pokračuje |
| Scrollbar contrast + hover | TERM-03 | interakční vizuál | V light mode scrollovat dlouhý výstup, ověřit track/thumb idle + hover/drag |
| Git colors readability | TREE-01, TREE-02 | barevné vnímání v UI | Mít M/A/??/D ve file tree, přepnout light mode, ověřit odlišitelnost stavů |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
