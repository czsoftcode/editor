---
id: S02
parent: M004
milestone: M004
provides:
  - 4 nové CommandId varianty (Find, Replace, GotoLine, CommandPalette) v centrálním dispatch pipeline
  - 5 nových command registrací v init_defaults() včetně F1 alternativního bindingu
  - parse_shortcut() whitelist pro standalone klávesy (F1–F12, Escape, Delete, Insert) bez povinného modifikátoru
  - MenuActions flagy (find, replace, goto_line, command_palette) + process_menu_actions napojení na editor stav a command palette toggle
  - Menu edit.rs napojení na flagy + dynamické shortcut labely z keymapu + Command Palette menu položka
  - 7 command-name i18n klíčů × 5 jazyků (cs, en, sk, de, ru) + menu-edit-command-palette × 5 jazyků
  - 4 nové unit testy (F1 parsing, Escape parsing, nové commandy dispatch, Ctrl+Shift+P ordering)
requires:
  - slice: S01
    provides: Keymap dispatch, parse_shortcut, format_shortcut, Command.shortcut jako Option<KeyboardShortcut>, init_defaults se stávajícími commandy
affects:
  - S03
key_files:
  - src/app/ui/widgets/command_palette.rs
  - src/app/registry/mod.rs
  - src/app/keymap.rs
  - src/app/ui/workspace/menubar/mod.rs
  - src/app/ui/workspace/menubar/edit.rs
  - locales/{cs,en,sk,de,ru}/ui.ftl
  - locales/{cs,en,sk,de,ru}/menu.ftl
key_decisions:
  - F1 alternativní binding registrován jako druhý Command záznam s id "ui.command_palette_f1" a sdíleným i18n klíčem — nejjednodušší přístup bez nového multi-binding mechanismu
  - is_standalone_key_allowed() jako privátní helper pro whitelist F1-F12/Escape/Delete/Insert — oddělená od hlavní parse logiky, snadno rozšiřitelná
  - Command palette toggle v process_menu_actions (ne execute_command) — execute_command jen nastaví flag, process_menu_actions řeší stav
  - Find/Replace/GotoLine přesunuty z editor-interních hardcoded handlerů do centrálního command registry — reviduje S01 rozhodnutí o hardcoded labelech
  - Ctrl+F při otevřeném search baru refocusne input (ne toggle off) — VS Code chování
patterns_established:
  - Standalone klávesy v parse_shortcut() whitelist — nové shortcuty bez modifikátoru přidávat do is_standalone_key_allowed()
  - Menu položky s flagy pattern — kliknutí nastaví actions.* = true + ui.close_menu(), shortcut label přes shortcut_label(keymap, CommandId::*)
observability_surfaces:
  - parse_shortcut() vrací None pro nevalidní vstupy — pokrytý unit testy (F1 pass, "S" fail)
  - Keymap::dispatch() vrací None pokud shortcut nematchne — diagnostický signál
  - ws.command_palette.is_some() — runtime inspekce command palette toggle stavu
  - Chybějící i18n klíč se zobrazí jako raw identifikátor v UI — vizuálně detekovatelné
drill_down_paths:
  - .gsd/milestones/M004/slices/S02/tasks/T01-SUMMARY.md
duration: 30m
verification_result: passed
completed_at: 2026-03-13
---

# S02: Chybějící keyboard handlery a oživení command palette

**Rozšíření centrálního dispatch pipeline o Find/Replace/GotoLine/CommandPalette commandy, F1 alternativní binding, dynamické menu shortcut labely, command palette toggle, a kompletní i18n pro 7 command-name klíčů × 5 jazyků.**

## What Happened

Celý slice byl jeden koherentní task rozšiřující existující pipeline (CommandId → registry → MenuActions → process_menu_actions → menu/palette).

**CommandId a registry:** Přidány 4 nové varianty do `CommandId` enum (Find, Replace, GotoLine, CommandPalette). V `init_defaults()` registrováno 5 nových commandů — Find (Cmd+F), Replace (Cmd+H), GotoLine (Cmd+G), CommandPalette (Cmd+Shift+P), a alternativní F1 binding pro CommandPalette (registrován jako samostatný command záznam s id "ui.command_palette_f1").

**parse_shortcut() whitelist:** Přidána `is_standalone_key_allowed()` funkce pro F1–F12, Escape, Delete, Insert. Tyto klávesy obcházejí validaci "musí mít modifikátor" — F1 funguje bez Ctrl/Cmd.

**MenuActions pipeline:** Přidány 4 nové flagy (find, replace, goto_line, command_palette). `execute_command()` mapuje nové CommandId na flagy. `process_menu_actions()` implementuje: Find → show_search + refocus, Replace → show_search + show_replace + refocus, GotoLine → show_goto_line + focus, CommandPalette → toggle (is_some → None, jinak new).

**Menu edit.rs:** Find/Replace/GotoLine kliknutí nastavují flagy místo jen close_menu. Shortcut labely přepsány z hardcoded stringů na dynamické `shortcut_label(keymap, CommandId::*)`. Přidána Command Palette menu položka se separátorem.

