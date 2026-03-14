---
id: S03
parent: M004
milestone: M004
provides:
  - keybindings field v Settings s #[serde(default)] backward compat
  - CommandRegistry::find_mut() pro mutable lookup dle id
  - is_reserved_shortcut() helper — blokuje override TextEdit reserved kláves (Ctrl+A/C/V/X/Z/Y)
  - apply_keybinding_overrides() s validací (reserved, invalid, conflict, unknown id, empty string)
  - Init wiring — overrides aplikovány PŘED Keymap::from_commands v init_workspace()
  - Save wiring — re-init defaults + overrides + keymap rebuild v save_settings_draft()
  - Menu a command palette labely reflektují uživatelské overrides automaticky
  - 10 unit testů pro override logiku + 2 backward compat testy v settings.rs
requires:
  - slice: S01
    provides: Keymap dispatch, parse_shortcut(), Modifiers::COMMAND, Command.shortcut jako Option<KeyboardShortcut>
  - slice: S02
    provides: Rozšířený CommandId enum, funkční command palette, menu napojení na flagy
affects: []
key_files:
  - src/settings.rs
  - src/app/registry/mod.rs
  - src/app/keymap.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/workspace/modal_dialogs/settings.rs
key_decisions:
  - Override na CommandRegistry commands (single source of truth), ne na keymapu přímo — palette čte cmd.shortcut, menu čte z keymapy, obojí se aktualizuje automaticky
  - Re-init defaults před apply overrides v save_settings_draft — čistý stav + nové overrides zaručí konzistenci
  - Ctrl+A/C/V/X/Z/Y jako reserved keys neoverridovatelné — TextEdit je zpracuje sám
  - Konflikt detekce je warning-only (ne hard error) — obě overrides projdou, poslední vyhraje
  - Keybindings clone v save flow kvůli borrow checker (mutable borrow registry vs immutable settings)
patterns_established:
  - Keybinding override pattern: init_defaults() → apply_keybinding_overrides() → Keymap::from_commands()
  - Grepovatelné eprintln! warningy s [keybinding] prefixem pro diagnostiku
observability_surfaces:
  - "eprintln! warningy s [keybinding] prefixem: unknown command, invalid shortcut, reserved key, conflict"
  - "Command.shortcut (pub) — inspektovatelný override stav za runtime"
  - "Keymap.bindings (pub) — inspektovatelný obsah keymapy po rebuild"
drill_down_paths:
  - .gsd/milestones/M004/slices/S03/tasks/T01-SUMMARY.md
duration: 25m
verification_result: passed
completed_at: 2026-03-13
---

# S03: Uživatelská konfigurace keybindings a dynamické labely

**Uživatel může přemapovat klávesové zkratky přes `[keybindings]` sekci v settings.toml — menu a command palette labely reflektují overrides automaticky, backward compat zachována.**

## What Happened

Přidán `keybindings: HashMap<String, String>` field s `#[serde(default)]` do Settings structu, takže settings.toml bez `[keybindings]` sekce funguje beze změny (backward compat).

Hlavní logika v `CommandRegistry::apply_keybinding_overrides()` iteruje uživatelské overrides a validuje: neexistující command id → warning, nevalidní shortcut string → warning (default zůstane), reserved klávesy (Ctrl+A/C/V/X/Z/Y) → warning + odmítnutí, prázdný string → vymaže shortcut, konflikt (dvě overrides na stejnou zkratku) → warning. Helper `is_reserved_shortcut()` v keymap.rs detekuje TextEdit reserved klávesy.

Wiring: v `init_workspace()` se overrides aplikují PŘED `Keymap::from_commands()` — keymap se buildí z už overridnutých commands. V `save_settings_draft()` po uložení settings: `init_defaults()` obnoví defaulty, `apply_keybinding_overrides()` aplikuje nové overrides, `Keymap::from_commands()` rebuildne keymapu. Re-init defaults zajistí čistý stav bez akumulace starých overrides.

