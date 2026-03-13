---
id: T01
parent: S01
milestone: M004
provides:
  - Keymap modul s dispatch, parserem, formátovačem shortcut stringů
  - Nové CommandId varianty (FocusEditor, FocusBuild, FocusClaude)
  - Command.shortcut změněn z Option<&'static str> na Option<KeyboardShortcut>
  - 9 unit testů pro keymap modul
key_files:
  - src/app/keymap.rs
  - src/app/registry/mod.rs
  - src/app/ui/widgets/command_palette.rs
  - src/app/mod.rs
key_decisions:
  - Modifiers::COMMAND použit všude místo Modifiers::CTRL — cross-platform (Ctrl na Linuxu, Cmd na macOS)
  - parse_shortcut mapuje "Ctrl" i "Cmd" na Modifiers::COMMAND
  - Keymap bindings seřazeny sestupně podle počtu modifikátorů — consume_shortcut konzumuje event, řazení zaručuje korektní prioritu
  - CommandId získal Debug derive pro diagnostické assert messages v testech
patterns_established:
  - Keymap::from_commands() pro extrakci shortcutů z command registry
  - format_shortcut() jako centrální formátovač shortcut labelů (palette, menu)
  - modifier_count() pro řazení bindings
observability_surfaces:
  - Keymap::dispatch() vrací Option<CommandId> — None = žádný match, Some = matchnutý příkaz
  - Keymap.bindings je pub — inspektovatelný obsah za runtime
  - parse_shortcut() vrací None pro nevalidní vstup, pokrytý test_parse_shortcut_invalid
duration: 25m
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: Keymap modul — struct, parse, format, dispatch + unit testy

**Vytvořen centrální keymap modul (`src/app/keymap.rs`) s dispatch logikou, parserem a formátovačem shortcut stringů. Command.shortcut převeden z hardcoded stringů na egui KeyboardShortcut. Přidány 3 nové CommandId varianty pro fokus panelů.**

## What Happened

1. Přidány `CommandId::FocusEditor`, `FocusBuild`, `FocusClaude` do command_palette.rs. Execute_command má placeholder handlery (implementace v T02).

2. `Command.shortcut` změněn z `Option<&'static str>` na `Option<KeyboardShortcut>`. `init_defaults()` přepsán s helpery `cmd()`, `cmd_shift()`, `cmd_alt()` pro čitelnost. Registrovány nové trojkombinace: Ctrl+Alt+E → FocusEditor, Ctrl+Alt+B → FocusBuild, Ctrl+Alt+A → FocusClaude.

3. Command palette rendering aktualizován — shortcut label generován přes `format_shortcut()` místo přímého zobrazení stringu.

4. Vytvořen `src/app/keymap.rs`:
   - `Keymap` struct s `pub bindings: Vec<(KeyboardShortcut, CommandId)>` seřazenými sestupně dle počtu modifikátorů
   - `Keymap::from_commands()` extrahuje shortcuty z command registry a řadí
   - `Keymap::dispatch()` iteruje bindings, volá `consume_shortcut()`, vrátí první match
   - `parse_shortcut()` parsuje "Ctrl+Alt+B", "Ctrl+Shift+F" atd. s cross-platform mapováním Ctrl/Cmd → COMMAND
   - `format_shortcut()` wrapper přes egui API s platform-aware výstupem
   - `modifier_count()` helper pro řazení

5. Přidán `pub mod keymap;` do `src/app/mod.rs`.

6. 9 unit testů pokrývajících parse (basic, triple, shift, invalid, comma), format roundtrip, dispatch ordering, event consumption, prázdnou keymapu.

## Verification

- `cargo check` — čistý (0 errors, 0 warnings)
- `cargo test --all-targets --all-features` — 156 testů prošlo (9 nových keymap testů)
- `./check.sh` — fmt + clippy + testy bez warningů ✅
- `test_dispatch_ordering` explicitně ověřuje: Ctrl+Alt+B → FocusBuild (ne Build) ✅
- `test_parse_shortcut_shift`: "Ctrl+Shift+F" → COMMAND | SHIFT + F ✅
- `test_format_shortcut`: roundtrip formátování na Linuxu správné ✅
- `test_parse_shortcut_invalid`: nevalidní vstup vrací None ✅
- `test_empty_keymap_dispatch`: prázdná keymapa vrátí None ✅

### Slice verification status (T01 — 1. z 2 tasků):
- ✅ `cargo test --all-targets --all-features` projdou
- ✅ `./check.sh` čistý
- ✅ Dispatch ordering test: trojkombinace matchne před dvoukombinací
- ✅ parse_shortcut("Ctrl+Shift+F") vrátí správný KeyboardShortcut
- ✅ format_shortcut vrátí "Ctrl+Shift+F" na Linuxu
- ⏳ Grep ad-hoc handlerů — T02 je odstraní
- ✅ Diagnostické testy projdou

## Diagnostics

- `cargo test keymap` — spustí 9 keymap unit testů
- `Keymap.bindings` je veřejný — obsah keymapy inspektovatelný za runtime
- `Keymap::dispatch()` vrací `Option<CommandId>` — None/Some jako jasný signál
- `parse_shortcut()` vrací None pro nevalidní vstupy (pokrytý testem)

## Deviations

- Přidán `Debug` derive na `CommandId` — potřeba pro explicitní assert messages v testech. Minimální dopad, žádná breaking change.
- Přidán test `test_parse_shortcut_comma` a `test_empty_keymap_dispatch` navíc oproti plánu — rozšiřují diagnostické pokrytí.

## Known Issues

- `execute_command()` má pro FocusEditor/FocusBuild/FocusClaude prázdné handlery — implementace v T02 při napojení na workspace.
- i18n klíče `command-name-focus-editor`, `command-name-focus-build`, `command-name-focus-claude` ještě neexistují v překladových souborech — přidají se při UI integraci.

## Files Created/Modified

- `src/app/keymap.rs` — nový modul s Keymap struct, parse_shortcut, format_shortcut, dispatch, 9 unit testů
- `src/app/mod.rs` — přidán `pub mod keymap;`
- `src/app/registry/mod.rs` — Command.shortcut změněn na Option<KeyboardShortcut>, init_defaults přepsán s KeyboardShortcut, nové trojkombinace
- `src/app/ui/widgets/command_palette.rs` — 3 nové CommandId varianty, Debug derive, shortcut label přes format_shortcut()
- `.gsd/milestones/M004/slices/S01/S01-PLAN.md` — přidány diagnostické verifikační kroky
- `.gsd/milestones/M004/slices/S01/tasks/T01-PLAN.md` — přidána sekce Observability Impact
