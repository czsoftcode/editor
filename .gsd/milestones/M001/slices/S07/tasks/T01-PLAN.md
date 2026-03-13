# T01: 04-infrastructure 01

**Slice:** S07 — **Milestone:** M001

## Description

Zavést sandbox režim do Settings (persistovaný do settings.toml), přidat přepínač v Settings > Projekt s tooltipem a inline poznámkou o reopen, a při startu projektu nastavovat build_in_sandbox / file_tree_in_sandbox podle uloženého režimu. Terminály musí respektovat režim pro cwd i label.

## Must-Haves

- [ ] "Settings obsahují nový boolean pro sandbox režim a je persistován do settings.toml."
- [ ] "Přepínač v Settings > Projekt respektuje Save/Cancel semantiku a změna se projeví až po znovuotevření projektu."
- [ ] "Při startu projektu se build_in_sandbox a file_tree_in_sandbox nastaví z uloženého sandbox režimu."
- [ ] "Terminály používají správný cwd podle sandbox režimu a label odpovídá (ON: Sandbox, OFF: Terminal + cesta)."
- [ ] "Runtime UI nečte `settings.sandbox_mode` přímo; změna se uplatní jen přes init (apply on reopen)."

## Files

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
