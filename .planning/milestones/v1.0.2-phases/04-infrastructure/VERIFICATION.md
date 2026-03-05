---
phase: 04-infrastructure
verified: 2026-03-05T04:21:19Z
status: human_needed
score: 7/7 must-haves verified (code)
---

# Phase 04: Infrastructure Verification Report

**Phase goal:** Dokoncit infrastrukturu sandbox nastaveni a odstranit UAT gapy v discoverability sandbox tooltipu.
**Verified:** 2026-03-05T04:21:19Z
**Status:** human_needed

## Goal Achievement

### Must-Haves (Plan 04-01)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Settings obsahují nový boolean pro sandbox režim a je persistován do settings.toml. | ✓ VERIFIED | `sandbox_mode` je v `Settings` a persistuje se v testu round-trip. `src/settings.rs:126-155`, `src/settings.rs:577-595`. |
| 2 | Přepínač v Settings > Projekt respektuje Save/Cancel semantiku a změna se projeví až po znovuotevření projektu. | ✓ VERIFIED (code) | Save ukládá `draft.save()` a Cancel zahazuje draft. Runtime pracuje s `sandbox_mode_enabled` uloženým při initu. `src/app/ui/workspace/modal_dialogs/settings.rs:436-492`, `src/app/ui/workspace/state/init.rs:201-203`. |
| 3 | Při startu projektu se `build_in_sandbox` a `file_tree_in_sandbox` nastaví z uloženého sandbox režimu. | ✓ VERIFIED | `file_tree_in_sandbox` i `build_in_sandbox` se nastavují z `settings.sandbox_mode`. `src/app/ui/workspace/state/init.rs:24-34`, `src/app/ui/workspace/state/init.rs:201-203`. |
| 4 | Terminály používají správný cwd podle sandbox režimu a label odpovídá (ON: Sandbox, OFF: Terminal + cesta). | ✓ VERIFIED | CWD i label jsou odvozeny z `sandbox_mode_enabled`. `src/app/ui/terminal/mod.rs:30-47`, `src/app/ui/workspace/mod.rs:82-110`, `src/app/ui/terminal/bottom/build_bar.rs:11-47`. |
| 5 | Runtime UI nečte `settings.sandbox_mode` přímo; změna se uplatní jen přes init (apply on reopen). | ✓ VERIFIED (code) | `settings.sandbox_mode` je použit pouze v initu; UI používá `ws.sandbox_mode_enabled`. `src/app/ui/workspace/state/init.rs:24-25`, `src/app/ui/workspace/mod.rs:82-110`. |

### Must-Haves (Plan 04-02)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 6 | Tooltip k sandbox režimu je snadno objevitelný (větší hover area než jen malá ikona). | ✓ VERIFIED | Tooltip je navázaný na celý řádek (`allocate_ui_with_layout` + `on_hover_text` na `response`). `src/app/ui/workspace/modal_dialogs/settings.rs:269-283`. |
| 7 | Inline poznámka o reopen je vizuálně čitelná (nepůsobí příliš potlačeně). | ✓ VERIFIED | Inline hint je renderovaný přes `RichText::strong()`. `src/app/ui/workspace/modal_dialogs/settings.rs:284-286`. |

### Required Artifacts (Plan 04-01)

| Artifact | Expected | Status | Evidence |
|----------|----------|--------|---------|
| `src/settings.rs` | Pole a persistování sandbox režimu + testy persistence | ✓ EXISTS + SUBSTANTIVE | `sandbox_mode` + testy `test_sett02_*`/`test_sett05_*`. `src/settings.rs:126-129`, `src/settings.rs:577-620`. |
| `src/app/ui/workspace/modal_dialogs/settings.rs` | UI přepínač + tooltip + inline info o reopen + toasty + save handler | ✓ EXISTS + SUBSTANTIVE | Toggle řádek, tooltip, inline info, toasty v Save flow. `src/app/ui/workspace/modal_dialogs/settings.rs:269-287`, `src/app/ui/workspace/modal_dialogs/settings.rs:436-484`. |
| `src/app/ui/workspace/state/init.rs` | Aplikace flagů build_in_sandbox / file_tree_in_sandbox při initu | ✓ EXISTS + SUBSTANTIVE | Init nastavuje flagy z `settings.sandbox_mode`. `src/app/ui/workspace/state/init.rs:24-34`, `src/app/ui/workspace/state/init.rs:201-203`. |
| `src/app/ui/terminal/mod.rs` | CWD a label terminálů podle sandbox režimu | ✓ EXISTS + SUBSTANTIVE | `terminal_working_dir` + `terminal_mode_label`. `src/app/ui/terminal/mod.rs:30-47`. |

### Key Link Verification (Plan 04-01)

| From | To | Via | Status | Evidence |
|------|----|-----|--------|----------|
| Settings Save | settings.toml | persist sandbox_mode | ✓ WIRED | `draft.save()` zapisuje `sandbox_mode` do TOML. `src/app/ui/workspace/modal_dialogs/settings.rs:438-443`, `src/settings.rs:205-218`. |
| Workspace init | build_in_sandbox / file_tree_in_sandbox | settings.sandbox_mode | ✓ WIRED | Init přiřazuje flagy z `settings.sandbox_mode`. `src/app/ui/workspace/state/init.rs:24-34`, `src/app/ui/workspace/state/init.rs:201-203`. |
| sandbox OFF | terminal cwd | root project path | ✓ WIRED | `terminal_working_dir(false, _, root)` vrací `project_root`. `src/app/ui/terminal/mod.rs:38-47`. |

## Requirements Cross-Reference

Plan frontmatter requirements:
- 04-01: `SETT-01`, `SETT-02`, `SETT-03`, `SETT-04`, `SETT-05`, `TERM-01`, `TERM-02`, `TERM-03`
- 04-02: `SETT-01`, `SETT-02`, `SETT-03`, `SETT-04`, `SETT-05`

All IDs are present in `REQUIREMENTS.md`:

| Requirement ID | Present in REQUIREMENTS.md | Evidence |
|---|---|---|
| SETT-01 | ✓ | `.planning/REQUIREMENTS.md:41-47` |
| SETT-02 | ✓ | `.planning/REQUIREMENTS.md:41-47` |
| SETT-03 | ✓ | `.planning/REQUIREMENTS.md:41-47` |
| SETT-04 | ✓ | `.planning/REQUIREMENTS.md:41-47` |
| SETT-05 | ✓ | `.planning/REQUIREMENTS.md:41-47` |
| TERM-01 | ✓ | `.planning/REQUIREMENTS.md:22-27` |
| TERM-02 | ✓ | `.planning/REQUIREMENTS.md:22-27` |
| TERM-03 | ✓ | `.planning/REQUIREMENTS.md:22-27` |

## Human Verification Needed

1. **Settings Save/Cancel + toast**
   Otevři Settings > Projekt, přepni sandbox OFF, Cancel, ověř beze změny. Pak OFF + Save, ověř toast a zápis do `settings.toml`.
2. **Apply-on-reopen**
   Po uložení sandbox OFF znovu otevři projekt a ověř, že build/file tree/terminály používají OFF režim.
3. **Terminály cwd + label**
   Po reopen ověř `Terminal — <path>` v OFF režimu a `Sandbox` v ON režimu.

## Tooling

- `cargo check`: neprováděno
- `cargo test settings -- --nocapture`: neprováděno
- `./check.sh`: neprováděno

---
*Verifier: Codex (code inspection)*
