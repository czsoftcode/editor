# S02: Chybějící keyboard handlery a oživení command palette — Research

**Date:** 2026-03-13

## Summary

Kódová báze po S01 má funkční centrální keymap dispatch (`Keymap::dispatch()` v `render_workspace()` PŘED widget renderingem), ale z 13 shortcut labelů v menu pouze 7 má funkční keyboard handler (Save, CloseTab, Build, Run, OpenFile, ProjectSearch, Settings + 3 focus trojkombinace). Chybí: **Find (Ctrl+F), Replace (Ctrl+H), GotoLine (Ctrl+G), CommandPalette (Ctrl+Shift+P)**. Editor má kompletní UI pro search bar (`search.rs`), replace bar (toggle v search baru), a goto line bar (`goto_line_bar()` v `render/tabs.rs`), ale žádný z nich se nedá aktivovat z klávesnice ani z menu — menu klik jen zavře menu bez nastavení jakéhokoliv flagu. Command palette widget (`CommandPaletteState`) je plně implementovaný s rendering, filtrem, execute_command dispatch, ale `CommandPaletteState::new()` se nikde nevolá — mrtvý kód.

Přístup je přímočarý díky S01 infrastruktuře: rozšířit `CommandId` o 4 nové varianty (Find, Replace, GotoLine, CommandPalette), přidat odpovídající commandy do `init_defaults()` s KeyboardShortcut, rozšířit `MenuActions` o 4 nové flagy, napojit v `process_menu_actions()` na editor stav (`ws.editor.show_search/show_replace/show_goto_line` + focus request flagy), a pro command palette nastavit `ws.command_palette = Some(CommandPaletteState::new(...))`. Menu edit.rs potřebuje napojit Find/Replace/GotoLine kliknutí na nové MenuActions flagy. I18n klíče pro 7 nových command-name + 3 chybějící focus klíče ze S01 = 10 klíčů × 5 jazyků.

**Klíčový finding:** egui TextEdit interně zpracovává `Ctrl+H` jako "delete previous char" (backspace). Centrální dispatch konzumuje event PŘED TextEdit, takže Ctrl+H = Replace (ne delete). To je správné chování odpovídající VS Code konvenci. Ctrl+F a Ctrl+G nemají v TextEdit žádný handler.

## Recommendation

### Přístup: Rozšíření existujícího dispatch pipeline

S01 vybudoval kompletní pipeline: `Keymap::dispatch()` → `execute_command()` → `MenuActions` → `process_menu_actions()`. S02 jen rozšíří každý článek o nové commandy. Žádná nová architektura.

**Kroky:**

1. **Rozšířit `CommandId`** — přidat `Find`, `Replace`, `GotoLine`, `CommandPalette` varianty
2. **Registrovat nové commandy** v `init_defaults()` s shortcuty:
   - `Find` → `Ctrl+F` (COMMAND + F)
   - `Replace` → `Ctrl+H` (COMMAND + H)
   - `GotoLine` → `Ctrl+G` (COMMAND + G)
   - `CommandPalette` → `Ctrl+Shift+P` (COMMAND | SHIFT + P)
3. **Rozšířit `MenuActions`** — přidat `find`, `replace`, `goto_line`, `command_palette` flagy
4. **Rozšířit `execute_command()`** — mapovat nové CommandId → MenuActions flagy
5. **Rozšířit `process_menu_actions()`** — napojit flagy na editor stav a command palette
6. **Opravit menu edit.rs** — Find/Replace/GotoLine kliknutí nastaví `actions.find/replace/goto_line = true` + dynamické labely z keymapu
7. **i18n klíče** — 10 nových command-name klíčů × 5 jazyků (cs, en, sk, de, ru)

### Dispatch ordering — Ctrl+P vs Ctrl+Shift+P

`Ctrl+Shift+P` (CommandPalette) má 2 modifikátory (COMMAND + SHIFT), `Ctrl+P` (OpenFile) má 1 modifikátor (COMMAND). Díky S01 seřazení bindings od nejspecifičtějších, Ctrl+Shift+P matchne jako CommandPalette (ne OpenFile). Žádný konflikt.

### Command palette toggle vs open

Command palette by měla být toggle — pokud je otevřená, Ctrl+Shift+P ji zavře (VS Code chování). Implementace: v `process_menu_actions`, pokud `actions.command_palette` a `ws.command_palette.is_some()`, nastavit `ws.command_palette = None`. Jinak otevřít.

