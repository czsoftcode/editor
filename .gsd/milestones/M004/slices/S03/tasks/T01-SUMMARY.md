---
id: T01
parent: S03
milestone: M004
provides:
  - keybindings field v Settings s serde backward compat
  - CommandRegistry::find_mut() pro mutable lookup
  - is_reserved_shortcut() helper pro TextEdit reserved kláves
  - apply_keybinding_overrides() s validací (reserved, invalid, conflict, unknown id)
  - init wiring — overrides aplikovány PŘED keymap build
  - save wiring — re-init defaults + overrides + keymap rebuild po save
  - 10 unit testů pro override logiku + 2 backward compat testy
key_files:
  - src/settings.rs
  - src/app/registry/mod.rs
  - src/app/keymap.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/workspace/modal_dialogs/settings.rs
key_decisions:
  - Reserved klávesy = Ctrl+A/C/V/X/Z/Y (přesně Modifiers::COMMAND, bez dalších modifikátorů). Ctrl+Shift+C NENÍ reserved.
  - Konflikt detekce je warning-only (ne hard error) — obě overrides projdou, poslední vyhraje díky HashMap iteraci
  - Re-init defaults v save flow je nutný pro čistý stav před aplikací nových overrides
  - Keybindings clone v save flow kvůli borrow checker (mutable borrow na registry vs immutable na settings)
patterns_established:
  - Keybinding override pattern: init_defaults() → apply_keybinding_overrides() → Keymap::from_commands()
  - Grepovatelné eprintln! warningy s [keybinding] prefixem
observability_surfaces:
  - "eprintln! warningy s [keybinding] prefixem: unknown command, invalid shortcut, reserved key, conflict"
  - "Command.shortcut (pub) — inspektovatelný override stav za runtime"
  - "Keymap.bindings (pub) — inspektovatelný obsah keymapy po rebuild"
duration: 25m
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T01: Implementace keybinding overrides, rebuild keymapy a unit testy

**Přidáno uživatelské přemapování klávesových zkratek přes `[keybindings]` sekci v settings.toml s validací, wiring do init/save flow a 12 unit testy.**

## What Happened

1. **Settings field** — přidán `pub keybindings: HashMap<String, String>` s `#[serde(default)]` do Settings structu. Import `HashMap` přidán. Backward compat test + roundtrip test přidány.

2. **CommandRegistry::find_mut** — přidána metoda pro mutable lookup dle id přes `by_id` HashMap.

3. **is_reserved_shortcut** — helper v keymap.rs kontroluje Ctrl+A/C/V/X/Z/Y (přesně COMMAND modifikátor bez dalších). Ctrl+Shift+C není reserved.

4. **apply_keybinding_overrides** — metoda na CommandRegistry iteruje overrides, validuje (neexistující id, prázdný string, nevalidní parse, reserved klávesy, konflikty) a přepisuje Command.shortcut. Conflict assigned tracking pro detekci duplicit.

5. **Init wiring** — v init_workspace() lock na shared je nyní `let mut sh` pro mutable přístup. Overrides se aplikují PŘED Keymap::from_commands.

6. **Save wiring** — v save_settings_draft() po uložení settings: re-init defaults → apply overrides → rebuild keymap. Keybindings clone kvůli borrow checker.

7. **Unit testy** — 10 testů v registry/mod.rs (find_mut, basic override, empty string, invalid, unknown id, reserved, conflict, reserved comprehensive, ctrl+shift not reserved) + 2 testy v settings.rs (backward compat no keybindings, roundtrip).

## Verification

- `cargo test --bin polycredo-editor app::registry` — 10 passed ✅
- `cargo test --bin polycredo-editor settings::tests` — 36 passed (včetně 2 nových) ✅
- `./check.sh` — fmt + clippy + 172 testů: "All checks passed successfully!" ✅
- `cargo check` — čistý ✅

## Diagnostics

- `grep -r "\[keybinding\]"` — najde všechny warning log body
- `CommandRegistry::find("editor.save").unwrap().shortcut` — inspekce override stavu
- `Keymap.bindings` — inspekce rebuildnuté keymapy
- Nevalidní override se tiše ignoruje (default shortcut zůstane, label v menu/palette nezměněn)

## Deviations

None

## Known Issues

None

## Files Created/Modified

- `src/settings.rs` — přidán `keybindings: HashMap<String, String>` field + Default + 2 testy
- `src/app/registry/mod.rs` — přidán `find_mut()`, `apply_keybinding_overrides()`, importy, 10 unit testů
- `src/app/keymap.rs` — přidán `is_reserved_shortcut()` helper
- `src/app/ui/workspace/state/init.rs` — override aplikace před keymap build (mutable lock)
- `src/app/ui/workspace/modal_dialogs/settings.rs` — re-init + override + keymap rebuild v save flow
- `.gsd/milestones/M004/slices/S03/S03-PLAN.md` — přidán failure-path verification check
- `.gsd/milestones/M004/slices/S03/tasks/T01-PLAN.md` — přidána Observability Impact sekce
