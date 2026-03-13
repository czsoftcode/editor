# M004: Klávesové Zkratky a Centrální Keymap — Research

**Date:** 2026-03-13

## Summary

Kódová báze má tři jasně oddělené problémy: (1) ad-hoc keyboard handling v `workspace/mod.rs` bez exkluzivního modifier filtrování — 6 handlerů používá `ctx.input(|i| i.modifiers.ctrl && i.key_pressed(...))`, což nekonzumuje event a nefiltruje extra modifikátory; (2) z 13 shortcut labelů v menu (Ctrl+F, H, G, P, Shift+F) pouze 4 mají funkční keyboard handler (Ctrl+S, Ctrl+W, Ctrl+B, Ctrl+R) — Ctrl+F/H/G nemají vůbec žádný handler, Ctrl+P a Ctrl+Shift+F mají handler jen přes menu klik; (3) command palette je implementovaná jako widget (`CommandPaletteState`) s render logikou a execute_command dispatch, ale `CommandPaletteState::new()` se nikde nevolá — palette je mrtvý kód, nedostupný z žádného místa.

Egui 0.31.1 nativně podporuje `KeyboardShortcut` struct, `consume_shortcut()` metodu na `InputState` (která konzumuje event a vrací bool), a `Modifiers::COMMAND` logický modifier (= Cmd na macOS, Ctrl na Linuxu). Klíčový detail: `consume_shortcut` používá `matches_logically()`, které **ignoruje** extra Shift a Alt modifikátory — to znamená, že `Ctrl+B` shortcut matchne i `Ctrl+Alt+B`. Egui to řeší pořadím: "match most specific shortcuts first" (jejich komentář). Stačí tedy řadit bindings od nejvíce modifikátorů po nejméně a používat `consume_shortcut` (ne `key_pressed`). Jediný existující handler, který to dělá správně, je `consume_close_tab_shortcut()` pro Ctrl+W — ten je vzorem pro všechny ostatní.

**Doporučení:** Vybudovat centrální `Keymap` struct, který drží seřazený Vec `(KeyboardShortcut, CommandId)` — od nejspecifičtějších po nejméně specifické. Dispatch voláme jednou na začátku frame přes `ctx.input_mut(|i| keymap.dispatch(i))`, vrátíme `Option<CommandId>`, a výsledek zpracujeme přes existující `execute_command()` → `MenuActions` → `process_menu_actions()` pipeline. Shortcut field v `Command` se změní z `Option<&'static str>` na parsovaný `Option<KeyboardShortcut>`. Uživatelské overrides se načtou z `[keybindings]` sekce settings.toml a přemapují defaults.

## Recommendation

### Přístup: Centrální dispatch napojený na existující MenuActions pipeline

Existující `execute_command(CommandId, &mut MenuActions)` + `process_menu_actions()` pipeline je plně funkční a pokrývá 17 příkazů. Nový keymap dispatch se napojí na tento flow — žádná duplikace logiky. Pořadí:

1. Parsování shortcut stringu → `KeyboardShortcut` (jednoduchý parser: "Ctrl+Shift+F" → `Modifiers::COMMAND | Modifiers::SHIFT`, `Key::F`)
2. `Keymap` struct se seřazeným Vec bindings (modifier count desc → stable sort)
3. Centrální dispatch: `ctx.input_mut(|i| keymap.consume_first_match(i))` → `Option<CommandId>`
4. Výsledek zpracovat přes `execute_command` + `process_menu_actions`
5. Rozšířit `CommandId` o chybějící příkazy (Find, Replace, GotoLine, CommandPalette)
6. Rozšířit `MenuActions` o chybějící flagy (find, replace, goto_line, command_palette)
7. Napojit flagy na editor stav (`show_search`, `show_replace`, `show_goto_line`)
8. Uživatelský override: `Settings.keybindings: HashMap<String, String>` → merge s defaults

### Co testovat prvně

Modifier matching a dispatch ordering — je to jádro problému. Trojkombinace `Ctrl+Alt+B` vs dvoukombinace `Ctrl+B` je regresní test #1.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Shortcut matching + event consumption | `egui::InputState::consume_shortcut()` | Nativní egui API; konzumuje event, takže double-match je nemožný. Vlastní matching by duplikoval logiku a riskoval edge cases. |
| Cross-platform Ctrl↔Cmd | `egui::Modifiers::COMMAND` | Logický modifier: Ctrl na Linuxu, Cmd na macOS. Egui to mapuje automaticky — ruční `cfg!(target_os)` je zbytečný. |
| Shortcut label formátování | `KeyboardShortcut::format()` + `ctx.os()` | Egui formátuje "Ctrl+S" na Linuxu, "⌘S" na macOS. Využít pro menu shortcut_text i command palette labels. |
| TOML parsing pro keybindings | `serde` + `toml` crate (už v dependencies) | `HashMap<String, String>` s `#[serde(default)]` — existující pattern v Settings. |

## Existing Code and Patterns

