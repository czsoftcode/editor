---
phase: 38
slug: watcher-stability-verification
status: ready
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-12
---

# Phase 38 - Validation Strategy

> Nyquist-ready validation contract pro RELIAB-03 (watcher stability + final quality gate evidence).

---

## Requirement Contract

| Requirement | Scope | Signal | PASS | FAIL | Source Evidence |
|-------------|-------|--------|------|------|-----------------|
| RELIAB-03 | Watcher merge/dedupe policy | `phase38_dedupe_path_kind`, `phase38_remove_precedence` | Oba testy projdou a potvrdi dedupe podle `path+kind` + remove-priority merge | Libovolny test failne nebo chybi v focused runu | `tests/phase38_watcher_stability.rs`, `cargo test phase38 -- --nocapture` |
| RELIAB-03 | Overflow fallback | `phase38_overflow_sets_fallback`, `phase38_overflow_triggers_single_reload` | Unit i orchestration test potvrdi overflow signal + one-shot reload guard | Overflow branch neexistuje, nebo test failne | `tests/phase38_watcher_stability.rs`, `tests/phase38_background_orchestration.rs` |
| RELIAB-03 | Anti reload storm + orchestration stabilita | `phase38_batch_applies_deduped_changes` + single reload helper hook | Background vrstva aplikuje dedupe helper a overflow branch vola one-shot helper | Chybi helper hook nebo branch failne | `src/app/ui/background.rs`, focused phase38 test run |
| RELIAB-03 | Final release gate | `cargo check` + `./check.sh` | Obe command-level gate kontroly jsou explicitne zaznamenane jako PASS ve verification reportu | Jedna z gate kontrol failne nebo chybi evidence | `.planning/phases/38-watcher-stability-verification/38-VERIFICATION.md` |

## Nyquist Validation Matrix

| Validation Item | Layer | Automated/Manual | PASS/FAIL Criteria | Evidence Source |
|-----------------|-------|------------------|--------------------|-----------------|
| Merge/dedupe deterministic final state | Unit | Automated | PASS: `phase38_dedupe_path_kind` PASS, FAIL: assert mismatch | `tests/phase38_watcher_stability.rs` |
| Remove precedence across colliding events | Unit | Automated | PASS: `phase38_remove_precedence` PASS, FAIL: remove nezvitezil | `tests/phase38_watcher_stability.rs` |
| Overflow fallback strips granular replay | Unit | Automated | PASS: `phase38_overflow_sets_fallback` PASS, FAIL: non-empty changes pri overflow | `tests/phase38_watcher_stability.rs` |
| Overflow branch triggers one reload only | Orchestration | Automated | PASS: `phase38_overflow_triggers_single_reload` PASS, FAIL: chybi one-shot hook | `tests/phase38_background_orchestration.rs` |
| Batch applies deduped changes only | Orchestration | Automated | PASS: `phase38_batch_applies_deduped_changes` PASS, FAIL: chybi dedupe apply hook | `tests/phase38_background_orchestration.rs` |
| Visible anti-storm UX behavior | Manual | Manual | PASS: zadny reload/toast storm po burstu eventu, FAIL: opakovany refresh loop | Manual scenario + verification report |
| Final gate integrity | Build/QA | Automated | PASS: `cargo check` + `./check.sh` oba PASS, FAIL: libovolny gate fail | Verification report command evidence |

## Test Design

1. Focused smoke: `RUSTC_WRAPPER= cargo test phase38 -- --nocapture`.
2. Unit assertions:
`phase38_dedupe_path_kind`, `phase38_remove_precedence`, `phase38_overflow_sets_fallback`.
3. Orchestration assertions:
`phase38_batch_applies_deduped_changes`, `phase38_overflow_triggers_single_reload`.
4. Stabilita naming/hook kontraktu:
test names musi korespondovat s RELIAB-03 mapou ve verification artefaktu.
5. Final gate: `RUSTC_WRAPPER= cargo check` a `RUSTC_WRAPPER= ./check.sh`.

PASS policy: vsechny focused testy + obe gate commandy PASS.
FAIL policy: jakykoliv fail prerusuje sign-off a reportuje se do `38-VERIFICATION.md`.

## Manual Verification Scenario

| Scenario | Steps | PASS | FAIL | Evidence |
|----------|-------|------|------|----------|
| Anti reload storm pri burstu file eventu | 1) Otevri projekt s watcherem. 2) Vygeneruj burst create/modify/remove zmen. 3) Sleduj refresh chovani stromu. | Max jeden reload pulse na overflow, bez opakovaneho toast loopu a bez zamrznuti UI | Opakovane reloady/toasty nebo viditelne sekani UI | Poznamka v `38-VERIFICATION.md` manual section |
| Remove-priority consistency | 1) Vytvor soubor. 2) Hned uprav a smaz. 3) Sleduj finalni stav ve stromu. | Finalni stav je odstraneni bez navratu phantom tabu | Phantom navrat nebo stale create/modify stav | Poznamka v `38-VERIFICATION.md` manual section |

## Gate Execution Plan

| Gate | Command | Expected Result | Evidence Sink |
|------|---------|-----------------|---------------|
| Focused RELIAB-03 gate | `RUSTC_WRAPPER= cargo test phase38 -- --nocapture` | PASS | `38-VERIFICATION.md` command log |
| Compiler gate | `RUSTC_WRAPPER= cargo check` | PASS | `38-VERIFICATION.md` command log |
| Full quality gate | `RUSTC_WRAPPER= ./check.sh` | PASS | `38-VERIFICATION.md` command log |

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness (`cargo test`) |
| **Config file** | none — standard Cargo workflow |
| **Quick run command** | `RUSTC_WRAPPER= cargo test phase38 -- --nocapture` |
| **Full suite command** | `RUSTC_WRAPPER= ./check.sh` |
| **Estimated runtime** | ~120 seconds |

---

## Validation Sign-Off

- [x] Requirement Contract exists with measurable PASS/FAIL and source evidence
- [x] Nyquist Validation Matrix contains unit + orchestration + manual + gate signals
- [x] Test Design aligns hooks with RELIAB-03 map
- [x] Manual Verification Scenario captures non-automated UX checks
- [x] Gate Execution Plan defines focused + compiler + full quality gates
- [x] `nyquist_compliant: true` set in frontmatter after completeness check

**Approval:** ready for execution evidence capture
