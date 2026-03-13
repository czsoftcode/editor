# S01: Centrální keymap dispatch a oprava modifier filtrování

**Goal:** Centrální `Keymap` dispatch nahrazuje ad-hoc keyboard handlery. Modifier filtrování je exkluzivní — trojkombinace nespouštějí dvoukombinace.
**Demo:** Ctrl+Alt+B přepne fokus na build panel BEZ spuštění cargo build. Ctrl+B spustí build. Všechny existující zkratky (Ctrl+S, Ctrl+W, Ctrl+B, Ctrl+R, Ctrl+Alt+E/B/A, Ctrl+,) fungují přes centrální dispatch. Cross-platform Cmd/Ctrl automaticky přes Modifiers::COMMAND.

## Must-Haves

- `Keymap` struct s dispatch seřazeným od nejvíce modifikátorů po nejméně — egui `consume_shortcut` konzumuje event a zabrání falešným matchům
- `parse_shortcut("Ctrl+Alt+B") -> KeyboardShortcut` parser pro string→shortcut konverzi (základ pro S03 konfiguraci)
- `format_shortcut(KeyboardShortcut) -> String` pro generování menu/palette labelů z dat (ne hardcoded stringy)
- `Command.shortcut` změněn z `Option<&'static str>` na `Option<KeyboardShortcut>`
- Ad-hoc keyboard handlery ve `workspace/mod.rs` (řádky 509-539) nahrazeny centrálním dispatch přes `Keymap`
- Nové `CommandId` varianty: `FocusEditor`, `FocusBuild`, `FocusClaude` pro trojkombinace
- Všechny zkratky přes `Modifiers::COMMAND` (ne `Modifiers::CTRL`) — Cmd na macOS, Ctrl na Linuxu
- `Ctrl+A/C/V/X/Z/Y` neregistrovány v keymapu — TextEdit je zpracuje sám
- Unit testy: dispatch ordering (3-key matchne před 2-key), parse_shortcut, format_shortcut

## Proof Level

- This slice proves: contract + integration
- Real runtime required: yes (cargo check + testy, GUI manuální verifikace)
- Human/UAT required: yes (Ctrl+Alt+B nespustí build — vizuální ověření)

## Verification

- `cargo test --all-targets --all-features` — nové unit testy pro keymap modul (parse, format, dispatch ordering) projdou
- `./check.sh` — fmt + clippy + testy bez warningů
- Unit test: `Keymap` s bindings `Ctrl+B → Build` a `Ctrl+Alt+B → FocusBuild` vrátí `FocusBuild` při Ctrl+Alt+B stisku (ne Build)
- Unit test: `parse_shortcut("Ctrl+Shift+F")` vrátí `KeyboardShortcut { modifiers: COMMAND | SHIFT, key: F }`
- Unit test: `format_shortcut` vrátí "Ctrl+Shift+F" na Linuxu, respektuje OS
- Žádný ad-hoc `ctx.input(|i| i.modifiers.ctrl && ...)` keyboard handler v `workspace/mod.rs` (ověřeno grepem)
- Diagnostic: `parse_shortcut("invalid_garbage")` vrátí `None` (ne panic) — testováno unit testem `test_parse_shortcut_invalid`
- Diagnostic: `Keymap::dispatch` na prázdnou keymapu vrátí `None` — žádný false-positive match

## Observability / Diagnostics

- Runtime signals: dispatch vrací `Option<CommandId>` — `None` = žádný match, `Some(id)` = matchnutá akce
- Inspection surfaces: grep `consume_shortcut` v workspace/mod.rs — nesmí být ad-hoc, jen přes Keymap
- Failure visibility: pokud dispatch nevrátí správný CommandId, unit test selže s explicitním assert message
- Redaction constraints: none

## Integration Closure

