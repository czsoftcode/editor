# S02: Chybějící keyboard handlery a oživení command palette

**Goal:** Ctrl+F/H/G/Shift+P fungují jako keyboard zkratky, menu Find/Replace/GotoLine skutečně otvírají příslušné UI, command palette se otvírá přes Ctrl+Shift+P (nebo F1) a zobrazuje aktuální keybinding labely.

**Demo:** Ctrl+F otevře search bar s fokusem, Ctrl+H otevře search+replace, Ctrl+G otevře goto line, Ctrl+Shift+P otevře command palette, F1 taktéž. Palette zobrazuje dynamické shortcut labely z keymapu. Menu Edit → Find/Replace/GotoLine kliknutí funguje (ne jen zavření menu).

## Must-Haves

- `CommandId` rozšířen o `Find`, `Replace`, `GotoLine`, `CommandPalette`
- Nové commandy registrovány v `init_defaults()` se správnými shortcuty (COMMAND+F, COMMAND+H, COMMAND+G, COMMAND|SHIFT+P)
- `MenuActions` rozšířen o `find`, `replace`, `goto_line`, `command_palette` flagy
- `execute_command()` mapuje nové CommandId → MenuActions flagy
- `process_menu_actions()` napojuje flagy na editor stav (`show_search/replace/goto_line`, focus requesty) a command palette toggle
- Menu edit.rs: Find/Replace/GotoLine kliknutí nastaví příslušné `actions.*` flagy (ne jen `ui.close_menu()`)
- Menu edit.rs: dynamické shortcut labely z keymapu (ne hardcoded "Ctrl+F" stringy)
- `parse_shortcut()` rozšířen o whitelist standalone kláves (F1–F12, Escape, Delete, Insert) bez povinného modifikátoru
- F1 registrován jako alternativní binding pro CommandPalette
- Command palette toggle chování: Ctrl+Shift+P při otevřené palette ji zavře
- Ctrl+F při otevřeném search baru refocusne input (ne toggle off)
- i18n: command-name klíče pro Find, Replace, GotoLine, CommandPalette + FocusEditor/FocusBuild/FocusClaude (chybí ze S01) ve všech 5 jazycích
- `cargo check` + `./check.sh` projde čistě

## Verification

- `cargo test --bin polycredo-editor app::keymap` — existující 9 testů + nové testy pro F1 parsing a nové commandy
- `./check.sh` — fmt + clippy + všechny testy pass
- `grep -c "command-name-find" locales/en/ui.ftl` vrátí ≥1
- `grep -c "command-name-focus-editor" locales/en/ui.ftl` vrátí ≥1
- Nový unit test: `test_parse_shortcut_f1` — `parse_shortcut("F1")` vrátí `Some(KeyboardShortcut)` s `Modifiers::NONE` a `Key::F1`
- Nový unit test: `test_dispatch_find_replace_gotoLine` — dispatch pro Ctrl+F vrátí `CommandId::Find`
- Nový unit test: `test_dispatch_command_palette_vs_open_file` — Ctrl+Shift+P vrátí CommandPalette (ne OpenFile)
- Diagnostický test: `test_parse_shortcut_invalid` — ověřuje, že `parse_shortcut("S")` (standalone písmeno bez modifikátoru) vrátí `None` (failure path visibility)

## Observability / Diagnostics

- **Dispatch miss diagnostika:** `Keymap::dispatch()` vrací `None` pokud žádný binding nematchuje — agent může grep logiku pro "dispatch vrátil None" při debugování nefunkčních zkratek.
- **Command palette toggle stav:** `ws.command_palette.is_some()` je přímo inspektovatelný — agent zkontroluje, zda palette je otevřená/zavřená po toggle akci.
- **parse_shortcut diagnostika:** Vrací `None` pro nevalidní vstupy — unit testy pokrývají edge cases (F1 whitelist, chybějící modifikátor, neznámá klávesa).
- **i18n klíče:** Chybějící klíč v Fluent se projeví zobrazením raw klíče (např. "command-name-find") v UI — vizuálně detekovatelné.
- **Menu flagy:** Nepropojený flag v MenuActions znamená, že menu kliknutí nemá efekt — detekovatelné funkčním testem (klik → žádná reakce).
- **Redakce:** Žádné citlivé údaje v tomto slice.

