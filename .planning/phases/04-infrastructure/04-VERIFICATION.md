---
phase: 04-infrastructure
verified: 2026-03-05T03:33:48Z
status: human_needed
score: 5/5 must-haves verified
---

# Phase 04: Infrastructure Verification Report

**Phase Goal:** Základní přepínač sandbox režimu v Settings > Projekt napojený na terminály a init projektu; OFF = práce v rootu a terminály v rootu; změna se projeví po znovuotevření projektu.
**Verified:** 2026-03-05T03:33:48Z
**Status:** human_needed

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Settings obsahují nový boolean pro sandbox režim a je persistován do settings.toml. | ✓ VERIFIED | `src/settings.rs` má `sandbox_mode` + test `test_sett02_canonical_toml_persists_sandbox_mode`. |
| 2 | Přepínač v Settings > Projekt respektuje Save/Cancel semantiku a změna se projeví až po znovuotevření projektu. | ✓ VERIFIED | Settings modal ukládá `sandbox_mode` až na Save; runtime používá `sandbox_mode_enabled` pouze z initu. |
| 3 | Při startu projektu se build_in_sandbox a file_tree_in_sandbox nastaví z uloženého sandbox režimu. | ✓ VERIFIED | `src/app/ui/workspace/state/init.rs` nastavuje `build_in_sandbox` i `file_tree_in_sandbox` z `settings.sandbox_mode`. |
| 4 | Terminály používají správný cwd podle sandbox režimu a label odpovídá (ON: Sandbox, OFF: Terminal + cesta). | ✓ VERIFIED | `terminal_working_dir` a `terminal_mode_label` v `src/app/ui/terminal/mod.rs` + použití v build bar/terminálech. |
| 5 | Runtime UI nečte `settings.sandbox_mode` přímo; změna se uplatní jen přes init (apply on reopen). | ✓ VERIFIED | `rg settings.sandbox_mode src/app` vrací jen init; runtime čte `sandbox_mode_enabled`. |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/settings.rs` | Pole a persistování sandbox režimu (settings.toml) + testy persistence | ✓ EXISTS + SUBSTANTIVE | `sandbox_mode` + testy migrace/roundtrip. |
| `src/app/ui/workspace/modal_dialogs/settings.rs` | UI přepínač + tooltip + inline info o reopen + toasty + save handler | ✓ EXISTS + SUBSTANTIVE | Toggle, tooltip, toasty, Save handler. |
| `src/app/ui/workspace/state/init.rs` | Aplikace flagů build_in_sandbox / file_tree_in_sandbox při initu | ✓ EXISTS + SUBSTANTIVE | Nastavení z `settings.sandbox_mode`. |
| `src/app/ui/terminal/mod.rs` | CWD a label terminálů podle sandbox režimu | ✓ EXISTS + SUBSTANTIVE | `terminal_working_dir` + `terminal_mode_label`. |

**Artifacts:** 4/4 verified

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| Settings Save | settings.toml | persist sandbox_mode | ✓ WIRED | `Settings::save` serializuje `sandbox_mode` do TOML. |
| Workspace init | build_in_sandbox / file_tree_in_sandbox | settings.sandbox_mode | ✓ WIRED | Init nastavuje oba flagy z settings. |
| sandbox OFF | terminal cwd | root project path | ✓ WIRED | `terminal_working_dir(false, _, root)` vrací `root`. |

**Wiring:** 3/3 connections verified

## Requirements Coverage

| Requirement | Status | Blocking Issue |
|-------------|--------|----------------|
| SETT-01 | ✓ SATISFIED | - |
| SETT-02 | ✓ SATISFIED | - |
| SETT-03 | ✓ SATISFIED | - |
| SETT-04 | ✓ SATISFIED | - |
| SETT-05 | ✓ SATISFIED | - |
| TERM-01 | ✓ SATISFIED | - |
| TERM-02 | ✓ SATISFIED | - |
| TERM-03 | ✓ SATISFIED | - |

**Coverage:** 8/8 requirements satisfied

## Anti-Patterns Found

None observed in code review.

## Human Verification Required

### 1. Settings Save/Cancel + toast
**Test:** Otevři Settings > Projekt, přepni sandbox OFF, Cancel, ověř beze změny. Pak OFF + Save, ověř toast a zápis do `settings.toml`.
**Expected:** Cancel neuloží; Save uloží + zobrazí toast.
**Why human:** UI flow.

### 2. Reopen apply-on-reopen
**Test:** Po uložení sandbox OFF znovu otevři projekt.
**Expected:** `build_in_sandbox` a `file_tree_in_sandbox` odpovídají OFF.
**Why human:** Runtime behavior.

### 3. Terminály cwd a label
**Test:** Po reopen projektu ověř, že terminály běží v rootu a label je `Terminal — <path>`. Poté zapni sandbox a ověř label `Sandbox`.
**Expected:** CWD a label odpovídají režimu.
**Why human:** UI/PTY behavior.

## Gaps Summary

**No gaps found.** Phase goal achieved pending human verification.

## Verification Metadata

**Verification approach:** Goal-backward (derived from phase goal)
**Must-haves source:** 04-01-PLAN.md frontmatter
**Automated checks:** 2 passed (cargo test settings, cargo check)
**Human checks required:** 3
**Total verification time:** 8 min

---
*Verified: 2026-03-05T03:33:48Z*
*Verifier: Codex (manual)*
