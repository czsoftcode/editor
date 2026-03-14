# S03: Uživatelská konfigurace keybindings a dynamické labely — Research

**Date:** 2026-03-13

## Summary

Slice S03 má dva jasné deliverables: (1) `[keybindings]` sekce v settings.toml umožňující uživateli přemapovat zkratky, a (2) dynamické labely v menu i command palette reflektující uživatelské overrides. Kódová báze je na obojí dobře připravená díky S01/S02 infrastruktuře — `parse_shortcut()` parser, `format_shortcut()` formátovač, `Keymap::from_commands()` builder, a `shortcut_label()` helper v menu.

**Kritický detail:** Existují **dva zdroje shortcut dat** a oba se musí aktualizovat při user override:
- **Keymap.bindings** — dispatch engine, čte z něj `shortcut_label()` pro menu labely
- **Command.shortcut** — command palette čte shortcut přímo z `cmd.shortcut` v `Command` structu (command_palette.rs:174), ne z keymapy

Pokud user override změní jen keymapu, menu zobrazí správný label, ale command palette zobrazí starý default. Řešení: override aplikovat na `Command.shortcut` v `CommandRegistry` (single source of truth), pak rebuild keymapu z aktualizovaných commands. Obě UI (menu i palette) pak automaticky reflektují overrides.

**Další finding:** Keymapa se buduje jednou při `init_workspace()` a **nikdy se nerebuilduje**. Po uložení settings draftu (`save_settings_draft()`) se aktualizuje agent registry, ale ne keymap. S03 musí přidat rebuild keymapy po save — buď v `save_settings_draft()` nebo přes `settings_version` mechanismus v main loop.

## Recommendation

### Přístup: Override na CommandRegistry + rebuild keymapy po save

1. **Settings rozšíření:** Přidat `keybindings: HashMap<String, String>` do `Settings` struct s `#[serde(default)]` — mapuje command id (např. `"editor.save"`) na shortcut string (např. `"Ctrl+Shift+S"`). Chybějící sekce = prázdná mapa = default bindings.

2. **Apply overrides na CommandRegistry:** V `Registry::apply_keybinding_overrides(&mut self, overrides: &HashMap<String, String>)` — pro každý override parsovat shortcut přes `parse_shortcut()`, najít command dle id, a přepsat `cmd.shortcut`. To zajistí, že:
   - `Keymap::from_commands()` automaticky použije overridden shortcut pro dispatch
   - Command palette (která čte `cmd.shortcut`) zobrazí overridden label
   - `shortcut_label()` (která čte z `keymap.bindings`) zobrazí overridden label

3. **Rebuild flow:** V `save_settings_draft()` po `s.settings = Arc::new(draft)`:
   - Zavolat `s.registry.apply_keybinding_overrides(&s.settings.keybindings)` 
   - Aktualizovat `ws.keymap = Keymap::from_commands(s.registry.commands.get_all())`
   
4. **Init flow:** V `init_workspace()` po stávajícím `Keymap::from_commands()`:
   - Načíst keybindings ze settings a aplikovat overrides na registry commands *před* vytvořením keymapy

5. **Conflict detection:** Pokud dvě overrides mapují na stejnou zkratku → warning log + první match wins. Detekce při parsování.

6. **Testy:** Unit test pro parse+apply override, roundtrip persistence, conflict detection, backward compat bez `[keybindings]` sekce.

### Proč ne Keymap-only override

Alternativou by bylo overridy aplikovat jen na `Keymap` (přidat/nahradit bindings). Ale to by neopravilo command palette labely — ta čte z `Command.shortcut`, ne z keymapy. Museli bychom buď:
- Předělat palette aby četla z keymapy (zbytečná práce, rozbije existující API)
- Nebo overridy na oba místa (duplikace, fragile)

Override na CommandRegistry je single source of truth — čistější.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Shortcut string parsing | `parse_shortcut()` v `keymap.rs` | Hotový parser s whitelistem, cross-platform Ctrl/Cmd mapování, unit testy. |
| TOML parsing pro `[keybindings]` | `serde` + `toml` crate (v deps) | `HashMap<String, String>` s `#[serde(default)]` — existující pattern v Settings. |
| Shortcut label formátování | `format_shortcut()` v `keymap.rs` | Platform-aware, egui nativní. Palette i menu ho už používají. |
| Keymap rebuild | `Keymap::from_commands()` | Existující builder, automatické řazení dle modifier_count. |

## Existing Code and Patterns

- `src/settings.rs` — Settings struct s 25+ fieldy, všechny s `#[serde(default)]`. Přidání `keybindings: HashMap<String, String>` s `#[serde(default)]` je přesně existující vzor. `TempConfigDir` test pattern pro izolované persistence testy.
- `src/app/keymap.rs` — `parse_shortcut()` parser přijímá "Ctrl+Shift+S" a vrací `Option<KeyboardShortcut>`. `Keymap::from_commands()` extrahuje shortcuty z commands a řadí dle `modifier_count()`. Obojí přímo využitelné pro user overrides.
- `src/app/registry/mod.rs` — `CommandRegistry` s `by_id: HashMap<String, usize>` indexem — efektivní lookup commandu dle string id. `Command.shortcut: Option<KeyboardShortcut>` je místo kde override přepíše default.
- `src/app/ui/widgets/command_palette.rs:174-176` — Palette čte `cmd.shortcut` a volá `format_shortcut()`. Pokud se override aplikuje na `Command.shortcut`, palette automaticky zobrazí nový label.
- `src/app/ui/workspace/menubar/mod.rs:26-35` — `shortcut_label()` čte z `keymap.bindings`. Pokud se keymapa rebuildne z overridden commands, labely se aktualizují automaticky.
- `src/app/ui/workspace/modal_dialogs/settings.rs:223-270` — `save_settings_draft()` flow: ukládá draft, aktualizuje registry (agents), bumpne `settings_version`. Vzor pro přidání keymap rebuild.
- `src/app/ui/workspace/state/init.rs:48-51` — Inicializace keymapy v `init_workspace()`. Místo kde se musí aplikovat user overrides při startu.

