---
estimated_steps: 7
estimated_files: 6
---

# T01: Implementace keybinding overrides, rebuild keymapy a unit testy

**Slice:** S03 — Uživatelská konfigurace keybindings a dynamické labely
**Milestone:** M004

## Description

Přidání `[keybindings]` sekce do settings.toml pro uživatelské přemapování klávesových zkratek. Override se aplikuje na `CommandRegistry` commands (single source of truth), pak se keymapa rebuildne z aktualizovaných commands. Menu i command palette automaticky reflektují overrides díky existující infrastruktuře (`shortcut_label()` čte z keymapy, palette čte `cmd.shortcut`).

## Steps

1. **Settings field** — přidat `pub keybindings: HashMap<String, String>` s `#[serde(default)]` do `Settings` structu v `src/settings.rs`. Import `HashMap` pokud chybí. Přidat backward compat unit test: deserializace TOML bez `[keybindings]` sekce musí projít.

2. **CommandRegistry::find_mut** — přidat `pub fn find_mut(&mut self, id: &str) -> Option<&mut Command>` na `CommandRegistry` v `src/app/registry/mod.rs`. Lookup přes `by_id` HashMap, vrátí mutable reference na Command.

3. **Reserved keys helper** — přidat `pub fn is_reserved_shortcut(shortcut: &KeyboardShortcut) -> bool` do `src/app/keymap.rs`. Kontroluje jestli shortcut odpovídá Ctrl+A/C/V/X/Z/Y (TextEdit reserved). Tyto klávesy nesmí být overridovány.

4. **apply_keybinding_overrides** — implementovat `pub fn apply_keybinding_overrides(&mut self, overrides: &HashMap<String, String>)` na `CommandRegistry` v `src/app/registry/mod.rs`:
   - Pro každý (id, shortcut_str) v overrides:
     - Pokud id neexistuje v registry → `eprintln!` warning, skip
     - Pokud shortcut_str je prázdný → nastavit `cmd.shortcut = None` (vymazání zkratky)
     - Parsovat přes `parse_shortcut()` → pokud None → warning, skip
     - Kontrola `is_reserved_shortcut()` → pokud true → warning, skip
     - Kontrola konfliktu: stejná zkratka už přiřazena jinému commandu → warning (ne hard error)
     - Přepsat `cmd.shortcut = Some(parsed)`

5. **Init wiring** — v `src/app/ui/workspace/state/init.rs` v `init_workspace()`: po lock na shared, aplikovat overrides na registry commands *před* vytvořením keymapy:
   ```
   sh.registry.commands.apply_keybinding_overrides(&settings.keybindings);
   Keymap::from_commands(sh.registry.commands.get_all())
   ```

6. **Save wiring** — v `src/app/ui/workspace/modal_dialogs/settings.rs` v `save_settings_draft()`: po `s.settings = Arc::new(draft)` a agent registry update:
   - Re-init defaults: `s.registry.commands = CommandRegistry::new(); s.registry.init_defaults();`
   - Aplikovat overrides: `s.registry.commands.apply_keybinding_overrides(&s.settings.keybindings);`
   - Na workspace (ws): `ws.keymap = Keymap::from_commands(s.registry.commands.get_all());`
   Pozn: re-init defaults je nutný protože předchozí overrides mohly změnit command state — chceme čistý stav + nové overrides.

7. **Unit testy** — v `src/app/registry/mod.rs` (existující test modul nebo nový `#[cfg(test)] mod tests`):
   - `test_apply_override_basic` — override "editor.save" na "Ctrl+Shift+S", ověřit Command.shortcut
   - `test_apply_override_empty_string` — override na "" vymaže shortcut
   - `test_apply_override_invalid_shortcut` — override na "Foo+Bar" se ignoruje, shortcut zůstane default
   - `test_apply_override_unknown_id` — override neexistujícího id se ignoruje
   - `test_apply_override_reserved_key` — override na "Ctrl+C" se odmítne
   - `test_apply_override_conflict_detection` — dvě overrides na stejnou zkratku, obě projdou ale warning (ověřit stav, ne warning output)
   - Backward compat test v settings.rs: deserializace `[editor_font_size]\nfont_size = 14` bez keybindings → keybindings je prázdná HashMap

## Must-Haves

- [ ] `keybindings` field v Settings s `#[serde(default)]` — backward compat
- [ ] `find_mut()` na CommandRegistry
- [ ] `is_reserved_shortcut()` helper v keymap.rs
- [ ] `apply_keybinding_overrides()` na CommandRegistry s validací
- [ ] Override aplikován v init_workspace PŘED keymap build
- [ ] Override + keymap rebuild v save_settings_draft PO uložení
- [ ] 6+ unit testů pro override logiku
- [ ] Backward compat test pro Settings deserializaci
- [ ] `cargo check` + `./check.sh` projde čistě

## Verification

- `cargo test --bin polycredo-editor app::registry` — nové override testy pass
- `cargo test --bin polycredo-editor settings::tests` — backward compat test pass
- `./check.sh` — fmt + clippy + všechny testy pass ("All checks passed successfully!")
- `cargo check` — čistý

## Observability Impact

- **New runtime signals:** `eprintln!` warnings pro nevalidní shortcut string, neexistující command id, reserved key override pokus a shortcut konflikt. Všechny obsahují `[keybinding]` prefix pro grepovatelnost.
- **Inspectable state:** `Command.shortcut` (pub) — po apply_keybinding_overrides reflektuje uživatelský override nebo None (pokud override na ""). `Keymap.bindings` (pub) — rebuildnutá keymapa obsahuje overridden zkratky.
- **Failure visibility:** Nevalidní override se tiše ignoruje (default shortcut zůstane). Uživatel vidí default label v menu/palette — žádný crash ani chybný stav. Agent může ověřit přes `CommandRegistry::find()` + inspekce `cmd.shortcut`.
- **Diagnostic grep patterns:** `[keybinding] unknown command`, `[keybinding] invalid shortcut`, `[keybinding] reserved key`, `[keybinding] conflict`

## Inputs

- `src/app/keymap.rs` — parse_shortcut(), format_shortcut(), Keymap::from_commands()
- `src/app/registry/mod.rs` — CommandRegistry, Command struct, init_defaults()
- `src/settings.rs` — Settings struct se serde pattern
- `src/app/ui/workspace/state/init.rs` — init_workspace() s existujícím keymap buildem
- `src/app/ui/workspace/modal_dialogs/settings.rs` — save_settings_draft() flow
- S01/S02 summary — architektonické rozhodnutí a fragile body

## Expected Output

- `src/settings.rs` — Settings struct rozšířen o `keybindings` field
- `src/app/registry/mod.rs` — `find_mut()`, `apply_keybinding_overrides()`, 6+ unit testů
- `src/app/keymap.rs` — `is_reserved_shortcut()` helper
- `src/app/ui/workspace/state/init.rs` — override aplikace při startu
- `src/app/ui/workspace/modal_dialogs/settings.rs` — override + rebuild při save
- Všechny existující i nové testy projdou
