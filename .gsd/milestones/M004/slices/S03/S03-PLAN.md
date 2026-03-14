# S03: Uživatelská konfigurace keybindings a dynamické labely

**Goal:** Uživatel může v `[keybindings]` sekci settings.toml přemapovat klávesové zkratky. Menu a command palette zobrazují aktuální (ne default) keybinding labely včetně uživatelských overrides.
**Demo:** Uživatel přidá `[keybindings]` sekci s `"editor.save" = "Ctrl+Shift+S"` → Ctrl+Shift+S uloží, menu zobrazí "Ctrl+Shift+S" místo "Ctrl+S". Chybějící `[keybindings]` sekce = default bindings (backward compat).

## Must-Haves

- `keybindings: HashMap<String, String>` field v `Settings` structu s `#[serde(default)]`
- `CommandRegistry::apply_keybinding_overrides()` přepíše `Command.shortcut` dle user overrides
- Overrides se aplikují v `init_workspace()` PŘED vytvořením keymapy
- Overrides se aplikují + keymapa se rebuildne v `save_settings_draft()` po uložení
- Nevalidní shortcut stringy (parse_shortcut vrátí None) se tiše ignorují (warning log)
- Prázdný override string (`""`) vymaže shortcut (Command.shortcut = None)
- Neexistující command id se tiše ignoruje (warning log)
- Konflikt detekce: dvě overrides na stejnou zkratku → warning log
- Ctrl+A/C/V/X/Z/Y nejsou overridovatelné (TextEdit reserved)
- Backward compat: settings.toml bez `[keybindings]` sekce funguje bez změny
- Menu shortcut labely reflektují uživatelské overrides (shortcut_label čte z rebuildnuté keymapy)
- Command palette shortcut labely reflektují overrides (palette čte Command.shortcut, které je overridden)
- Unit testy pokrývají: apply override, prázdný override, nevalidní override, neexistující id, conflict detection, backward compat

## Verification

- `cargo test --bin polycredo-editor app::registry` — nové unit testy pro apply_keybinding_overrides
- `./check.sh` — fmt + clippy + všechny testy projdou čistě
- `cargo test --bin polycredo-editor settings::tests` — backward compat test (deserializace bez `[keybindings]`)
- Failure-path check: nastavení nevalidního override (např. `"editor.save" = "Foo+Bar"`) nezmění default shortcut — `Command.shortcut` zůstane `Some(Ctrl+S)`. Ověřeno unit testem `test_apply_override_invalid_shortcut`.

## Observability / Diagnostics

- Runtime signals: `eprintln!` warning log pro nevalidní shortcut, neexistující command id, shortcut konflikt
- Inspection surfaces: `Keymap.bindings` je pub — inspektovatelný obsah keymapy za runtime; `Command.shortcut` je pub — inspektovatelný override stav
- Failure visibility: Nevalidní override se neprojeví (default shortcut zůstane) — uživatel vidí default label v menu/palette
- Redaction constraints: none

## Integration Closure

- Upstream surfaces consumed: `parse_shortcut()` z keymap.rs, `Keymap::from_commands()` z keymap.rs, `CommandRegistry` z registry/mod.rs, `Settings` z settings.rs, `save_settings_draft()` z settings dialog, `init_workspace()` z state/init.rs
- New wiring introduced in this slice: `apply_keybinding_overrides()` volaná v init_workspace a save_settings_draft, `keybindings` field v Settings
- What remains before the milestone is truly usable end-to-end: nothing — toto je poslední slice M004

## Tasks

- [x] **T01: Implementace keybinding overrides, rebuild keymapy a unit testy** `est:45m`
  - Why: Jediný deliverable slice — Settings field, override logika, wiring do init a save flow, testy. Scope je koherentní: data model → logika → napojení → testy.
  - Files: `src/settings.rs`, `src/app/registry/mod.rs`, `src/app/keymap.rs`, `src/app/ui/workspace/state/init.rs`, `src/app/ui/workspace/modal_dialogs/settings.rs`, `src/app/ui/workspace/state/mod.rs`
  - Do: (1) Přidat `keybindings: HashMap<String, String>` s `#[serde(default)]` do Settings. (2) Přidat `find_mut()` na CommandRegistry pro lookup dle id. (3) Implementovat `apply_keybinding_overrides()` na CommandRegistry — iteruje overrides, parsuje shortcut, validuje (reserved klávesy, neexistující id, konflikty), přepisuje Command.shortcut. (4) V init_workspace: po lock na shared, aplikovat overrides z settings PŘED Keymap::from_commands. (5) V save_settings_draft: po `s.settings = Arc::new(draft)`, aplikovat overrides na registry commands a rebuildnout keymapu. (6) Unit testy v registry/mod.rs: apply basic override, prázdný string override, nevalidní shortcut, neexistující id, conflict detection, reserved key rejection. (7) Backward compat test v settings.rs pro deserializaci bez [keybindings].
  - Verify: `cargo check` + `./check.sh` projde čistě, nové unit testy pass
  - Done when: `[keybindings]` overrides fungují při startu i po save, menu/palette labely reflektují overrides, backward compat zachována, všechny testy pass

## Files Likely Touched

- `src/settings.rs` — přidání keybindings field
- `src/app/registry/mod.rs` — find_mut, apply_keybinding_overrides, unit testy
- `src/app/keymap.rs` — případný helper (reserved keys list)
- `src/app/ui/workspace/state/init.rs` — override aplikace při startu
- `src/app/ui/workspace/state/mod.rs` — keymap field rebuild access
- `src/app/ui/workspace/modal_dialogs/settings.rs` — override + rebuild v save_settings_draft
