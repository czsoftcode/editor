---
id: M004
provides:
  - Centrální Keymap dispatch systém nahrazující všechny ad-hoc keyboard handlery
  - Exkluzivní modifier matching — trojkombinace nespouštějí dvoukombinace (seřazení dle modifier_count)
  - 7 nových CommandId variant (FocusEditor, FocusBuild, FocusClaude, Find, Replace, GotoLine, CommandPalette)
  - Cross-platform Ctrl↔Cmd přes Modifiers::COMMAND
  - Funkční keyboard handlery pro všechny menu zkratky (Ctrl+F/H/G/Shift+P/Shift+F, F1)
  - Command palette toggle přes Ctrl+Shift+P a F1
  - Uživatelská konfigurace keybindings přes [keybindings] sekci v settings.toml
  - Validace overrides (reserved keys, invalid input, conflicts, unknown id)
  - Dynamické menu a palette shortcut labely reflektující uživatelské overrides
  - parse_shortcut() parser s whitelistem standalone kláves (F1–F12, Escape, Delete, Insert)
  - format_shortcut() pro platform-aware shortcut labely
key_decisions:
  - Modifiers::COMMAND místo Modifiers::CTRL — cross-platform (Ctrl na Linuxu, Cmd na macOS)
  - Keymap bindings seřazeny sestupně dle modifier_count() — zajišťuje prioritu trojkombinací
  - Ctrl+A/C/V/X/Z/Y neregistrovány a neoverridovatelné — TextEdit je zpracuje sám
  - Override na CommandRegistry commands (single source of truth) — palette i menu se aktualizují automaticky
  - F1 alternativní binding jako druhý command záznam — bez nového multi-binding mechanismu
  - Keymap dispatch PŘED widget renderingem — consume_shortcut konzumuje event dřív než widgety
  - Re-init defaults před apply overrides v save flow — zaručuje konzistenci
patterns_established:
  - Keymap::from_commands() extrahuje shortcuty z command registry
  - init_defaults() → apply_keybinding_overrides() → Keymap::from_commands() pipeline pro keybinding lifecycle
  - format_shortcut() jako centrální formátovač shortcut labelů
  - shortcut_label(keymap, CommandId) helper pro menu lookup
  - is_standalone_key_allowed() whitelist pro standalone klávesy bez modifikátoru
  - Menu položky s flagy pattern — kliknutí nastaví flag, process_menu_actions řeší stav
observability_surfaces:
  - Keymap::dispatch() vrací Option<CommandId> — None = žádný match, Some = matchnutý příkaz
  - Keymap.bindings a Command.shortcut jsou pub — inspektovatelné za runtime
  - parse_shortcut() vrací None pro nevalidní vstup (pokrytý unit testy)
  - eprintln! warningy s [keybinding] prefixem pro diagnostiku override problémů
  - grep -c "i.modifiers.ctrl" src/app/ui/workspace/mod.rs musí vracet 0
requirement_outcomes:
  - id: R010
    from_status: active
    to_status: validated
    proof: Keymap::dispatch() v render_workspace() nahrazuje všechny ad-hoc handlery. 9 keymap unit testů pass. grep na "i.modifiers.ctrl" v workspace/mod.rs = 0.
  - id: R011
    from_status: active
    to_status: validated
    proof: test_dispatch_ordering — Ctrl+Alt+B → FocusBuild (ne Build). Bindings seřazeny sestupně dle modifier_count().
  - id: R012
    from_status: active
    to_status: validated
    proof: 4 nové CommandId varianty (Find, Replace, GotoLine, CommandPalette). 5 command registrací včetně F1. test_dispatch_new_commands a test_dispatch_command_palette_ordering pass. Menu napojení na flagy.
  - id: R013
    from_status: active
    to_status: validated
    proof: apply_keybinding_overrides() s 10 unit testy. keybindings HashMap s serde(default) backward compat. Init + save wiring. Menu/palette labely reflektují overrides.
  - id: R014
    from_status: active
    to_status: validated
    proof: Modifiers::COMMAND použit ve všech registracích (0 výskytů Modifiers::CTRL). parse_shortcut mapuje Ctrl/Cmd → COMMAND. format_shortcut() platform-aware.
  - id: R015
    from_status: active
    to_status: validated
    proof: Ctrl+F/H/G/Shift+P/F1 odpovídají VS Code konvencím. Centrální dispatch + exkluzivní matching + uživatelská konfigurovatelnost.
duration: 1h40m
verification_result: passed
completed_at: 2026-03-13
---

# M004: Klávesové Zkratky a Centrální Keymap

**Centrální dispatch systém s exkluzivním modifier matchingem, kompletními keyboard handlery pro všechny menu zkratky, command palette (Ctrl+Shift+P / F1), uživatelskou konfigurací keybindings přes settings.toml, a cross-platform Cmd/Ctrl podporou.**

## What Happened

Tři slicí vybudovaly kompletní keymap infrastrukturu od základů.

