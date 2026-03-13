---
id: S01
parent: M004
milestone: M004
provides:
  - Keymap modul s dispatch, parserem, formátovačem shortcut stringů (src/app/keymap.rs)
  - Centrální dispatch v render_workspace() nahrazující všechny ad-hoc keyboard handlery
  - Nové CommandId varianty (FocusEditor, FocusBuild, FocusClaude) pro trojkombinace
  - Command.shortcut změněn z Option<&'static str> na Option<KeyboardShortcut>
  - Dynamické menu shortcut labely generované z Keymap dat
  - parse_shortcut() parser pro string→KeyboardShortcut konverzi (základ pro S03 konfiguraci)
  - format_shortcut() pro generování labelů z dat (ne hardcoded stringy)
  - Keymap field v WorkspaceState inicializovaný z CommandRegistry
requires:
  - slice: none
    provides: first slice
affects:
  - S02
  - S03
key_files:
  - src/app/keymap.rs
  - src/app/mod.rs
  - src/app/registry/mod.rs
  - src/app/ui/widgets/command_palette.rs
  - src/app/ui/workspace/mod.rs
  - src/app/ui/workspace/state/mod.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/workspace/menubar/mod.rs
  - src/app/ui/workspace/menubar/edit.rs
  - src/app/ui/workspace/menubar/file.rs
key_decisions:
  - Modifiers::COMMAND použit všude místo Modifiers::CTRL — cross-platform (Ctrl na Linuxu, Cmd na macOS)
  - Keymap bindings seřazeny sestupně podle počtu modifikátorů — consume_shortcut ignoruje extra modifikátory, řazení zaručuje korektní prioritu trojkombinací
  - Ctrl+A/C/V/X/Z/Y neregistrovány v keymapu — TextEdit je zpracuje sám, interceptování by rozbilo Copy/Paste/Undo
  - Focus panel commandy implementovány přes MenuActions flagy + process_menu_actions, ne přes přímý &mut WorkspaceState v execute_command
  - Keymap dispatch umístěn PŘED widget renderingem — consume_shortcut konzumuje event, musí být první
  - Clipboard a editor-interní zkratky ponechány s hardcoded menu labely — nejsou v command registry
patterns_established:
  - Keymap::from_commands() pro extrakci shortcutů z command registry
  - format_shortcut() jako centrální formátovač shortcut labelů (palette, menu)
  - shortcut_label(keymap, CommandId) helper pro lookup labelů v menu
  - modifier_count() pro řazení bindings od nejspecifičtějšího
observability_surfaces:
  - Keymap::dispatch() vrací Option<CommandId> — None = žádný match, Some = matchnutý příkaz
  - Keymap.bindings je pub — inspektovatelný obsah keymapy za runtime
  - parse_shortcut() vrací None pro nevalidní vstup (pokrytý unit testem)
  - "grep -c 'i.modifiers.ctrl' src/app/ui/workspace/mod.rs" musí vracet 0
drill_down_paths:
  - .gsd/milestones/M004/slices/S01/tasks/T01-SUMMARY.md
  - .gsd/milestones/M004/slices/S01/tasks/T02-SUMMARY.md
duration: 45m
verification_result: passed
completed_at: 2026-03-13
---

# S01: Centrální keymap dispatch a oprava modifier filtrování

**Centrální `Keymap` dispatch nahrazuje všechny ad-hoc keyboard handlery. Modifier filtrování je exkluzivní — trojkombinace (Ctrl+Alt+B) nespouštějí dvoukombinace (Ctrl+B). Cross-platform Cmd/Ctrl přes Modifiers::COMMAND.**

## What Happened

**T01 (Keymap modul):** Vytvořen `src/app/keymap.rs` s `Keymap` structem, `parse_shortcut()` parserem (string→KeyboardShortcut s cross-platform Ctrl/Cmd→COMMAND mapováním), `format_shortcut()` wrapperem pro platform-aware labely, a `dispatch()` logikou seřazenou sestupně dle počtu modifikátorů. `Command.shortcut` převeden z `Option<&'static str>` na `Option<KeyboardShortcut>` v celé registry. Přidány `CommandId::FocusEditor/FocusBuild/FocusClaude` pro trojkombinace. Command palette rendering aktualizován na `format_shortcut()`. 9 unit testů pokrývá parse, format roundtrip, dispatch ordering, event consumption, prázdnou keymapu a nevalidní vstupy.

**T02 (Workspace napojení):** Keymap přidán jako field do `WorkspaceState`, inicializovaný z `CommandRegistry` přes `Keymap::from_commands()`. V `render_workspace()` nahrazen celý blok ad-hoc shortcutů (Ctrl+S/B/R/W, Ctrl+Alt+E/B/A) jediným `keymap.dispatch()` voláním PŘED widget renderingem. `consume_close_tab_shortcut()` smazána — Ctrl+W jde přes keymap. Focus panel commandy napojeny přes nové `MenuActions` flagy → `process_menu_actions()`. Menu shortcut labely v `file.rs`/`edit.rs` přepsány na dynamické `shortcut_label()` volání. Test `unsaved_close_guard_ctrl_w` přepsán na Keymap dispatch verzi.

## Verification

- `./check.sh` — fmt + clippy + 156 testů projdou čistě ✅
- 9 keymap unit testů: dispatch ordering, parse (basic, triple, shift, comma, invalid), format roundtrip, event consumption, prázdná keymapa ✅
- `test_dispatch_ordering`: Ctrl+Alt+B → FocusBuild (ne Build) ✅
- `test_parse_shortcut_shift`: "Ctrl+Shift+F" → COMMAND | SHIFT + F ✅
- `test_parse_shortcut_invalid`: nevalidní vstup → None ✅
- `test_empty_keymap_dispatch`: prázdná keymapa → None ✅
- `grep -c "i.modifiers.ctrl" src/app/ui/workspace/mod.rs` → 0 ✅
- `grep -c "consume_close_tab_shortcut" src/app/ui/workspace/mod.rs` → 0 ✅
- Ctrl+A/C/V/X/Z/Y nejsou v keymapu (grep ověřeno) ✅
- `unsaved_close_guard_ctrl_w_via_keymap_dispatch` test projde ✅

