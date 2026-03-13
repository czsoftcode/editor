# M004: Klávesové Zkratky a Centrální Keymap — Context

**Gathered:** 2026-03-13
**Status:** Queued — pending auto-mode execution.

## Project Description

Oprava rozbité klávesové zkratky, vybudování centralizovaného keymap dispatch systému napojeného na command registry, sjednocení zkratek s VS Code / JetBrains konvencemi, a uživatelská konfigurace přes `[keybindings]` sekci v `settings.toml`. Cross-platform podpora Linux + macOS (Ctrl↔Cmd).

## Why This Milestone

Klávesové zkratky v editoru mají tři kategorie problémů:

1. **Trojkombinace nefungují** — modifikátorové filtrování je rozbité. Např. `Ctrl+Alt+B` (focus build panel) spustí zároveň `Ctrl+B` (cargo build), protože handler pro `Ctrl+B` testuje jen `modifiers.ctrl && key_pressed(Key::B)` bez `!modifiers.alt`. Stejný problém platí pro všechny dvojkombinace, které matchnou i trojkombinace.

2. **Většina zkratek nemá keyboard handler** — menu zobrazuje `Ctrl+F` (find), `Ctrl+H` (replace), `Ctrl+G` (goto line), `Ctrl+P` (open file), `Ctrl+Shift+F` (project search), ale žádný z nich nemá implementovaný keyboard handler. Fungují jen přes menu klik. Command palette (`Ctrl+Shift+P`) se nedá otevřít klávesou vůbec.

3. **Architektura neumožňuje údržbu** — zkratky jsou roztroušené v `workspace/mod.rs` jako ad-hoc `ctx.input()` volání, nekonzistentní API (mix `key_pressed` a `consume_shortcut`), command registry má `shortcut` pole jako čistý label (`&'static str`) bez napojení na dispatch. Přidání nové zkratky vyžaduje editaci minimálně 3 souborů.

Uživatel nemůže efektivně pracovat s editorem bez funkčních klávesových zkratek — je to základní UX problém.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Používat Ctrl+F pro otevření search baru, Ctrl+H pro replace, Ctrl+G pro goto line — všechny odpovídající VS Code / JetBrains konvencím
- Otevřít command palette přes Ctrl+Shift+P (nebo F1)
- Používat Ctrl+Shift+F pro project-wide search bez konfliktu s jinými zkratkami
- Trojkombinace (Ctrl+Alt+B/E/A) fungují bez nechtěného spuštění dvoukombinací (Ctrl+B, Ctrl+E, Ctrl+A)
- Přemapovat klávesové zkratky v `[keybindings]` sekci settings.toml
- Na macOS používat Cmd místo Ctrl pro stejné zkratky

### Entry point / environment

- Entry point: klávesnice v editoru, settings.toml pro konfiguraci
- Environment: desktop editor (eframe/egui), Linux + macOS, single-process multi-window
- Live dependencies involved: none — vše je lokální

## Completion Class

- Contract complete means: centrální keymap dispatch parsuje a matchuje zkratky z registry; modifier filtrování je exkluzivní (Ctrl+B nematchne Ctrl+Alt+B); všechny zkratky z menu mají funkční keyboard handler; unit testy pokrývají modifier matching, config parsing, platform-aware dispatch.
- Integration complete means: celý tok funguje end-to-end — stisk klávesy → keymap lookup → command dispatch → akce v editoru; uživatelský override v settings.toml přepíše default binding; command palette zobrazuje aktuální (ne default) keybinding labels.
- Operational complete means: nový příkaz přidaný do command registry automaticky podporuje keybinding bez dalšího kódu; macOS Cmd mapování funguje bez duplikace handlerů.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Stisk Ctrl+Shift+F otevře project search (ne normální find). Stisk Ctrl+F otevře in-file search. Oba fungují bez konfliktu.
- Stisk Ctrl+Alt+B přepne focus na build panel BEZ spuštění cargo build. Stisk Ctrl+B (bez Alt) spustí build.
- Command palette se otevře přes Ctrl+Shift+P. Zobrazené zkratky odpovídají skutečným bindings (včetně uživatelských overrides).
- Uživatel přidá do settings.toml `[keybindings]` sekci s `"editor.save" = "Ctrl+Shift+S"` → Ctrl+S přestane ukládat, Ctrl+Shift+S začne.
- Na macOS: Cmd+S uloží, Cmd+Shift+F otevře project search.