**S01 (Centrální dispatch)** vytvořil jádro — `Keymap` modul s `dispatch()`, `parse_shortcut()` a `format_shortcut()`. Klíčový design: bindings seřazeny sestupně dle počtu modifikátorů, takže `Ctrl+Alt+B` (3 modifikátory) matchne před `Ctrl+B` (2). Všechny ad-hoc `ctx.input()` handlery v workspace/mod.rs byly nahrazeny jediným `keymap.dispatch()` voláním umístěným PŘED widget renderingem. `Command.shortcut` převeden z `Option<&'static str>` na `Option<KeyboardShortcut>` — parsovaná forma slouží pro dispatch i label rendering. Přidány `FocusEditor/FocusBuild/FocusClaude` CommandId varianty pro trojkombinace. Menu shortcut labely přepsány na dynamické `shortcut_label()` volání. 9 unit testů pokrývá parse, format, dispatch ordering, event consumption.

**S02 (Chybějící handlery + command palette)** rozšířil pipeline o `Find`, `Replace`, `GotoLine`, `CommandPalette` varianty. 5 nových command registrací včetně F1 alternativního bindingu (standalone klávesa přes nový `is_standalone_key_allowed()` whitelist). MenuActions pipeline rozšířen o 4 nové flagy. Command palette toggle implementován (Ctrl+Shift+P zavře i otevře — VS Code chování). Find při otevřeném search baru refocusne input. 7 nových i18n klíčů × 5 jazyků. 4 nové unit testy.

**S03 (Uživatelská konfigurace)** přidal `keybindings: HashMap<String, String>` do Settings s `#[serde(default)]` backward compat. `apply_keybinding_overrides()` validuje: reserved keys (Ctrl+A/C/V/X/Z/Y), nevalidní shortcut string, konflikty, neznámé command id, prázdný string (vymaže binding). Wiring: overrides se aplikují PŘED `Keymap::from_commands()` v init, a po save se provede re-init defaults + apply overrides + keymap rebuild. Menu a palette labely reflektují overrides automaticky díky single-source-of-truth architektuře (override na CommandRegistry commands). 10 unit testů + 2 backward compat testy.

## Cross-Slice Verification

### Success Criteria

**Ctrl+Alt+B přepne fokus BEZ spuštění cargo build. Ctrl+B spustí build.**
→ ✅ `test_dispatch_ordering` unit test: Ctrl+Alt+B → FocusBuild (ne Build). Bindings seřazeny sestupně dle modifier_count(). grep "i.modifiers.ctrl" workspace/mod.rs = 0.

**Ctrl+F/H/G/Shift+F otevře search/replace/goto/project search bez konfliktů.**
→ ✅ 4 nové CommandId varianty registrovány s příslušnými shortcuts. `test_dispatch_new_commands` potvrzuje dispatch. `test_dispatch_command_palette_ordering` potvrzuje Ctrl+Shift+P vs Ctrl+P separaci.

**Ctrl+Shift+P (nebo F1) otevře command palette. Palette zobrazuje aktuální keybinding labely.**
→ ✅ F1 parsing přes `is_standalone_key_allowed()` whitelist (`test_parse_shortcut_f1` pass). Palette shortcut labely přes `format_shortcut()` z keymap dat. Toggle chování (otevření/zavření).

**Uživatel přidá `[keybindings]` sekci — přemapovaná zkratka funguje.**
→ ✅ `test_apply_override_basic` — override změní Command.shortcut. `settings_keybindings_roundtrip` — serde roundtrip. `settings_backward_compat_no_keybindings` — chybějící sekce = prázdná HashMap. Init + save wiring ověřeno.

**macOS: Cmd nahrazuje Ctrl automaticky.**
→ ✅ 0 výskytů `Modifiers::CTRL` v registry/keymap/workspace. `parse_shortcut` mapuje "Ctrl"/"Cmd" → COMMAND. `format_shortcut()` je platform-aware wrapper.

**cargo check + ./check.sh projde.**
→ ✅ 172 testů pass (0 failures), "All checks passed successfully!". cargo check čistý, clippy čistý.

### Definition of Done

- ✅ Centrální Keymap dispatch nahrazuje všechny ad-hoc handlery — grep potvrzuje 0 výskytů "i.modifiers.ctrl" v workspace/mod.rs
- ✅ Modifier filtrování je exkluzivní — test_dispatch_ordering ověřuje trojkombinace
- ✅ Všechny zkratky z menu mají funkční keyboard handler — 13 keymap testů pass
- ✅ Command palette se otvírá přes Ctrl+Shift+P — CommandPalette CommandId + toggle logika
- ✅ Uživatelský override v settings.toml přemapuje zkratku — 10 override testů + backward compat
- ✅ Cross-platform Modifiers::COMMAND použit všude — 0 Modifiers::CTRL
- ✅ cargo check + ./check.sh projde čistě — 172 testů, all green
- ✅ Všechny 3 slicí [x] v roadmapě, všechny summaries existují