**i18n:** 7 nových `command-name-*` klíčů v ui.ftl × 5 jazyků + `menu-edit-command-palette` v menu.ftl × 5 jazyků.

**Unit testy:** 4 nové: F1 parsing, Escape parsing, nové commandy dispatch, Ctrl+Shift+P vs Ctrl+P ordering.

## Verification

- `cargo test --bin polycredo-editor app::keymap` — 13/13 testů prošlo (9 existujících + 4 nové) ✅
- `./check.sh` — fmt + clippy + všechny testy pass ("All checks passed successfully!") ✅
- `grep -c "command-name-find" locales/en/ui.ftl` → 1 ✅
- `grep -c "command-name-focus-editor" locales/en/ui.ftl` → 1 ✅
- `grep -c "command-name-command-palette" locales/en/ui.ftl` → 1 ✅
- i18n klíče: 24 command-name klíčů v každém z 5 jazyků ✅
- Diagnostický test `test_parse_shortcut_invalid` — standalone "S" vrací None ✅
- Diagnostický test `test_parse_shortcut_f1` — F1 vrací Some ✅

## Requirements Advanced

- R012 — Chybějící keyboard handlery: Find (Ctrl+F), Replace (Ctrl+H), GotoLine (Ctrl+G), CommandPalette (Ctrl+Shift+P, F1) nyní mají funkční keyboard handlery přes centrální dispatch. Zbývá Ctrl+Shift+F (project search — už fungoval z S01).
- R015 — Sjednocení s VS Code/JetBrains konvencemi: Ctrl+F, Ctrl+H, Ctrl+G, Ctrl+Shift+P, F1 odpovídají VS Code konvencím. Command palette toggle chování (Ctrl+Shift+P zavře i otevře). Ctrl+F refocusne existující search bar.

## Requirements Validated

- R012 — Všechny zkratky z menu mají funkční keyboard handler. 4 nové unit testy potvrzují dispatch. Menu kliknutí nastavuje flagy. i18n kompletní.

## New Requirements Surfaced

- none

## Requirements Invalidated or Re-scoped

- none

## Deviations

- i18n klíč `menu-edit-command-palette` přidán do `menu.ftl` (ne `ui.ftl`) — konzistentní s existujícím vzorem, kde menu labely žijí v menu.ftl.

## Known Limitations

- Uživatelská konfigurace keybindings (přemapování v settings.toml) je scope S03 — zatím jen default bindings.
- Menu a command palette zobrazují default shortcut labely — uživatelské overrides z S03 se ještě neprojeví.

## Follow-ups

- none — všechno plánované je scope S03.

## Files Created/Modified

- `src/app/ui/widgets/command_palette.rs` — 4 nové CommandId varianty + 4 nové execute_command match větve
- `src/app/registry/mod.rs` — 5 nových command registrací v init_defaults()
- `src/app/keymap.rs` — is_standalone_key_allowed() helper + parse_shortcut whitelist + 4 nové unit testy
- `src/app/ui/workspace/menubar/mod.rs` — 4 nové MenuActions flagy + 4 nové process_menu_actions handlery
- `src/app/ui/workspace/menubar/edit.rs` — menu napojení na flagy + dynamické shortcut labely + Command Palette menu položka
- `locales/{cs,en,sk,de,ru}/ui.ftl` — 7 nových command-name klíčů per jazyk
- `locales/{cs,en,sk,de,ru}/menu.ftl` — menu-edit-command-palette klíč per jazyk

## Forward Intelligence

### What the next slice should know
- `parse_shortcut()` nyní podporuje standalone klávesy přes `is_standalone_key_allowed()` whitelist — uživatelské overrides v S03 mohou využít F1–F12, Escape, Delete, Insert jako binding cíle.
- F1 alternativní binding je registrován jako separátní command záznam — pokud S03 implementuje multi-binding mechanismus, "ui.command_palette_f1" záznam lze nahradit čistějším řešením.
- `shortcut_label()` v menubar/mod.rs hledá shortcut přes `keymap.get_shortcut_for_command(CommandId::*)` — po user override v S03 musí tato funkce reflektovat overridden shortcuty, ne jen defaults.

### What's fragile
- F1 jako druhý command záznam — pokud někdo zavolá `get_shortcut_for_command(CommandPalette)`, dostane Cmd+Shift+P (první registrovaný), ne F1. To je záměr, ale při budoucí implementaci "show all bindings" by se F1 ztratil.
- Menu edit.rs `shortcut_label` volání závisí na tom, že keymap obsahuje expected CommandId — pokud se v S03 změní registrace, labely se mohou rozbít (ale zobrazí se jen prázdný string, ne crash).

### Authoritative diagnostics
- `cargo test --bin polycredo-editor app::keymap` — 13 testů pokrývajících parse, dispatch, ordering. Selhání = regrese v keymap logice.
- `grep -c "command-name-" locales/*/ui.ftl` — 24 per jazyk. Pokles = chybějící i18n klíč.

### What assumptions changed
- S01 decision o hardcoded labelech pro editor-interní zkratky (Ctrl+F/H/G) revidován — tyto zkratky jsou nyní v command registry s dynamickými labely, protože S02 je potřeboval pro dispatch.