### F1 jako alternativa pro command palette

Roadmap zmiňuje "Ctrl+Shift+P (nebo F1)". `parse_shortcut()` vyžaduje aspoň jeden modifikátor — F1 bez modifikátoru by selhalo. Dvě možnosti:
1. Rozšířit `parse_shortcut()` aby akceptoval Fn klávesy bez modifikátoru
2. Přidat F1 jako druhý binding mimo parser

**Doporučení:** Rozšířit parser — Fn klávesy (F1–F12) jako standalone zkratky jsou legitimní a budou potřeba i v budoucnu. Přidat výjimku: pokud klávesa je F1–F12, Escape, Delete, nebo Insert, modifikátor není povinný.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Dispatch nových commandů | `execute_command()` → `MenuActions` → `process_menu_actions()` pipeline | Existující pipeline z S01 — stačí rozšířit o nové varianty. Nový pipeline by duplikoval logiku. |
| Shortcut label pro menu | `shortcut_label(keymap, CommandId)` helper | Z S01 — lookup v keymap bindings + `format_shortcut()`. |
| Shortcut label v palette | `format_shortcut(&shortcut)` | Z S01 — palette už to používá pro existující commandy. |
| Command palette rendering | `render_command_palette()` + `CommandPaletteState` | Plně implementovaný widget — jen chybí `new()` volání pro otevření. |
| Search/Replace/GotoLine UI | Editor `search_bar()` + `goto_line_bar()` | Plně funkční UI — jen chybí nastavení flagů z keyboard handleru. |

## Existing Code and Patterns

- `src/app/keymap.rs` — **Použít jako je.** Keymap, dispatch, parse_shortcut, format_shortcut. Rozšíření parse_shortcut pro Fn klávesy bez modifikátoru.
- `src/app/registry/mod.rs:157-281` — `init_defaults()` s helpers `cmd()`, `cmd_shift()`, `cmd_alt()`. **Přidat nové commandy** do `defaults` Vec.
- `src/app/ui/widgets/command_palette.rs:8-29` — `CommandId` enum. **Přidat** Find, Replace, GotoLine, CommandPalette.
- `src/app/ui/widgets/command_palette.rs:193-225` — `execute_command()` match. **Přidat** nové varianty → `actions.find/replace/goto_line/command_palette`.
- `src/app/ui/widgets/command_palette.rs:40-49` — `CommandPaletteState::new()`. **Volat** z `process_menu_actions()` pro otevření palette.
- `src/app/ui/workspace/menubar/mod.rs:42-66` — `MenuActions` struct. **Přidat** `find`, `replace`, `goto_line`, `command_palette` flagy.
- `src/app/ui/workspace/menubar/mod.rs:94-250` — `process_menu_actions()`. **Přidat** handling pro nové flagy — nastavit editor stav.
- `src/app/ui/workspace/menubar/edit.rs:31-48` — Find/Replace/GotoLine menu items. **Napojit** na `actions.find/replace/goto_line` + dynamické labely z keymapu.
- `src/app/ui/editor/mod.rs:140-155` — Editor flagy `show_search`, `show_replace`, `show_goto_line`, `search_focus_requested`, `goto_line_focus_requested`. **Nastavit** z `process_menu_actions`.
- `src/app/ui/workspace/state/mod.rs:100` — `command_palette: Option<CommandPaletteState>`. **Nastavit** na `Some(new())` pro otevření.
- `locales/en/ui.ftl` — Vzor pro i18n klíče. **Přidat** command-name-find, command-name-replace, command-name-goto-line, command-name-command-palette + 3 focus klíče ze S01.

## Constraints

- **Dispatch musí být PŘED widget renderingem** — consume_shortcut konzumuje event z fronty. Aktuální pozice (řádek 500 workspace/mod.rs) je správná a S02 ji nemění.
- **egui TextEdit Ctrl+H = delete previous char** — dispatch konzumuje Ctrl+H dřív než TextEdit → Ctrl+H = Replace. Správné chování.
- **Ctrl+A/C/V/X/Z/Y nesmí být v keymapu** — S01 decision, S02 ho nemění.
- **Terminálový handler je nezávislý** — S02 nepřidává zkratky kolidující s terminálovou vrstvou.
- **parse_shortcut vyžaduje modifikátor** — pro F1 nutno rozšířit parser (whitelist Fn/Escape/Delete/Insert).
- **command_palette otevření potřebuje registry commands** — `CommandPaletteState::new(commands)` potřebuje `Vec<Command>` z `shared.lock().registry.commands.get_all().to_vec()`. Přístup k `shared` v `process_menu_actions` je dostupný (parametr).
- **`cargo check` + `./check.sh` musí projít** — 156 testů baseline.
- **i18n: 5 jazyků** — cs, en, sk, de, ru. Všechny nové command-name klíče musí být ve všech.

