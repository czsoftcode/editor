---
estimated_steps: 8
estimated_files: 4
---

# T01: Keymap modul — struct, parse, format, dispatch + unit testy

**Slice:** S01 — Centrální keymap dispatch a oprava modifier filtrování
**Milestone:** M004

## Description

Vytvořit `src/app/keymap.rs` — samostatný modul s `Keymap` structem, parserem shortcut stringů, formátovačem shortcut labelů a dispatch logikou. Tento modul nemá UI závislosti — je plně testovatelný unit testy. Zároveň přidat nové `CommandId` varianty a změnit `Command.shortcut` na `Option<KeyboardShortcut>`.

## Steps

1. **Přidat nové CommandId varianty** do `src/app/ui/widgets/command_palette.rs`: `FocusEditor`, `FocusBuild`, `FocusClaude`. Tyto pokryjí trojkombinace Ctrl+Alt+E/B/A.

2. **Změnit `Command.shortcut`** v `src/app/registry/mod.rs` z `Option<&'static str>` na `Option<egui::KeyboardShortcut>`. Aktualizovat `init_defaults()` — místo `Some("Ctrl+B")` použít `Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::B))`. Přidat nové trojkombinace (Ctrl+Alt+E → FocusEditor, Ctrl+Alt+B → FocusBuild, Ctrl+Alt+A → FocusClaude). Použít `Modifiers::COMMAND` všude místo `Modifiers::CTRL`.

3. **Aktualizovat command palette rendering** v `command_palette.rs` — `cmd.shortcut` je teď `KeyboardShortcut`, ne `&str`. Formátovat label přes `format_shortcut()` (bude vytvořena v kroku 5).

4. **Vytvořit `src/app/keymap.rs`** s:
   - `pub struct Keymap` — drží `Vec<(KeyboardShortcut, CommandId)>` seřazený od nejvíce modifikátorů po nejméně
   - `Keymap::from_commands(commands: &[Command]) -> Self` — extrahuje shortcuty z command registry a seřadí
   - `Keymap::dispatch(input: &mut egui::InputState) -> Option<CommandId>` — iteruje seřazené bindings, volá `input.consume_shortcut()`, vrátí první match
   - Helper `modifier_count(Modifiers) -> u8` pro řazení

5. **Přidat `parse_shortcut()` a `format_shortcut()`** do `keymap.rs`:
   - `parse_shortcut(s: &str) -> Option<KeyboardShortcut>` — parsuje "Ctrl+Alt+B", "Ctrl+Shift+F" atd. Rozpoznává Ctrl/Cmd → `Modifiers::COMMAND`, Alt, Shift a egui key names.
   - `format_shortcut(shortcut: &KeyboardShortcut) -> String` — wrapper přes `KeyboardShortcut::format()` s egui `ModifierNames::NAMES` a `cfg!(target_os = "macos")` pro platform-aware label.

6. **Přidat `pub mod keymap;`** do `src/app/mod.rs`.

7. **Unit testy** v `keymap.rs` (`#[cfg(test)]` modul):
   - `test_parse_shortcut_basic` — "Ctrl+S" → COMMAND + S
   - `test_parse_shortcut_triple` — "Ctrl+Alt+B" → COMMAND | ALT + B
   - `test_parse_shortcut_shift` — "Ctrl+Shift+F" → COMMAND | SHIFT + F
   - `test_parse_shortcut_invalid` — "Foo+Bar" → None
   - `test_format_shortcut` — roundtrip parse→format
   - `test_dispatch_ordering` — keymap s Ctrl+B (Build) a Ctrl+Alt+B (FocusBuild): simulovaný event Ctrl+Alt+B vrátí FocusBuild, ne Build
   - `test_dispatch_consumes_event` — po dispatch vrátí None pro stejný event (konzumace)

8. **Ověřit** `cargo check` a `cargo test` projdou.

## Must-Haves

- [ ] `Keymap` struct se seřazenými bindings (víc modifikátorů first)
- [ ] `parse_shortcut()` parsuje "Ctrl+Alt+B", "Ctrl+Shift+F" správně
- [ ] `format_shortcut()` generuje platform-aware label
- [ ] `Command.shortcut` je `Option<KeyboardShortcut>` (ne `&'static str`)
- [ ] Nové CommandId: FocusEditor, FocusBuild, FocusClaude
- [ ] Dispatch ordering test: trojkombinace matchne před dvoukombinací
- [ ] Všechny zkratky přes `Modifiers::COMMAND`

## Verification

- `cargo check` projde čistě
- `cargo test --all-targets --all-features` projde — nové unit testy pro parse, format, dispatch ordering
- Dispatch ordering test explicitně ověřuje: Ctrl+Alt+B → FocusBuild (ne Build)

## Observability Impact

- **Dispatch signal:** `Keymap::dispatch()` vrací `Option<CommandId>` — `None` znamená žádný match, `Some(id)` identifikuje matchnutou akci. Budoucí agent může grepnout `dispatch` volání a sledovat, jak se výsledek propaguje.
- **Parse failure:** `parse_shortcut()` vrací `None` pro nevalidní vstup — žádný panic. Unit test `test_parse_shortcut_invalid` to ověřuje.
- **Binding ordering:** `Keymap::from_commands()` logicky řadí bindings od nejvíce modifikátorů po nejméně. `test_dispatch_ordering` explicitně ověřuje, že trojkombinace (Ctrl+Alt+B) se matchne před dvoukombinací (Ctrl+B).
- **Inspection:** `Keymap.bindings` je `pub` — agent může inspektovat obsah keymapy za runtime pro diagnostiku.

## Inputs

- `src/app/registry/mod.rs` — existující `Command` struct a `init_defaults()`
- `src/app/ui/widgets/command_palette.rs` — existující `CommandId` enum a `execute_command()`
- egui 0.31 `KeyboardShortcut`, `Modifiers::COMMAND`, `consume_shortcut` API

## Expected Output

- `src/app/keymap.rs` — nový modul s Keymap, parse_shortcut, format_shortcut, unit testy
- `src/app/mod.rs` — přidán `pub mod keymap;`
- `src/app/registry/mod.rs` — `Command.shortcut: Option<KeyboardShortcut>`, aktualizovaný `init_defaults()`
- `src/app/ui/widgets/command_palette.rs` — nové CommandId varianty, shortcut label rendering přes format_shortcut