## Requirement Changes

- R010 (centrální dispatch): active → validated — Keymap::dispatch() v render_workspace(), 13 unit testů, 0 ad-hoc handlerů
- R011 (exkluzivní modifier matching): active → validated — test_dispatch_ordering, modifier_count seřazení
- R012 (chybějící handlery): active → validated — 4 nové CommandId, 5 command registrací, 13 keymap testů pass
- R013 (uživatelská konfigurace): active → validated — apply_keybinding_overrides() s 10 unit testy, backward compat
- R014 (cross-platform Ctrl↔Cmd): active → validated — Modifiers::COMMAND, parse_shortcut Ctrl/Cmd→COMMAND
- R015 (VS Code/JetBrains konvence): active → validated — S01 dispatch + S02 konvence + S03 konfigurovatelnost

## Forward Intelligence

### What the next milestone should know
- Centrální keymap dispatch je stabilní a rozšiřitelný — nový command se přidá registrací v `init_defaults()` a automaticky se projeví v dispatch, menu labelech a command palette.
- Keybinding override pattern (`init_defaults() → apply_keybinding_overrides() → Keymap::from_commands()`) je znovupoužitelný pro budoucí rozšíření.
- `parse_shortcut()` podporuje standalone klávesy přes whitelist — F1–F12, Escape, Delete, Insert.
- Celý pipeline prošel 172 testy bez failures — quality gate je silný.

### What's fragile
- F1 jako druhý command záznam — `get_shortcut_for_command(CommandPalette)` vrátí Cmd+Shift+P (první registrovaný), ne F1. Budoucí "show all bindings" by potřeboval multi-binding support.
- Conflict detection je warning-only (eprintln) — žádný UI feedback pro uživatele. Pokud přiřadí stejnou zkratku dvěma příkazům, poslední vyhraje bez viditelného varování.
- Re-init defaults v save flow závisí na kompletní sadě defaultních zkratek v init_defaults() — nový command bez default shortcut funguje, ale explicitní test by posílil důvěru.

### Authoritative diagnostics
- `cargo test --bin polycredo-editor app::keymap` — 13 testů pokrývajících parse, dispatch, ordering
- `cargo test --bin polycredo-editor app::registry` — 10 testů pokrývajících override validaci
- `grep -c "i.modifiers.ctrl" src/app/ui/workspace/mod.rs` — musí být 0 (potvrzuje absenci ad-hoc handlerů)
- `grep -rn '\[keybinding\]' src/` — najde všechny warning log body pro diagnostiku

### What assumptions changed
- Původní odhad celého milestone ~4-5h — reálná doba ~1h40m. Registry a pipeline architektura z předchozích milestones byla velmi dobře rozšiřitelná.
- S01 rozhodnutí o hardcoded labelech pro Ctrl+F/H/G bylo revidováno v S02 — tyto zkratky přesunuty do command registry s dynamickými labely.
- egui `Modifiers::COMMAND` funguje cross-platform správně — žádný manuální `cfg!(target_os)` nebyl potřeba.

## Files Created/Modified

- `src/app/keymap.rs` — nový modul: Keymap struct, parse_shortcut, format_shortcut, dispatch, is_reserved_shortcut, is_standalone_key_allowed, 13 unit testů
- `src/app/mod.rs` — přidán pub mod keymap
- `src/app/registry/mod.rs` — Command.shortcut na Option<KeyboardShortcut>, init_defaults s 12 commandy, find_mut(), apply_keybinding_overrides(), 10 unit testů
- `src/app/ui/widgets/command_palette.rs` — 7 nových CommandId variant, Debug derive, format_shortcut labely, execute_command rozšířen
- `src/app/ui/workspace/mod.rs` — smazány ad-hoc handlery, centrální keymap dispatch
- `src/app/ui/workspace/state/mod.rs` — přidán keymap field
- `src/app/ui/workspace/state/init.rs` — inicializace keymapy, override aplikace
- `src/app/ui/workspace/menubar/mod.rs` — focus + command flagy v MenuActions, shortcut_label() helper, process_menu_actions rozšířen
- `src/app/ui/workspace/menubar/edit.rs` — dynamické shortcut labely, Command Palette menu položka
- `src/app/ui/workspace/menubar/file.rs` — dynamické shortcut labely
- `src/app/ui/workspace/modal_dialogs/settings.rs` — re-init + override + keymap rebuild v save flow
- `src/app/ui/workspace/tests/unsaved_close_guard.rs` — test přepsán na Keymap dispatch verzi
- `src/settings.rs` — keybindings HashMap field, serde(default), 2 backward compat testy
- `locales/{cs,en,sk,de,ru}/ui.ftl` — 7 command-name i18n klíčů per jazyk
- `locales/{cs,en,sk,de,ru}/menu.ftl` — menu-edit-command-palette klíč per jazyk