## Requirements Advanced

- R010 (centrální dispatch) — všechny zkratky procházejí Keymap::dispatch(), žádné ad-hoc handlery
- R011 (exkluzivní modifier matching) — seřazení bindings + unit test dispatch_ordering dokazují korektní prioritu
- R014 (cross-platform Ctrl↔Cmd) — Modifiers::COMMAND použit ve všech registracích, parse_shortcut mapuje "Ctrl"/"Cmd" → COMMAND
- R015 (VS Code/JetBrains konvence) — defaultní bindings odpovídají konvencím, dispatch infrastruktura připravena

## Requirements Validated

- R010 — centrální dispatch plně funkční, ověřeno unit testy + grep na absenci ad-hoc handlerů
- R011 — exkluzivní modifier matching ověřen unit testem test_dispatch_ordering (trojkombinace matchne před dvoukombinací)
- R014 — Modifiers::COMMAND použit všude, parse_shortcut testován s "Ctrl" i "Cmd" vstupem

## New Requirements Surfaced

- none

## Requirements Invalidated or Re-scoped

- none

## Deviations

- `Debug` derive přidán na `CommandId` — potřeba pro explicitní assert messages v testech. Minimální dopad.
- Focus panel handling přes `MenuActions` flagy místo přímého `&mut WorkspaceState` v `execute_command()` — čistější architektura.
- Dva extra unit testy oproti plánu (`test_parse_shortcut_comma`, `test_empty_keymap_dispatch`) — rozšiřují diagnostické pokrytí.

## Known Limitations

- i18n klíče `command-name-focus-editor`, `command-name-focus-build`, `command-name-focus-claude` neexistují v překladových souborech — přidají se v S02 při UI integraci.
- Clipboard a editor-interní zkratky (Ctrl+F/H/G) mají hardcoded menu labely — nejsou v command registry, S02 rozhodne jestli je přidat.
- R012 (chybějící handlery) a R013 (uživatelská konfigurace) závisí na S02 a S03.

## Follow-ups

- none — vše pokračuje dle roadmapy v S02.

## Files Created/Modified

- `src/app/keymap.rs` — nový modul: Keymap struct, parse_shortcut, format_shortcut, dispatch, 9 unit testů
- `src/app/mod.rs` — přidán `pub mod keymap;`
- `src/app/registry/mod.rs` — Command.shortcut na Option<KeyboardShortcut>, init_defaults přepsán, nové trojkombinace
- `src/app/ui/widgets/command_palette.rs` — 3 nové CommandId varianty, Debug derive, shortcut label přes format_shortcut, focus flagy v execute_command
- `src/app/ui/workspace/mod.rs` — smazány ad-hoc handlery, centrální keymap dispatch, smazána consume_close_tab_shortcut()
- `src/app/ui/workspace/state/mod.rs` — přidán keymap field
- `src/app/ui/workspace/state/init.rs` — inicializace keymapy z CommandRegistry
- `src/app/ui/workspace/menubar/mod.rs` — focus flagy v MenuActions, shortcut_label() helper, handling fokus v process_menu_actions
- `src/app/ui/workspace/menubar/edit.rs` — dynamické shortcut labely
- `src/app/ui/workspace/menubar/file.rs` — dynamické shortcut labely
- `src/app/ui/workspace/tests/unsaved_close_guard.rs` — test přepsán na Keymap dispatch verzi
- `src/app/mod.rs` — testovací WorkspaceState literály doplněny o keymap field

## Forward Intelligence

### What the next slice should know
- `Keymap::from_commands()` vrací keymapu z `CommandRegistry` — S02 přidá nové commandy (Find, Replace, GotoLine, CommandPalette) do registry a keymapa se automaticky rozšíří.
- `parse_shortcut()` a `format_shortcut()` jsou stabilní API — S03 je použije pro parsování uživatelských overrides ze settings.toml.
- `execute_command()` v `command_palette.rs` je centrální bod pro přidávání nových command handlerů — S02 sem doplní Find/Replace/GotoLine/CommandPalette/ProjectSearch handling.
- Menu labely pro commandy v keymapu se generují přes `shortcut_label()` v `menubar/mod.rs` — S02 rozšíří tento pattern na nové menu položky.

### What's fragile
- Keymap dispatch se volá PŘED widget renderingem — pokud S02 přidá widgety, které samy konzumují keyboard eventy (např. search bar), musí dispatch respektovat focus stav (je-li search bar fokusovaný, dispatch nesmí interceptovat typing).
- `Keymap.bindings` je seřazený Vec — přidání nových bindings v S02/S03 musí projít přes `from_commands()` (nebo budoucí merge), ne přímý push (jinak se poruší řazení).

### Authoritative diagnostics
- `cargo test --bin polycredo-editor app::keymap` — spustí 9 keymap unit testů
- `grep -c "i.modifiers.ctrl" src/app/ui/workspace/mod.rs` — musí být 0 (potvrzuje absenci ad-hoc handlerů)
- `Keymap.bindings` je pub — runtime inspekce obsahu keymapy

### What assumptions changed
- Původní odhad T01 1h30m — reálná doba 25m (registry a palette změny byly jednodušší než odhadováno)
- Původní odhad T02 1h — reálná doba 20m (existující MenuActions pipeline dobře rozšiřitelný)
