# M004: Klávesové Zkratky a Centrální Keymap

**Vision:** Funkční klávesové zkratky s centrálním dispatch systémem — modifier filtrování nerozbíjí trojkombinace, všechny zkratky z menu mají keyboard handler, uživatel může přemapovat zkratky v settings.toml, macOS Cmd funguje automaticky.

## Success Criteria

- Ctrl+Alt+B přepne fokus na build panel BEZ spuštění cargo build. Ctrl+B (bez Alt) spustí build.
- Ctrl+F otevře in-file search bar. Ctrl+H otevře replace. Ctrl+G otevře goto line. Ctrl+Shift+F otevře project search. Žádné konflikty mezi nimi.
- Ctrl+Shift+P (nebo F1) otevře command palette. Palette zobrazuje aktuální keybinding labely (ne hardcoded).
- Uživatel přidá `[keybindings]` sekci do settings.toml s `"editor.save" = "Ctrl+Shift+S"` → Ctrl+Shift+S uloží, Ctrl+S přestane ukládat.
- Na macOS: Cmd nahrazuje Ctrl ve všech zkratkách automaticky (přes `Modifiers::COMMAND`).
- `cargo check` + `./check.sh` projde po každé slice.

## Key Risks / Unknowns

- **egui `consume_shortcut` ignoruje extra modifikátory** — `Ctrl+B` matchne i `Ctrl+Alt+B`. Dispatch MUSÍ řadit bindings od nejspecifičtějšího (nejvíc modifikátorů) po nejméně specifické. Chyba v řazení = trojkombinace nefungují.
- **TextEdit built-in shortcuts** — egui TextEdit interně zpracovává Ctrl+A/C/V/X/Z/Y. Centrální dispatch je nesmí konzumovat, jinak Copy/Paste/Undo přestanou fungovat v editoru.
- **Dispatch timing** — centrální dispatch musí proběhnout před widget renderingem (consume_shortcut odebere event z fronty dřív než ho TextEdit vidí), ale po egui built-in handlerech pro Ctrl+C/V/X.

## Proof Strategy

- egui `consume_shortcut` ordering → retire in S01 unit testem: assert že keymap seřazený od 3-key → 2-key → 1-key vrátí správný CommandId pro Ctrl+Alt+B (ne Ctrl+B akci)
- TextEdit koexistence → retire in S01 tím, že Ctrl+A/C/V/X/Z/Y nejsou registrovány v keymapu — TextEdit je zpracuje sám

## Verification Classes

- Contract verification: `cargo check`, `./check.sh` (195+ testů), nové unit testy pro shortcut parsing, modifier ordering, keymap dispatch, config parsing
- Integration verification: manuální test v GUI — klávesové zkratky spouštějí správné akce
- Operational verification: none
- UAT / human verification: vizuální ověření v GUI (menu labely, command palette labely, search bar otevření)

## Milestone Definition of Done

This milestone is complete only when all are true:

- Centrální `Keymap` dispatch nahrazuje všechny ad-hoc `ctx.input()` keyboard handlery
- Modifier filtrování je exkluzivní — Ctrl+B nematchne Ctrl+Alt+B (ověřeno unit testem)
- Všechny zkratky z menu (Ctrl+F, H, G, P, Shift+F, Shift+P) mají funkční keyboard handler
- Command palette se otvírá přes Ctrl+Shift+P
- Uživatelský override v settings.toml přemapuje zkratku a menu/palette zobrazují nový binding
- Cross-platform `Modifiers::COMMAND` je použit místo `Modifiers::CTRL` pro všechny zkratky
- `cargo check` + `./check.sh` projde čistě
- Final integrated acceptance scenarios z M004-CONTEXT pass

## Requirement Coverage

- Covers: R010 (centrální dispatch), R011 (exkluzivní modifier matching), R012 (chybějící handlery), R013 (uživatelská konfigurace), R014 (cross-platform Ctrl↔Cmd), R015 (VS Code/JetBrains konvence)
- Partially covers: none
- Leaves for later: none
- Orphan risks: none — všech 6 active requirements pro M004 je pokryto

## Slices

- [x] **S01: Centrální keymap dispatch a oprava modifier filtrování** `risk:high` `depends:[]`
  > After this: Ctrl+Alt+B přepne fokus na build panel bez spuštění cargo build. Ctrl+B spustí build. Všechny existující zkratky (Ctrl+S, Ctrl+W, Ctrl+B, Ctrl+R) fungují přes centrální dispatch místo ad-hoc handlerů. Cross-platform Cmd/Ctrl automaticky přes Modifiers::COMMAND.
- [x] **S02: Chybějící keyboard handlery a oživení command palette** `risk:medium` `depends:[S01]`
  > After this: Ctrl+F otevře search bar, Ctrl+H otevře replace, Ctrl+G otevře goto line, Ctrl+Shift+P otevře command palette, Ctrl+Shift+F otevře project search. Palette zobrazuje skutečné keybinding labely z keymapu.
- [ ] **S03: Uživatelská konfigurace keybindings a dynamické labely** `risk:low` `depends:[S01,S02]`
  > After this: Uživatel přidá `[keybindings]` sekci do settings.toml, přemapuje zkratku, a ta funguje. Menu a command palette zobrazují aktuální (ne default) keybinding labely včetně uživatelských overrides.

## Boundary Map

### S01 → S02

Produces:
- `src/app/keymap.rs` — `Keymap` struct s `dispatch(&mut InputState) -> Option<CommandId>`, `parse_shortcut(str) -> KeyboardShortcut`, `get_shortcut_for_command(CommandId) -> Option<KeyboardShortcut>`
- `Command.shortcut` field změněn z `Option<&'static str>` na `Option<KeyboardShortcut>` — parsovaná forma pro dispatch i label rendering
- `format_shortcut(KeyboardShortcut, Os) -> String` — formátování shortcut labelu (Ctrl+S na Linuxu, ⌘S na macOS)
- Existující zkratky (Save, CloseTab, Build, Run, focus panel trojkombinace) dispatchované centrálně

Consumes:
- nothing (first slice)

### S01 → S03

Produces:
- Stabilní `Keymap` API pro merge s uživatelskými overrides
- `parse_shortcut()` parser pro konverzi stringu z settings.toml na `KeyboardShortcut`

Consumes:
- nothing (first slice)

### S02 → S03

Produces:
- Rozšířený `CommandId` enum o Find, Replace, GotoLine, CommandPalette, ToggleCommandPalette
- `MenuActions` rozšířené o find, replace, goto_line, command_palette flagy
- Funkční command palette widget (otvíratelný přes keyboard shortcut)

Consumes:
- S01: `Keymap` dispatch, `parse_shortcut`, `Command.shortcut: Option<KeyboardShortcut>`