## Common Pitfalls

- **Menu edit Find/Replace/GotoLine nedělají nic** — kliknutí jen zavírá menu bez nastavení flagu. Oprava: přidat `actions.find = true` / `actions.replace = true` / `actions.goto_line = true` a nezapomenout na to, ne jen na keyboard handler.
- **Command palette toggle race** — pokud palette je `Some` a dispatch nastaví `command_palette = true`, `process_menu_actions` musí check `is_some()` a toggle (ne open duplicitně). Jinak by se palette "znovuotevřela" (reset query/selection).
- **Search bar focus** — po otevření search baru přes Ctrl+F musí search input dostat focus (`search_focus_requested = true`). Bez toho uživatel musí kliknout do inputu — špatný UX.
- **Goto line focus** — stejný pattern: `goto_line_focus_requested = true`.
- **Replace toggle vs open** — Ctrl+H by měl otevřít search bar S replace barem (ne jen replace bar). Nastavit `show_search = true` + `show_replace = true` + `search_focus_requested = true`.
- **Keymap bindings řazení** — nové commandy přidané přes `init_defaults()` → `Keymap::from_commands()` automaticky seřadí. Žádný ruční push.
- **Test fixtures** — unit testy v `mod.rs` a `unsaved_close_guard.rs` konstruují `WorkspaceState` s literálem — nové `MenuActions` flagy změní Default derive, ale to by mělo být OK (Default doplní false).

## Open Risks

- **Search bar a dispatch koexistence** — Když je search bar otevřený a uživatel píše do search inputu, centrální dispatch by neměl interceptovat typing. To je OK — dispatch konzumuje jen shortcuty (Ctrl+X kombinace), ne samotné písmena. Ale: **Ctrl+F znovu** při otevřeném search baru — měl by toggle? VS Code: Ctrl+F při otevřeném search baru refocusne input. Implementace: pokud `show_search` je true, jen nastavit `search_focus_requested = true` (ne toggle off).
- **i18n kvalita** — překlady pro de/ru/sk jsou odhadované, nemám native speakera. Ale to je existující pattern — celá i18n je přibližná.
- **Command palette a záběr commandů** — palette zobrazí VŠECHNY commandy z registry, včetně nových Find/Replace/GotoLine. To je správné — VS Code dělá totéž.
- **Missing i18n klíče ze S01** — `command-name-focus-editor/build/claude` neexistují. S02 je přidá. Pokud uživatel mezitím otevřel palette, tyto commandy by zobrazily surový klíč. Nízký impact — palette se nedala otevřít.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| egui / eframe | (žádný egui-specifický skill) | none found |

## Sources

- `src/app/keymap.rs` — S01 keymap modul: dispatch, parse_shortcut, format_shortcut
- `src/app/registry/mod.rs` — command registry s `init_defaults()` a 20 existujícími commandy
- `src/app/ui/widgets/command_palette.rs` — CommandId enum (20 variant), CommandPaletteState (mrtvý kód), execute_command dispatch, render_command_palette
- `src/app/ui/workspace/menubar/mod.rs` — MenuActions struct (23 flagů), process_menu_actions, shortcut_label helper
- `src/app/ui/workspace/menubar/edit.rs` — menu items Find/Replace/GotoLine bez akce
- `src/app/ui/editor/mod.rs` — Editor struct s show_search/show_replace/show_goto_line flagy
- `src/app/ui/editor/search.rs` — search_bar() + replace bar (plně funkční UI)
- `src/app/ui/editor/render/tabs.rs` — goto_line_bar() (plně funkční UI)
- `src/app/ui/workspace/mod.rs:498-508` — centrální keymap dispatch (S01)
- `~/.cargo/registry/src/*/egui-0.31.1/src/widgets/text_edit/builder.rs` — TextEdit Ctrl+H = delete previous char
- `locales/en/ui.ftl` — existujících 20 command-name i18n klíčů