- `src/app/ui/workspace/mod.rs:508-539` — Ad-hoc keyboard handling: 6× `ctx.input()` handlerů bez exkluzivního modifier filtrování. **Nahradit** centrálním dispatch. Jediný správný handler je `consume_close_tab_shortcut()` — použít jako vzor.
- `src/app/ui/workspace/mod.rs` `consume_close_tab_shortcut()` — Vzorový pattern: `ctx.input_mut(|input| input.consume_shortcut(&KeyboardShortcut::new(...)))`. Centrální dispatch rozšíří tento pattern na všechny zkratky.
- `src/app/registry/mod.rs` — CommandRegistry s 17 příkazy. `Command.shortcut: Option<&'static str>` je čistý display label. Změnit na `Option<KeyboardShortcut>` pro dispatch; label odvodit z `KeyboardShortcut::format()`.
- `src/app/ui/widgets/command_palette.rs` — `execute_command(CommandId, &mut MenuActions)` mapuje CommandId → MenuActions flagy. Hotový dispatch — centrální keymap se na něj napojí. `CommandPaletteState` je plně implementovaný, ale nikdy se neotvírá (mrtvý kód).
- `src/app/ui/workspace/menubar/mod.rs` — `MenuActions` struct + `process_menu_actions()`: zpracovává 20 boolean flagů. Chybí: `find`, `replace`, `goto_line`, `command_palette`. Přidat.
- `src/app/ui/workspace/menubar/edit.rs` — 13 hardcoded `.shortcut_text("Ctrl+...")` labelů. Menu items pro Find/Replace/Goto Line nemají žádnou akci. Napojit na MenuActions + nahradit hardcoded labely dynamickými z keymap.
- `src/app/ui/editor/mod.rs` — Editor struct: `show_search: bool`, `show_replace: bool`, `show_goto_line: bool`. Existující stav — stačí je nastavit z process_menu_actions.
- `src/app/ui/editor/search.rs` — Search bar rendering. `show_search` se nastavuje na `false` (zavření), ale nikdy na `true` z klávesnice. Logika hledání je funkční.
- `src/app/ui/terminal/instance/input.rs` — `terminal_key_bytes()`: explicitně filtruje `ctrl && !shift && !alt` (řádek 5). Vzor pro správné modifier filtrování v terminálové vrstvě. Nesmí kolidovat s centrálním dispatch.
- `src/settings.rs` — Settings struct s TOML serde. `#[serde(default)]` pattern pro backwards compatibility. `[keybindings]` sekce bude nový `HashMap<String, String>` field.
- `src/app/ui/workspace/state/mod.rs` — `WorkspaceState.command_palette: Option<CommandPaletteState>`. Nastavením na `Some(CommandPaletteState::new(...))` se palette otevře.
- `src/app/types.rs` — `FocusedPanel` enum: `Build, Claude, Editor, Files`. Trojkombinace `Ctrl+Alt+E/B/A` přepínají focus.

## Constraints

- **egui `consume_shortcut` ignoruje extra modifikátory** — `Ctrl+B` matchne `Ctrl+Alt+B`. Dispatch MUSÍ řadit od nejspecifičtějšího. Testy musí pokrýt toto chování.
- **egui TextEdit built-in shortcuts** — TextEdit interně zpracovává Ctrl+A/C/V/X/Z/Y. Centrální dispatch nesmí konzumovat tyto eventy, jinak Copy/Paste/Undo přestanou fungovat v editoru. Řešení: tyto zkratky neregistrovat v keymapu.
- **Terminálový handler je nezávislý** — `terminal_key_bytes()` filtruje `ctrl && !shift && !alt` explicitně a posílá bajty do PTY. Centrální dispatch musí proběhnout PŘED terminal handlerem (nebo terminál musí být chráněný fokusem). Aktuální kód: terminálový input se zpracovává v terminal/instance rendereru — pokud terminál nemá fokus, nepřijímá klávesy. To by mělo fungovat.
- **settings.toml backwards compatibility** — `[keybindings]` sekce musí být `#[serde(default)]` — chybějící sekce = prázdný HashMap = default bindings. Existující settings.toml nesmí selhat při deserializaci.
- **Žádné nové runtime závislosti** — parser shortcut stringu musí být hand-rolled (jednoduchý: split '+', match name → Key/Modifiers).
- **17 existujících CommandId** — nové příkazy (Find, Replace, GotoLine, CommandPalette, ToggleSearch) vyžadují rozšíření enumu.
- **`cargo check` + `./check.sh` musí projít** po každé slice.

## Common Pitfalls