## Constraints

- **Backward compatibility settings.toml** — `[keybindings]` sekce musí být `#[serde(default)]`. Chybějící sekce = prázdná `HashMap` = default bindings. Existující settings.toml bez `[keybindings]` nesmí selhat při deserializaci.
- **Keymap rebuild musí zachovat řazení** — `from_commands()` automaticky řadí bindings dle `modifier_count()`. Po override se musí zavolat `from_commands()`, ne ruční manipulace s `bindings` Vecem.
- **Command id musí matchovat existující registry** — Override key je `"editor.save"`, `"editor.find"` atd. — musí odpovídat `Command.id` v `init_defaults()`. Neexistující id = warning + ignore.
- **`cargo check` + `./check.sh` musí projít** po každé změně.
- **Žádné nové runtime závislosti.**
- **Command palette dostává commands přes `to_vec()`** — při otevření palette se klonuje aktuální stav commands z registry. Override se musí aplikovat na registry *před* otevřením palette.
- **Ctrl+A/C/V/X/Z/Y nesmí být overridovatelné** — tyto klávesy patří TextEdit, centrální dispatch je nesmí interceptovat. Validace při parsování overrides.

## Common Pitfalls

- **Command palette label desync** — Pokud override změní binding jen v keymapu ale ne v Command.shortcut, palette zobrazí starý label. Řešení: override na CommandRegistry commands (single source), ne na keymapu přímo.
- **Init ordering** — Override se musí aplikovat PŘED vytvořením keymapy v `init_workspace()`. Pokud se keymapa vytvoří z default commands a override se aplikuje až potom, dispatch bude mít staré shortcuty.
- **Registry reset při `init_defaults()` re-call** — Pokud se `init_defaults()` zavolá znovu (např. při save), overrides se ztratí. Řešení: overrides aplikovat na commands *po* init_defaults, ne v init_defaults samotném. Aktuálně se init_defaults volá jen jednou při startu — ověřeno grepem.
- **Keymap binding duplikace** — User override `"editor.save" = "Ctrl+B"` by vytvořil konflikt s Build (Ctrl+B). `from_commands()` to nezjistí — obě se zaregistrují. `consume_shortcut` konzumuje event pro prvního matchera. Řešení: konflikt detekce při apply s warning logem.
- **Nevalidní shortcut string** — `parse_shortcut("abcd")` vrací None. Override s nevalidním stringem se musí tiše ignorovat (warning log), ne crash.
- **Prázdný override** — `"editor.save" = ""` by měl vymazat shortcut (ne crash). Parsovat jako None.

## Open Risks

- **Přemapování focus trojkombinací** — Uživatel může přemapovat `"ui.focus_build"` na "Ctrl+B" čímž zanikne Build shortcut (ten nemá explicitní override → default Ctrl+B zůstane). Oba commandy by měly Ctrl+B. Dispatch ordering (`modifier_count`) by mohl preferovat špatný. **Mitigace:** Conflict detection warning — ne hard error, ale uživatel je informován.
- **F1 alternativní binding** — F1 je registrován jako "ui.command_palette_f1" (samostatný command id). Pokud uživatel overridne "ui.command_palette" na jinou klávesu, F1 stále zůstane — to je pravděpodobně záměr (F1 je konvenční help/palette klávesa). Ale pokud by chtěl F1 zrušit, musel by overridnout "ui.command_palette_f1" na "" (prázdný string).
- **Settings draft vs. runtime keymap** — Mezi otevřením Settings dialogu a Save existuje draft stav. Keybindings v draftu se *nemají* live-preview aplikovat na keymapu (na rozdíl od theme, kde live preview funguje). Keymapa se aktualizuje až po Save.

## Requirements Coverage

### Active requirements owned by S03

- **R013 (Uživatelská konfigurace keybindings)** — Primární deliverable. `[keybindings]` sekce v settings.toml, parsing, merge s defaults, rebuild keymapy.
- **R015 (VS Code/JetBrains konvence)** — Partially validated (S01+S02 doplnily defaultní bindings). S03 dokončuje tím, že umožní uživatelskou konfigurovatelnost — uživatel přecházející z jiného editoru si může přemapovat zkratky.

### Validation plan

- R013: Unit test pro parse + apply override, roundtrip persistence test, backward compat test, conflict detection test. Grep ověření že settings.toml bez `[keybindings]` stále funguje.
- R015: Verified indirectly — konfigurovatelnost je prerequisite pro plnou VS Code/JetBrains kompatibilitu.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| egui / eframe | (žádný specifický skill) | none found — S01 research zjistil že `bobmatnyc/claude-mpm-skills@rust-desktop-applications` je příliš obecný |

## Sources

- `src/app/keymap.rs` — parse_shortcut, format_shortcut, Keymap::from_commands, dispatch, 13 unit testů
- `src/app/registry/mod.rs` — CommandRegistry, Command struct, init_defaults s 24 command registracemi
- `src/settings.rs` — Settings struct, serde TOML, TempConfigDir test pattern
- `src/app/ui/widgets/command_palette.rs:174-176` — palette čte cmd.shortcut přímo
- `src/app/ui/workspace/menubar/mod.rs:26-35` — shortcut_label() čte z keymap.bindings
- `src/app/ui/workspace/modal_dialogs/settings.rs:223-270` — save_settings_draft flow
- `src/app/ui/workspace/state/init.rs:48-51` — keymap init z registry
