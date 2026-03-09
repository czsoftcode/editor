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

| Scenario ID | Behavior | Requirement IDs | Preconditions | Steps | PASS | FAIL |
|-------------|----------|-----------------|--------------|-------|------|------|
| `M-CTRL-S-EDITOR` | `Ctrl+S` uloží aktivní tab bez změny fokusu | SAVE-01, SAVE-02 | Otevřený editovatelný soubor, změněný obsah (`Modified`) | 1) Změnit text v aktivním tabu. 2) Bez změny fokusu stisknout `Ctrl+S`. 3) Ověřit tab status. 4) Ověřit obsah na disku. | Tab se okamžitě přepne na `Saved`; obsah souboru odpovídá editoru; toast nehlásí chybu. | Tab zůstane `Modified`, disk neobsahuje změny, nebo se objeví neočekávaná chyba/no-op toast. |
| `M-SAVE-FAILURE` | Save failure je viditelný a tab zůstane `Modified` | SAVE-03 | Soubor bez write oprávnění (nebo jiný reprodukovatelný write error) | 1) Otevřít read-only soubor. 2) Upravit obsah. 3) Spustit save (`Ctrl+S` nebo menu Save). | Zobrazí se error toast; tab zůstane `Modified`; žádný tichý fail. | Error toast se neobjeví, nebo tab přejde do `Saved` i přes neúspěch zápisu. |
| `M-CTRL-S-MODAL` | `Ctrl+S` v Settings modalu ukládá draft settings | SAVE-01, MODE-03 | Otevřený Settings modal, změněný `save_mode` v draftu | 1) Otevřít Settings. 2) Přepnout `save_mode` v draftu. 3) Stisknout `Ctrl+S` bez zavření modalu. | Persistuje se settings draft (ekvivalent modal Save); editor file-save flow se nespustí. | Uloží se editor soubor místo settings, nebo se neuloží nic. |
| `M-RESTART-PERSISTENCE` | Save mode se persistuje přes restart | MODE-01, MODE-02 | Aplikace běží, lze měnit Settings | 1) Nastavit `Automatic` a uložit. 2) Restartovat aplikaci. 3) Ověřit hodnotu. 4) Opakovat pro `Manual`. | Po restartu je aktivní přesně naposledy uložený režim; stará konfigurace bez `save_mode` naběhne jako `Manual`. | Po restartu je jiná hodnota, nebo chybí backward-compatible fallback na `Manual`. |
| `M-RUNTIME-APPLY` | Změna save mode se aplikuje bez restartu | MODE-03 | App běží, Settings dostupné | 1) Přepnout Manual -> Automatic, uložit. 2) Ověřit, že autosave behavior odpovídá Automatic. 3) Přepnout zpět na Manual, uložit. 4) Ověřit, že autosave je vypnutý bez restartu. | Chování autosave se změní okamžitě po Save v modalu v obou směrech. | Chování se změní až po restartu, nebo se nezmění vůbec. |

---

## Validation Sign-Off

### Nyquist Sign-Off Gate (`nyquist_compliant: true`)

- [ ] `24-VALIDATION.md` obsahuje kompletní mapování tasků 24-01..24-03 (žádné chybějící task ID).
- [ ] Všech 6 requirement ID (`SAVE-01..03`, `MODE-01..03`) má alespoň jeden jasný verifikační krok.
- [ ] Proběhl `cargo check` po task změnách validační dokumentace.
- [ ] Proběhl `./check.sh` jako wave-level gate před `gsd-verify-work`.
- [ ] Všechny manual scénáře `M-CTRL-S-EDITOR`, `M-SAVE-FAILURE`, `M-CTRL-S-MODAL`, `M-RESTART-PERSISTENCE`, `M-RUNTIME-APPLY` mají výsledek `PASS`.
- [ ] Každý manual scénář má binární výsledek `PASS` nebo `FAIL` bez nejednoznačných poznámek.
- [ ] Sign-off review neobsahuje otevřené blokery pro uzavření fáze 24.

### Sign-Off Result

| Checkpoint | Result | Evidence |
|------------|--------|----------|
| Coverage map complete | PASS | Task table + Requirement Coverage Matrix |
| Automated gates green | PASS | `cargo check`, `./check.sh` |
| Manual scenarios complete | PENDING | Vyplní se během `gsd-verify-work` |
| Ready to flip `nyquist_compliant` | PENDING | Přepnout na `true` až po PASS všech manual scénářů |

**Approval:** pending