- Upstream surfaces consumed: `CommandId` enum, `CommandAction`, `MenuActions`, `execute_command()`, `process_menu_actions()`
- New wiring introduced in this slice: `Keymap::dispatch()` volaný v `render_workspace()` před widget renderingem, výsledek → `execute_command()` → `process_menu_actions()`
- What remains before the milestone is truly usable end-to-end: S02 (chybějící handlery — Find, Replace, GotoLine, CommandPalette, ProjectSearch), S03 (uživatelská konfigurace keybindings, dynamické labely)

## Tasks

- [x] **T01: Keymap modul — struct, parse, format, dispatch + unit testy** `est:1h30m`
  - Why: Jádro celého milestone — testovatelný keymap modul bez UI závislostí. Pokryje R010 (centrální dispatch), R011 (exkluzivní modifier matching), R014 (cross-platform Ctrl↔Cmd).
  - Files: `src/app/keymap.rs`, `src/app/mod.rs`, `src/app/registry/mod.rs`, `src/app/ui/widgets/command_palette.rs`
  - Do: Vytvořit `keymap.rs` s `Keymap` struct, `parse_shortcut()`, `format_shortcut()`. Seřadit bindings od nejvíce modifikátorů. Dispatch přes `consume_shortcut`. Přidat nové `CommandId` varianty (FocusEditor, FocusBuild, FocusClaude). Změnit `Command.shortcut` z `Option<&'static str>` na `Option<KeyboardShortcut>`. Aktualizovat `init_defaults()` a `command_palette.rs` rendering pro nový typ. Přidat `pub mod keymap;` do `app/mod.rs`. Unit testy pro parse, format, dispatch ordering.
  - Verify: `cargo test --all-targets --all-features` projde, unit testy pro keymap pokrývají dispatch ordering (trojkombinace vs dvoukombinace), parse_shortcut, format_shortcut
  - Done when: `cargo check` čistý, keymap unit testy projdou, `Command.shortcut` je `Option<KeyboardShortcut>`

- [x] **T02: Napojení dispatch na workspace — nahrazení ad-hoc handlerů** `est:1h`
  - Why: Propojení keymap dispatch s existujícím command pipeline. Odstraní ad-hoc keyboard handlery a tím opraví modifier filtrování (R011). Pokryje R015 (VS Code konvence) a dovrší R010.
  - Files: `src/app/ui/workspace/mod.rs`, `src/app/ui/workspace/menubar/mod.rs`, `src/app/ui/workspace/menubar/edit.rs`, `src/app/ui/workspace/menubar/file.rs`, `src/app/ui/workspace/state/mod.rs`
  - Do: Přidat `Keymap` do `WorkspaceState`. Inicializovat z `CommandRegistry` při startu workspace. V `render_workspace()` volat `keymap.dispatch()` PŘED widget renderingem (ale po egui built-in handlerech). Výsledný `CommandId` směrovat přes `execute_command()` → `process_menu_actions()`. Smazat ad-hoc handlery (workspace/mod.rs:508-539). `consume_close_tab_shortcut()` nahradit dispatch — Ctrl+W přes keymap. Aktualizovat menu shortcut labely z hardcoded stringů na `format_shortcut()`. Rozšířit `execute_command()` o FocusEditor/FocusBuild/FocusClaude handling.
  - Verify: `./check.sh` projde čistě. `grep -n "i.modifiers.ctrl" src/app/ui/workspace/mod.rs` vrátí 0 výsledků (žádné ad-hoc handlery). Menu shortcut labely generované z `KeyboardShortcut`, ne z hardcoded stringů.
  - Done when: `./check.sh` čistý, žádný ad-hoc ctrl+key handler v workspace/mod.rs, všechny zkratky dispatchované centrálně přes Keymap

## Files Likely Touched

- `src/app/keymap.rs` (nový)
- `src/app/mod.rs`
- `src/app/registry/mod.rs`
- `src/app/ui/widgets/command_palette.rs`
- `src/app/ui/workspace/mod.rs`
- `src/app/ui/workspace/menubar/mod.rs`
- `src/app/ui/workspace/menubar/edit.rs`
- `src/app/ui/workspace/menubar/file.rs`
- `src/app/ui/workspace/state/mod.rs`