Menu a command palette labely reflektují overrides automaticky — menu shortcut label čte z keymapy, palette čte z `Command.shortcut`, obojí je overridden single-source-of-truth.

## Verification

- `cargo test --bin polycredo-editor app::registry` — 10/10 testů pass (find_mut, basic override, empty string, invalid, unknown id, reserved, conflict, reserved comprehensive, ctrl+shift not reserved)
- `cargo test --bin polycredo-editor settings::tests` — 36/36 testů pass (včetně backward compat + roundtrip)
- `./check.sh` — fmt + clippy + 172 testů: "All checks passed successfully!"
- `cargo check` — čistý
- Observability: 4 grepovatelné `[keybinding]` warningy v registry/mod.rs, `Command.shortcut` a `Keymap.bindings` pub fields pro runtime inspekci

## Requirements Advanced

- R013 — uživatelská konfigurace keybindings plně implementována a otestována
- R015 — uživatelská konfigurovatelnost zkratek doplňuje S01 dispatch + S02 konvence

## Requirements Validated

- R013 — `[keybindings]` sekce v settings.toml přemapuje zkratky, chybějící sekce = default bindings. 10 unit testů pokrývají validaci, backward compat test potvrzuje zpětnou kompatibilitu. Menu/palette labely reflektují overrides.
- R015 — plně validated: S01 centrální dispatch + S02 standardní zkratky (Ctrl+F/H/G/Shift+P/F1) + S03 uživatelská konfigurovatelnost. Uživatel může přemapovat jakoukoliv zkratku na jinou kombinaci.

## New Requirements Surfaced

- none

## Requirements Invalidated or Re-scoped

- none

## Deviations

None

## Known Limitations

- Konflikt detekce je warning-only — pokud uživatel přiřadí dvě akce na stejnou zkratku, obě projdou (poslední vyhraje). UI notifikace konfliktu není implementována.
- Nevalidní override se tiše ignoruje (default shortcut zůstane) — uživatel nemá UI feedback kromě toho, že label v menu/palette se nezmění.

## Follow-ups

- none — toto je poslední slice M004

## Files Created/Modified

- `src/settings.rs` — přidán `keybindings: HashMap<String, String>` field + Default + 2 testy
- `src/app/registry/mod.rs` — přidán `find_mut()`, `apply_keybinding_overrides()`, 10 unit testů
- `src/app/keymap.rs` — přidán `is_reserved_shortcut()` helper
- `src/app/ui/workspace/state/init.rs` — override aplikace před keymap build (mutable lock)
- `src/app/ui/workspace/modal_dialogs/settings.rs` — re-init + override + keymap rebuild v save flow

## Forward Intelligence

### What the next slice should know
- M004 je kompletní — centrální keymap dispatch, exkluzivní modifier matching, chybějící handlery, command palette, uživatelská konfigurace. Všech 6 requirements (R010–R015) validated.
- Keybinding override pattern (init_defaults → apply_overrides → Keymap::from_commands) je stabilní a znovupoužitelný pro budoucí rozšíření.

### What's fragile
- Conflict detection je HashMap-based (assigned tracking) — pokud by se přidaly multi-binding commands (víc klávesových zkratek pro jednu akci), conflict detection by potřebovala revizi.
- Re-init defaults v save flow (init_defaults() → apply_overrides) závisí na tom, že init_defaults nastaví kompletní sadu defaultních zkratek — pokud se přidá nový command bez default shortcut, override flow to zpracuje správně (find_mut vrátí Some, shortcut se nastaví).

### Authoritative diagnostics
- `grep -rn '\[keybinding\]' src/` — najde všechny warning log body pro diagnostiku override problémů
- `cargo test --bin polycredo-editor app::registry` — 10 testů pokrývají celou override validaci

### What assumptions changed
- none — slice proběhl přesně dle plánu
