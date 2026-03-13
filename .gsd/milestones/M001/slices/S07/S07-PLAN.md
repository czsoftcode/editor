# S07: Infrastructure

**Goal:** Zavést sandbox režim do Settings (persistovaný do settings.
**Demo:** Zavést sandbox režim do Settings (persistovaný do settings.

## Must-Haves


## Tasks

- [x] **T01: 04-infrastructure 01** `est:45min`
  - Zavést sandbox režim do Settings (persistovaný do settings.toml), přidat přepínač v Settings > Projekt s tooltipem a inline poznámkou o reopen, a při startu projektu nastavovat build_in_sandbox / file_tree_in_sandbox podle uloženého režimu. Terminály musí respektovat režim pro cwd i label.
- [x] **T02: 04-infrastructure 02** `est:1min`
  - Zvýšit viditelnost tooltipu a inline poznámky u sandbox režimu tak, aby uživatel informaci nepřehlédl.

## Files Likely Touched

- `src/settings.rs`
- `src/app/ui/background.rs`
- `src/app/ui/workspace/mod.rs`
- `src/app/ui/editor/files.rs`
- `src/app/ui/editor/ui.rs`
- `src/app/ui/workspace/menubar/mod.rs`
- `src/app/ui/workspace/modal_dialogs/conflict.rs`
- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `src/app/ui/workspace/state/init.rs`
- `src/app/ui/workspace/state/mod.rs`
- `src/app/mod.rs`
- `src/app/ui/terminal/mod.rs`
- `src/app/ui/terminal/instance/mod.rs`
- `src/app/ui/terminal/bottom/build_bar.rs`
- `locales/cs/ui.ftl`
- `locales/en/ui.ftl`
- `src/app/ui/workspace/modal_dialogs/settings.rs`
- `locales/cs/ui.ftl`
- `locales/en/ui.ftl`
