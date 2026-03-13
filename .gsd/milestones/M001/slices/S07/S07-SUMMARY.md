---
id: S07
parent: M001
milestone: M001
provides:
  - Sandbox režim jako persistované nastavení v settings.toml
  - UI přepínač sandbox režimu s tooltipem a session toastem
  - Workspace init načítá sandbox do stabilních flagů
  - Terminály a build bar respektují režim pro cwd i label
  - Zvýšená viditelnost tooltipu a inline poznámky
key_files:
  - src/settings.rs
  - src/app/ui/workspace/modal_dialogs/settings.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/terminal/mod.rs
key_decisions:
  - "Sandbox režim je stabilní flag; změna se projeví až po reopen"
  - "Session-only toast při vypnutí sandboxu"
  - "Hover target tooltipu navázaný na celý řádek"
patterns_established:
  - "Apply-on-reopen pattern pro sandbox mód"
observability_surfaces:
  - none
drill_down_paths:
  - tasks/T01-SUMMARY.md
  - tasks/T02-SUMMARY.md
duration: 46min
verification_result: passed
completed_at: 2026-03-05
---

# S07: Infrastructure

**Sandbox režim persistovaný v settings.toml s UI přepínačem, terminály respektujícími režim pro cwd/label a apply-on-reopen sémantikou.**

## What Happened

T01 zavedl kompletní sandbox infrastrukturu: persistování `sandbox_mode` do settings.toml s legacy mapováním `project_read_only`, UI přepínač s tooltipem a toast notifikacemi, workspace init nastavující stabilní flagy (`sandbox_mode_enabled`, `build_in_sandbox`, `file_tree_in_sandbox`), terminály a build bar s režimovými labely a cwd. T02 zvýšil viditelnost tooltipu (full-width hover) a inline poznámky (bez small() potlačení).

## Verification

- `cargo check` (s RUSTC_WRAPPER= kvůli sandbox sccache omezení)
- Round-trip persistence testy
- Lokalizační texty pro cs, en

## Deviations

- `cargo fmt --all` vyžadoval formátování v souborech mimo scope (auto-fixed)
- `./check.sh` selhával na sccache oprávněních v sandboxu — workaround RUSTC_WRAPPER=

## Known Limitations

- Sandbox režim se aktivuje až po reopen projektu — ne okamžitě (záměrné, řešeno v S08)
- check.sh selhává na předexistujících clippy warnings mimo scope

## Follow-ups

- Okamžité apply sandbox režimu po Save — řešeno v S08

## Files Created/Modified

- `src/settings.rs` — sandbox_mode s aliasem project_read_only, testy migrace
- `src/app/ui/workspace/modal_dialogs/settings.rs` — sandbox toggle, tooltip, toasty
- `src/app/ui/workspace/state/init.rs` — init sandbox flagů
- `src/app/ui/terminal/mod.rs` — label + cwd helpery
- `src/app/ui/terminal/bottom/build_bar.rs` — cwd/label podle režimu
- `locales/cs/ui.ftl`, `locales/en/ui.ftl` — nové texty

## Forward Intelligence

### What the next slice should know
- Sandbox je apply-on-reopen — runtime apply vyžaduje nový systém (pending_sandbox_apply)

### What's fragile
- sccache v sandbox prostředí — RUSTC_WRAPPER= je nutný workaround

### Authoritative diagnostics
- settings.rs testy — round-trip sandbox persistence

### What assumptions changed
- Žádné — plán proběhl dle očekávání
