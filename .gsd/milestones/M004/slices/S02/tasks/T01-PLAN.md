---
estimated_steps: 9
estimated_files: 10
---

# T01: Rozšířit dispatch pipeline o Find/Replace/GotoLine/CommandPalette a oživit command palette

**Slice:** S02 — Chybějící keyboard handlery a oživení command palette
**Milestone:** M004

## Description

Rozšíření centrálního dispatch pipeline z S01 o 4 nové commandy (Find, Replace, GotoLine, CommandPalette), rozšíření `parse_shortcut()` pro standalone Fn klávesy (F1), napojení na editor stav a command palette, oprava menu edit.rs, a i18n pro všech 5 jazyků. Výsledek: Ctrl+F/H/G/Shift+P/F1 fungují jako keyboard zkratky, menu Find/Replace/GotoLine provádějí akci, command palette se otvírá a zobrazuje dynamické labely.

## Steps

1. **Rozšířit `CommandId` enum** — přidat `Find`, `Replace`, `GotoLine`, `CommandPalette` do `src/app/ui/widgets/command_palette.rs`.

2. **Rozšířit `parse_shortcut()` pro standalone klávesy** — v `src/app/keymap.rs` přidat whitelist: pokud klávesa je F1–F12, Escape, Delete, nebo Insert, modifikátor není povinný. Přidat unit test `test_parse_shortcut_f1`.

3. **Registrovat nové commandy v `init_defaults()`** — v `src/app/registry/mod.rs` přidat 5 nových záznamů: Find (Cmd+F), Replace (Cmd+H), GotoLine (Cmd+G), CommandPalette (Cmd+Shift+P), CommandPalette alternativní (F1 bez modifikátoru). Pro F1 alternativu: buď druhý Command se stejným CommandId ale jiným id stringem, nebo nový registrační mechanismus. Nejjednodušší: přidat druhý záznam `("ui.command_palette_f1", "command-name-command-palette", Some(KeyboardShortcut::new(Modifiers::NONE, Key::F1)), CommandPalette)`.

4. **Rozšířit `MenuActions` struct** — v `src/app/ui/workspace/menubar/mod.rs` přidat `pub find: bool`, `pub replace: bool`, `pub goto_line: bool`, `pub command_palette: bool`.

5. **Rozšířit `execute_command()` match** — v `src/app/ui/widgets/command_palette.rs` přidat `CommandId::Find => actions.find = true`, analogicky pro Replace, GotoLine, CommandPalette.

6. **Rozšířit `process_menu_actions()`** — v `src/app/ui/workspace/menubar/mod.rs`:
   - `actions.find`: `ws.editor.show_search = true; ws.editor.search_focus_requested = true;` (search bar se nezavírá, jen refocusne)
   - `actions.replace`: `ws.editor.show_search = true; ws.editor.show_replace = true; ws.editor.search_focus_requested = true;`
   - `actions.goto_line`: `ws.editor.show_goto_line = true; ws.editor.goto_line_focus_requested = true;`
   - `actions.command_palette`: toggle — `if ws.command_palette.is_some() { ws.command_palette = None; } else { ws.command_palette = Some(CommandPaletteState::new(commands_from_registry)); }`. Commands získat přes `shared.lock().registry.commands.get_all().to_vec()`.

7. **Opravit menu edit.rs** — Find/Replace/GotoLine kliknutí nastaví `actions.find/replace/goto_line = true` + `ui.close_menu()`. Shortcut labely přepsat na dynamické `shortcut_label(keymap, CommandId::Find)` atd. Přidat separator a menu položku pro Command Palette s `shortcut_label(keymap, CommandId::CommandPalette)`.

8. **i18n klíče** — přidat do všech 5 locale souborů (`locales/{cs,en,sk,de,ru}/ui.ftl`):
   - `command-name-find` — Find / Najít / Hľadať / Suchen / Найти
   - `command-name-replace` — Replace / Nahradit / Nahradiť / Ersetzen / Заменить
   - `command-name-goto-line` — Go to Line / Přejít na řádek / Prejsť na riadok / Gehe zu Zeile / Перейти к строке
   - `command-name-command-palette` — Command Palette / Příkazová paleta / Paleta príkazov / Befehlspalette / Палитра команд
   - `command-name-focus-editor` — Focus Editor / Fokus na editor / Fokus na editor / Editor fokussieren / Фокус на редактор
   - `command-name-focus-build` — Focus Build / Fokus na build / Fokus na build / Build fokussieren / Фокус на сборку
   - `command-name-focus-claude` — Focus AI / Fokus na AI / Fokus na AI / KI fokussieren / Фокус на ИИ