## Integration Closure

- Upstream surfaces consumed: `src/app/keymap.rs` (Keymap, parse_shortcut, format_shortcut), `src/app/registry/mod.rs` (init_defaults, Command), `src/app/ui/workspace/menubar/mod.rs` (MenuActions, process_menu_actions, shortcut_label)
- New wiring introduced in this slice: 4 nové CommandId → MenuActions → process_menu_actions cesty, F1 alternativní binding, command palette otevření/toggle
- What remains before the milestone is truly usable end-to-end: S03 (uživatelská konfigurace keybindings v settings.toml + dynamické labely z overrides)

## Tasks

- [x] **T01: Rozšířit dispatch pipeline o Find/Replace/GotoLine/CommandPalette a oživit command palette** `est:45m`
  - Why: Jediný koherentní task — všechny kroky rozšiřují jeden pipeline (CommandId → registry → MenuActions → process_menu_actions → menu/palette), žádná část nezávisí na mezivýstupu jiné.
  - Files: `src/app/ui/widgets/command_palette.rs`, `src/app/registry/mod.rs`, `src/app/ui/workspace/menubar/mod.rs`, `src/app/ui/workspace/menubar/edit.rs`, `src/app/keymap.rs`, `locales/{cs,en,sk,de,ru}/ui.ftl`
  - Do: (1) Rozšířit `CommandId` enum o `Find`, `Replace`, `GotoLine`, `CommandPalette`. (2) Registrovat 5 nových commandů v `init_defaults()` (Find=Cmd+F, Replace=Cmd+H, GotoLine=Cmd+G, CommandPalette=Cmd+Shift+P, CommandPalette alternativní=F1). (3) Rozšířit `parse_shortcut()` — whitelist F1–F12/Escape/Delete/Insert bez modifikátoru. (4) Přidat `find`, `replace`, `goto_line`, `command_palette` flagy do `MenuActions`. (5) Rozšířit `execute_command()` match o nové varianty. (6) Rozšířit `process_menu_actions()` — Find: `show_search=true, search_focus_requested=true` (pokud už otevřen, jen refocus). Replace: `show_search=true, show_replace=true, search_focus_requested=true`. GotoLine: `show_goto_line=true, goto_line_focus_requested=true`. CommandPalette: toggle — `is_some()` → None, jinak `Some(new())`. (7) Opravit menu edit.rs: Find/Replace/GotoLine kliknutí nastaví flagy + dynamické shortcut labely z keymapu. (8) i18n: 7 command-name klíčů × 5 jazyků. (9) Unit testy: F1 parsing, nové commandy dispatch, Ctrl+Shift+P vs Ctrl+P ordering.
  - Verify: `cargo check` + `./check.sh` pass, nové unit testy pass, `grep` potvrdí i18n klíče
  - Done when: Ctrl+F/H/G/Shift+P/F1 dispatchují správné CommandId, menu kliknutí funguje, command palette se otvírá/zavírá, i18n kompletní, všechny testy pass

## Files Likely Touched

- `src/app/ui/widgets/command_palette.rs` — CommandId enum + execute_command match
- `src/app/registry/mod.rs` — init_defaults (nové commandy)
- `src/app/keymap.rs` — parse_shortcut whitelist + nové unit testy
- `src/app/ui/workspace/menubar/mod.rs` — MenuActions struct + process_menu_actions
- `src/app/ui/workspace/menubar/edit.rs` — menu napojení + dynamické labely
- `locales/cs/ui.ftl` — i18n klíče
- `locales/en/ui.ftl` — i18n klíče
- `locales/sk/ui.ftl` — i18n klíče
- `locales/de/ui.ftl` — i18n klíče
- `locales/ru/ui.ftl` — i18n klíče