## Risks and Unknowns

- **egui key event consumption** — egui `key_pressed()` nekonzumuje event (může matchnout vícekrát), zatímco `consume_shortcut()` ano. Centrální dispatch musí použít `consume_shortcut` pattern pro všechny zkratky, jinak se event dostane i do TextEdit widgetů (např. Ctrl+A = select all v TextEdit vs. custom akce).
- **egui Modifiers na macOS** — egui mapuje Cmd na `Modifiers::command` (ne `ctrl`). Dispatch musí používat `Modifiers::command` místo `Modifiers::ctrl` pro cross-platform, nebo explicitně řešit platform branching. Ověřit chování `KeyboardShortcut::new(Modifiers::COMMAND_OR_CTRL, Key::S)` v aktuální verzi egui 0.31.
- **Konflikty s egui built-in shortcuts** — egui TextEdit má built-in Ctrl+A (select all), Ctrl+C/V/X (copy/paste/cut), Ctrl+Z/Y (undo/redo). Centrální dispatch nesmí kolidovat s těmito — buď je nechat na TextEdit, nebo je explicitně konzumovat a forwardovat.
- **settings.toml parsing** — přidání `[keybindings]` sekce do existujícího settings.toml nesmí rozbít stávající serde deserializaci. Pole musí být `#[serde(default)]` a tolerovat chybějící sekci.
- **Pořadí dispatch** — trojkombinace musí matchnout před dvoukombinacemi. Dispatch musí řadit bindings od nejspecifičtějších (nejvíc modifikátorů) po nejméně specifické.

## Existing Codebase / Prior Art

- `src/app/registry/mod.rs` — CommandRegistry s `shortcut: Option<&'static str>` jako čistý label. 17 registrovaných příkazů, z toho 8 s shortcut labelem. Dispatch je v `execute_command()` přes `CommandAction::Internal(CommandId)`.
- `src/app/ui/workspace/mod.rs:508-539` — Ad-hoc keyboard handling: 6× `ctx.input(|i| i.modifiers.ctrl && i.key_pressed(...))` bez exkluzivního modifier filtrování. Jediná výjimka: `consume_close_tab_shortcut()` používá správný `consume_shortcut` pattern.
- `src/app/ui/workspace/menubar/edit.rs` — Menu items s `.shortcut_text("Ctrl+F")` atd. — čistě vizuální label, žádný dispatch.
- `src/app/ui/widgets/command_palette.rs` — Command palette widget. `execute_command()` mapuje `CommandId` na `MenuActions` flagy. Palette se nikde neotvírá z keyboard handleru.
- `src/app/ui/terminal/instance/input.rs` — Terminálový key handler: `terminal_key_bytes()` explicitně filtruje `ctrl && !shift && !alt` (řádek 5) — vzor pro správné modifier filtrování.
- `src/app/ui/editor/mod.rs` — Editor struct s `show_search`, `show_replace`, `show_goto_line` flagy — existující stav, který keyboard handler potřebuje nastavit.
- `src/app/ui/editor/search.rs` — Search bar rendering. `show_search` se nastavuje na `false` (zavření) ale nikdy na `true` z klávesové zkratky.
- `src/settings.rs` — Settings struct se serde TOML (de)serializací. `[keybindings]` sekce bude nový field.

> See `.gsd/DECISIONS.md` for all architectural and pattern decisions — it is an append-only register; read it during planning, append to it during execution.

## Relevant Requirements

- Nový scope — tento milestone zavádí nové requirements pro keyboard handling, centrální dispatch a konfiguraci. Nenavazuje na existující Active requirements.
- Částečně relevanantní: stávající Active backlog items (V-1 přes V-3, K-1, S-1, S-3, S-4, N-5) nejsou tímto milestone adresovány.

