---
phase: 24
slug: save-mode-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-09
---

# Phase 24 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo check` |
| **Full suite command** | `./check.sh` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check`
- **After every plan wave:** Run `./check.sh`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

### Sampling Execution Matrix

| Scope | Trigger | Command | Evidence |
|-------|---------|---------|----------|
| Task-level | Každý task commit v plánech 24-01..24-03 | `cargo check` | Commit + lokální výstup příkazu |
| Wave-level | Dokončení každé wave | `./check.sh` | Lokální výstup příkazu |
| Pre-UAT gate | Před `gsd-verify-work` | `cargo check && ./check.sh` | Poslední green běh ve validačním protokolu |

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement IDs | Verification Type | Evidence Command | Sampling Gate | Status |
|---------|------|------|-----------------|-------------------|------------------|---------------|--------|
| 24-01-01 | 01 | 1 | MODE-01, MODE-02 | automated (compile) | `cargo check` | Task commit gate | ✅ |
| 24-01-02 | 01 | 1 | MODE-01, MODE-02 | automated (unit) | `cargo test settings:: -- --nocapture` | Task commit gate + Wave 1 gate | ✅ |
| 24-02-01 | 02 | 2 | MODE-01, MODE-03 | automated (compile) | `cargo check` | Task commit gate | ✅ |
| 24-02-02 | 02 | 2 | MODE-03, SAVE-01 | automated + manual | `cargo check` + viz `M-CTRL-S-MODAL` | Task commit gate | ✅ |
| 24-02-03 | 02 | 2 | MODE-01, MODE-03 | automated + manual | `cargo check` + viz `M-RUNTIME-APPLY` | Task commit gate + Wave 2 gate | ✅ |
| 24-03-01 | 03 | 3 | SAVE-01, SAVE-02, SAVE-03, MODE-03 | automated + manual | `cargo check` + viz `M-CTRL-S-EDITOR`, `M-SAVE-FAILURE` | Task commit gate | ✅ |
| 24-03-02 | 03 | 3 | MODE-03 | automated (compile) | `cargo check` | Task commit gate | ✅ |
| 24-03-03 | 03 | 3 | SAVE-03 | automated (targeted test) | `cargo check && cargo test save_error_dedupe -- --nocapture` | Task commit gate + Wave 3 gate | ✅ |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Requirement Coverage Matrix

| Requirement ID | Covered by Task IDs | Primary Verification Step |
|----------------|---------------------|---------------------------|
| SAVE-01 | 24-02-02, 24-03-01 | `M-CTRL-S-EDITOR` + `cargo check` |
| SAVE-02 | 24-03-01 | `M-CTRL-S-EDITOR` (modified -> saved bez focus change) |
| SAVE-03 | 24-03-01, 24-03-03 | `M-SAVE-FAILURE` + `cargo test save_error_dedupe -- --nocapture` |
| MODE-01 | 24-01-01, 24-01-02, 24-02-01, 24-02-03 | `cargo test settings:: -- --nocapture` + `M-RESTART-PERSISTENCE` |
| MODE-02 | 24-01-01, 24-01-02 | `cargo test settings:: -- --nocapture` |
| MODE-03 | 24-02-01, 24-02-02, 24-02-03, 24-03-01, 24-03-02 | `M-RUNTIME-APPLY` + `cargo check` |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `Ctrl+S` uloží aktivní tab bez změny fokusu | SAVE-01, SAVE-02 | vyžaduje běžící UI + fokus workflow | Otevřít soubor, změnit obsah, stisknout `Ctrl+S`, ověřit status `Saved` a obsah na disku |
| Chyba ukládání je viditelná v toastu | SAVE-03 | failure injection (permissions) | Otevřít read-only soubor, provést save, ověřit error toast a zachování `Modified` |
| Přepínač Automatic/Manual se persistuje přes restart | MODE-01, MODE-02 | vyžaduje restart aplikace | V Settings přepnout režim, uložit, restartovat app, ověřit zachování volby |
| Změna save mode se aplikuje ihned po Save | MODE-03 | runtime behavior check | Přepnout z Manual na Automatic a zpět, ověřit chování autosave bez restartu |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