- **Dispatch ordering bug** — Pokud `Ctrl+B` handler proběhne před `Ctrl+Alt+B`, konzumuje event a trojkombinace nikdy nefunguje. Řešení: seřadit bindings v keymapu od nejvíce modifikátorů (3-key → 2-key → 1-key). Test: assert že `Ctrl+Alt+B` nespustí `Ctrl+B` akci.
- **Double dispatch** — Mix `key_pressed()` (nekonzumuje) a `consume_shortcut()` (konzumuje) = event se zpracuje dvakrát. Řešení: všechny zkratky přes `consume_shortcut`, žádný `key_pressed` pro dispatched shortcuts.
- **TextEdit intercepting shortcuts** — Pokud TextEdit (editor/search bar) má fokus a uživatel stiskne Ctrl+F, TextEdit může event zpracovat dřív než centrální dispatch (např. Ctrl+F v některých systémech = forward cursor). Řešení: centrální dispatch proběhne na začátku frame PŘED renderingem widgetů (jak je to teď) — `consume_shortcut` event odebere z fronty dříve než ho TextEdit vidí.
- **Keybinding conflict v user config** — Uživatel přiřadí stejnou zkratku dvěma příkazům. Řešení: první match wins + warning log. Detekce při parsování.
- **`Modifiers::COMMAND` vs `Modifiers::CTRL`** — Při definici shortcuts vždy používat `COMMAND` (= logický Ctrl/Cmd), ne `CTRL` (= fyzické Ctrl). `CTRL` na macOS = Control (ne Cmd), což by rozbilo UX.
- **Menu shortcut label desync** — Pokud uživatel přemapuje Ctrl+S na Ctrl+Shift+S, menu musí zobrazit "Ctrl+Shift+S", ne "Ctrl+S". Řešení: label generovat z keymap, ne hardcoded.

## Open Risks

- **Terminál vs centrální dispatch race** — Pokud terminál má fokus a uživatel stiskne Ctrl+B, má se spustit cargo build (centrální dispatch) nebo se má Ctrl+B poslat do terminálového PTY? Aktuální chování: Ctrl+B spustí cargo build VŽDY (handler v workspace/mod.rs je globální). To je pravděpodobně správné — terminál nemá vlastní handler pro Ctrl+B (ten jde přes `terminal_key_bytes` → PTY). Ověřit při implementaci.
- **Ctrl+, (settings) parsování** — Ověřeno: `egui::Key::Comma` existuje v 0.31.1, parsuje se z "," i "Comma". Riziko eliminováno.
- **Command palette focus management** — Otevření palette přes Ctrl+Shift+P musí převzít fokus z editoru/terminálu. Palette má `focus_requested: true` v `new()` — mělo by fungovat, ale netestováno.
- **egui zoom shortcuts** — egui 0.31 interně zpracovává Ctrl+= / Ctrl+- pro zoom. `gui_zoom.rs` používá `consume_shortcut`. Pokud bychom chtěli tyto zkratky pro jiný účel, museli bychom egui zoom deaktivovat.

## Requirements Analysis

### Table Stakes (musí být)
- **R010 (centrální dispatch)** — Jádro milestone. Bez toho nic dalšího nefunguje.
- **R011 (exkluzivní modifier matching)** — Kritický bug fix. Řešitelný automaticky přes dispatch ordering + `consume_shortcut`.
- **R012 (chybějící handlery)** — Primární UX problém. Rozšíření CommandId + napojení na editor stav.

### Expected (uživatel bude předpokládat)
- **R014 (cross-platform Ctrl↔Cmd)** — Automaticky řešeno přes `Modifiers::COMMAND`. Minimální effort.
- **R015 (VS Code/JetBrains konvence)** — Většina už je v menu labelech. Chybí: Ctrl+Shift+P (command palette), Ctrl+Tab (tab switch). Doplnit.

### Nice-to-have (konfigurovatelnost)
- **R013 (uživatelská konfigurace)** — Důležité, ale lze dodat jako poslední slice. `HashMap<String, String>` v settings.toml + merge s defaults.

### Candidate Requirements (z researche)
- **Žádné nové candidate requirements.** Scope v M004-CONTEXT pokrývá vše co research odhalil. Menu edit.rs Find/Replace/GotoLine nemají akci — to je podmnožinou R012.

### Out of Scope (potvrzeno)
- Vim/Emacs mode, multi-key sequences, keybinding UI editor — potvrzeno jako out of scope.
- Přemapování egui built-in shortcuts (Ctrl+A/C/V/X/Z/Y) — tyto patří TextEdit, centrální dispatch je nesmí interceptovat.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| egui / eframe | `bobmatnyc/claude-mpm-skills@rust-desktop-applications` | available (119 installs, ale příliš obecný — ne egui-specifický) |
| egui | (žádný egui-specifický skill) | none found |

## Sources

- egui 0.31.1 source: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/egui-0.31.1/` — `input_state/mod.rs` (consume_shortcut, consume_key, matches_logically), `data/input.rs` (Modifiers::COMMAND, KeyboardShortcut, cmd_ctrl_matches, ModifierNames::format), `context.rs` (os() detekce, format_keyboard_shortcut)
- Projekt source: `src/app/registry/mod.rs`, `src/app/ui/workspace/mod.rs`, `src/app/ui/widgets/command_palette.rs`, `src/app/ui/workspace/menubar/edit.rs`, `src/settings.rs`, `src/app/ui/terminal/instance/input.rs`