9. **Unit testy** — v `src/app/keymap.rs` přidat:
   - `test_parse_shortcut_f1`: `parse_shortcut("F1")` → Some s Key::F1 a Modifiers::NONE
   - `test_parse_shortcut_escape`: `parse_shortcut("Escape")` → Some (verifikace whitelistu)
   - `test_dispatch_new_commands`: keymap s Find(Cmd+F) → dispatch Ctrl+F vrátí Find
   - `test_dispatch_command_palette_ordering`: keymap s OpenFile(Cmd+P) a CommandPalette(Cmd+Shift+P) → dispatch Ctrl+Shift+P vrátí CommandPalette (ne OpenFile)

## Must-Haves

- [ ] CommandId má varianty Find, Replace, GotoLine, CommandPalette
- [ ] parse_shortcut("F1") vrátí Some (F1–F12 whitelist)
- [ ] init_defaults registruje Find/Replace/GotoLine/CommandPalette + F1 alternativu
- [ ] MenuActions má find/replace/goto_line/command_palette flagy
- [ ] execute_command mapuje nové CommandId → flagy
- [ ] process_menu_actions napojuje flagy na editor stav a command palette toggle
- [ ] Menu edit.rs: Find/Replace/GotoLine kliknutí nastaví flagy (ne jen close_menu)
- [ ] Menu edit.rs: dynamické shortcut labely z keymapu
- [ ] i18n: 7 command-name klíčů × 5 jazyků
- [ ] Unit testy: F1 parsing, nové commandy dispatch, Ctrl+Shift+P vs Ctrl+P ordering
- [ ] cargo check + ./check.sh pass

## Verification

- `cargo check` — kompilace čistá
- `./check.sh` — fmt + clippy + všechny testy pass
- `cargo test --bin polycredo-editor app::keymap` — nové i existující testy pass
- `grep -c "command-name-find" locales/en/ui.ftl` → ≥1
- `grep -c "command-name-focus-editor" locales/en/ui.ftl` → ≥1
- `grep -c "command-name-command-palette" locales/en/ui.ftl` → ≥1

## Inputs

- `src/app/keymap.rs` — S01 keymap modul (dispatch, parse_shortcut, format_shortcut, bindings řazení)
- `src/app/registry/mod.rs` — command registry s init_defaults() a helpery cmd()/cmd_shift()/cmd_alt()
- `src/app/ui/widgets/command_palette.rs` — CommandId enum, CommandPaletteState::new(), execute_command(), render_command_palette()
- `src/app/ui/workspace/menubar/mod.rs` — MenuActions struct, process_menu_actions(), shortcut_label() helper
- `src/app/ui/workspace/menubar/edit.rs` — menu items Find/Replace/GotoLine (aktuálně bez akce)
- `src/app/ui/editor/mod.rs` — Editor struct s show_search/show_replace/show_goto_line a focus request flagy
- S01 Forward Intelligence: Keymap::from_commands() automaticky seřadí nové bindings, execute_command() je centrální bod pro nové handlery

## Observability Impact

- **Nové diagnostické signály:** `parse_shortcut()` vrací `None` pro standalone klávesy mimo whitelist — pokrytý testem `test_parse_shortcut_invalid`. `Keymap::dispatch()` vrací `None` pokud shortcut nematchne žádný binding.
- **Inspekce command palette stavu:** `ws.command_palette.is_some()` — agent ověří toggle chování sledováním hodnoty.
- **Failure visibility:** Chybějící i18n klíč se zobrazí jako raw identifikátor v UI (např. "command-name-find") — vizuálně detekovatelné bez crashe.
- **Unit testy jako diagnostika:** 4 nové testy (`test_parse_shortcut_f1`, `test_parse_shortcut_escape`, `test_dispatch_new_commands`, `test_dispatch_command_palette_ordering`) slouží jako regression gate pro dispatch pipeline.

## Expected Output

- `src/app/ui/widgets/command_palette.rs` — 4 nové CommandId varianty, rozšířený execute_command match
- `src/app/registry/mod.rs` — 5 nových commandů v init_defaults (4 nové + F1 alternativa)
- `src/app/keymap.rs` — parse_shortcut whitelist pro Fn klávesy + 4 nové unit testy
- `src/app/ui/workspace/menubar/mod.rs` — 4 nové MenuActions flagy + process_menu_actions handling
- `src/app/ui/workspace/menubar/edit.rs` — menu napojení na flagy + dynamické shortcut labely + command palette menu item
- `locales/{cs,en,sk,de,ru}/ui.ftl` — 7 nových command-name klíčů v každém
