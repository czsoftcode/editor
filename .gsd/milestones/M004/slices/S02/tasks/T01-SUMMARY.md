---
id: T01
parent: S02
milestone: M004
provides:
  - 4 nové CommandId varianty (Find, Replace, GotoLine, CommandPalette) v dispatch pipeline
  - 5 nových command registrací v init_defaults() včetně F1 alternativní binding
  - parse_shortcut() whitelist pro standalone Fn/Escape/Delete/Insert klávesy
  - MenuActions flagy + process_menu_actions napojení na editor stav a command palette toggle
  - Menu edit.rs napojení na flagy + dynamické shortcut labely + Command Palette menu položka
  - 7 command-name i18n klíčů × 5 jazyků (cs, en, sk, de, ru)
  - 4 nové unit testy pro F1 parsing, Escape parsing, nové commandy dispatch, Ctrl+Shift+P ordering
key_files:
  - src/app/ui/widgets/command_palette.rs
  - src/app/registry/mod.rs
  - src/app/keymap.rs
  - src/app/ui/workspace/menubar/mod.rs
  - src/app/ui/workspace/menubar/edit.rs
  - locales/{cs,en,sk,de,ru}/ui.ftl
  - locales/{cs,en,sk,de,ru}/menu.ftl
key_decisions:
  - F1 alternativní binding registrován jako samostatný command záznam se stejným CommandId::CommandPalette ale jiným id stringem ("ui.command_palette_f1") — nejjednodušší přístup bez nového registračního mechanismu
  - is_standalone_key_allowed() jako privátní helper pro whitelist F1-F12/Escape/Delete/Insert — nemodifikuje existující Modifiers validaci, jen přidává výjimku
  - Command palette toggle chování implementováno v process_menu_actions, ne v execute_command — execute_command jen nastaví flag, process_menu_actions řeší stav
patterns_established:
  - Standalone klávesy (F1-F12, Escape, Delete, Insert) v parse_shortcut() whitelist — nové shortcuty bez modifikátoru přidávat do is_standalone_key_allowed()
  - Menu položky s flagy: kliknutí nastaví actions.* = true + ui.close_menu(), shortcut label přes shortcut_label(keymap, CommandId::*)
observability_surfaces:
  - parse_shortcut() vrací None pro nevalidní vstupy — pokrytý unit testy
  - Keymap::dispatch() vrací None pokud shortcut nematchne — diagnostický signál
  - ws.command_palette.is_some() — inspektovatelný stav command palette
  - Chybějící i18n klíč se zobrazí jako raw identifikátor v UI
duration: 30m
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: Rozšířit dispatch pipeline o Find/Replace/GotoLine/CommandPalette a oživit command palette

**Rozšíření centrálního dispatch pipeline o 4 nové commandy, oživení command palette toggle, F1 alternativní binding, dynamické menu shortcut labely a kompletní i18n pro 7 command-name klíčů × 5 jazyků.**

## What Happened

1. **CommandId enum** — přidány varianty `Find`, `Replace`, `GotoLine`, `CommandPalette` do `command_palette.rs`.

2. **parse_shortcut() whitelist** — přidána funkce `is_standalone_key_allowed()` pro F1–F12, Escape, Delete, Insert. Podmínka "musí mít modifikátor" nyní obchází tyto klávesy.

3. **init_defaults()** — registrováno 5 nových commandů: Find (Cmd+F), Replace (Cmd+H), GotoLine (Cmd+G), CommandPalette (Cmd+Shift+P), CommandPalette alternativní (F1 bez modifikátoru). F1 binding má id "ui.command_palette_f1" se sdíleným i18n klíčem.

4. **MenuActions** — přidány `find`, `replace`, `goto_line`, `command_palette` flagy.

5. **execute_command()** — rozšířen match o 4 nové varianty mapující na příslušné MenuActions flagy.

6. **process_menu_actions()** — implementováno: Find → show_search + search_focus_requested. Replace → show_search + show_replace + search_focus_requested. GotoLine → show_goto_line + goto_line_focus_requested. CommandPalette → toggle (is_some → None, jinak nový CommandPaletteState z registry).

7. **Menu edit.rs** — Find/Replace/GotoLine kliknutí nastavují flagy (ne jen close_menu). Shortcut labely přepsány na dynamické `shortcut_label(keymap, CommandId::*)`. Přidán separator a Command Palette menu položka.

8. **i18n** — 7 nových `command-name-*` klíčů v ui.ftl × 5 jazyků + `menu-edit-command-palette` v menu.ftl × 5 jazyků.

9. **Unit testy** — 4 nové: `test_parse_shortcut_f1`, `test_parse_shortcut_escape`, `test_dispatch_new_commands`, `test_dispatch_command_palette_ordering`.

## Verification

- `cargo check` — kompilace čistá ✅
- `./check.sh` — fmt + clippy + všechny testy pass ✅
- `cargo test --bin polycredo-editor app::keymap` — 13/13 testů prošlo (9 existujících + 4 nové) ✅
- `grep -c "command-name-find" locales/en/ui.ftl` → 1 ✅
- `grep -c "command-name-focus-editor" locales/en/ui.ftl` → 1 ✅
- `grep -c "command-name-command-palette" locales/en/ui.ftl` → 1 ✅
- Diagnostický test `test_parse_shortcut_invalid` — ověřuje, že standalone písmeno "S" vrátí None ✅ (existující test)

## Diagnostics

- `parse_shortcut("F1")` vrací `Some` — diagnostický test `test_parse_shortcut_f1`
- `parse_shortcut("S")` vrací `None` — diagnostický test `test_parse_shortcut_invalid`
- `Keymap::dispatch()` vrací `None` pro nematchované shortcuty — pokrytý testem `test_empty_keymap_dispatch`
- `ws.command_palette.is_some()` — runtime inspekce command palette toggle stavu
- Chybějící i18n klíč se projeví zobrazením raw identifikátoru v UI

## Deviations

- Přidán i18n klíč `menu-edit-command-palette` do `menu.ftl` (ne `ui.ftl`) pro menu položku — konzistentní s existujícím vzorem, kde menu labely žijí v `menu.ftl`.

## Known Issues

- None

## Files Created/Modified

- `src/app/ui/widgets/command_palette.rs` — 4 nové CommandId varianty + 4 nové execute_command match větve
- `src/app/registry/mod.rs` — 5 nových command registrací v init_defaults()
- `src/app/keymap.rs` — is_standalone_key_allowed() helper + parse_shortcut whitelist + 4 nové unit testy
- `src/app/ui/workspace/menubar/mod.rs` — 4 nové MenuActions flagy + 4 nové process_menu_actions handlery (find/replace/goto_line/command_palette)
- `src/app/ui/workspace/menubar/edit.rs` — menu napojení na flagy + dynamické shortcut labely + Command Palette menu položka
- `locales/en/ui.ftl` — 7 nových command-name klíčů
- `locales/cs/ui.ftl` — 7 nových command-name klíčů
- `locales/sk/ui.ftl` — 7 nových command-name klíčů
- `locales/de/ui.ftl` — 7 nových command-name klíčů
- `locales/ru/ui.ftl` — 7 nových command-name klíčů
- `locales/{cs,en,sk,de,ru}/menu.ftl` — menu-edit-command-palette klíč
- `.gsd/milestones/M004/slices/S02/S02-PLAN.md` — přidána Observability/Diagnostics sekce + diagnostický verifikační krok
- `.gsd/milestones/M004/slices/S02/tasks/T01-PLAN.md` — přidána Observability Impact sekce