## Scope

### In Scope

- Centrální keymap dispatch: parsování shortcut stringu ("Ctrl+Shift+F") na `KeyboardShortcut`, matchování přes `consume_shortcut`, routing na `CommandId`
- Oprava modifier filtrování: exkluzivní matching (Ctrl+B ≠ Ctrl+Alt+B)
- Implementace chybějících keyboard handlerů pro všechny příkazy v menu: Ctrl+F (find), Ctrl+H (replace), Ctrl+G (goto line), Ctrl+P (open file), Ctrl+Shift+F (project search), Ctrl+Shift+P (command palette)
- Sjednocení s VS Code / JetBrains konvencemi — přidat standardní zkratky které chybí (Ctrl+Tab pro přepínání tabů, Ctrl+Shift+P pro command palette, Ctrl+, pro settings, atd.)
- Uživatelská konfigurace: `[keybindings]` sekce v settings.toml s formátem `"command.id" = "Ctrl+Shift+X"`
- Cross-platform: macOS Cmd↔Ctrl mapping (egui `Modifiers::COMMAND_OR_CTRL`)
- Aktualizace command palette — zobrazovat skutečné keybinding labels (ne hardcoded), reflektovat uživatelské overrides
- i18n pro nové UI prvky (pokud přibudou — např. keybinding conflict warning)

### Out of Scope / Non-Goals

- Keybinding UI editor (vizuální přemapování v Settings modalu) — pro v1 stačí ruční editace settings.toml
- Vim/Emacs mode nebo modální keybinding systém
- Multi-key sequences (chord keybindings jako Ctrl+K Ctrl+C)
- Keybinding kontextové zóny (jiné zkratky v editoru vs. terminálu vs. file tree) — dispatch je globální, terminál má vlastní handler jako dnes
- Macro recording / replay

## Technical Constraints

- `cargo check` + `./check.sh` musí projít po každé slice
- Žádné nové runtime závislosti
- Neblokovat UI vlákno — keymap lookup musí být O(1) nebo O(n) s malým n (<50)
- Zachovat existující terminálový key handler (`terminal_key_bytes`) nezávislý — terminál zpracovává klávesy přímo pro PTY
- egui built-in shortcuts (Ctrl+A/C/V/X/Z/Y v TextEdit) nesmí být přepsány centrálním dispatchem
- Backwards compatible settings.toml — chybějící `[keybindings]` = default bindings

## Integration Points

- `src/app/registry/mod.rs` — shortcut field se změní z `Option<&'static str>` na parsovaný `KeyboardShortcut`. Alternativně přidat parsovanou verzi vedle stringu.
- `src/app/ui/workspace/mod.rs` — nahradit ad-hoc keyboard handling centrálním dispatch voláním
- `src/app/ui/workspace/menubar/edit.rs` — shortcut_text() labely musí čerpat z keymap (ne hardcoded)
- `src/app/ui/widgets/command_palette.rs` — shortcut labels z keymap
- `src/app/ui/editor/mod.rs` — napojit `show_search`, `show_replace`, `show_goto_line` na command dispatch
- `src/settings.rs` — nová `[keybindings]` sekce, serde parsing, merge s defaults

## Open Questions

- **egui `COMMAND_OR_CTRL`** — egui 0.31 má `Modifiers::COMMAND_OR_CTRL` constant? Nebo je třeba runtime detekce `cfg!(target_os = "macos")`? Ověřit v egui docs při planning.
- **Dispatch priority** — pokud je TextEdit focused a uživatel stiskne Ctrl+F, má se otevřít search bar (centrální dispatch), nebo má TextEdit dostat event prvně? Pravděpodobně centrální dispatch by měl mít prioritu (consume_shortcut před TextEdit rendering).
- **Keybinding conflict detection** — pokud uživatel v settings.toml přiřadí stejnou zkratku dvěma příkazům, co se stane? Pravděpodobně první match wins + warning log. Rozhodnout při implementaci.
