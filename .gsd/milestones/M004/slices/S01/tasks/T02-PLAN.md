---
estimated_steps: 7
estimated_files: 6
---

# T02: Napojení dispatch na workspace — nahrazení ad-hoc handlerů

**Slice:** S01 — Centrální keymap dispatch a oprava modifier filtrování
**Milestone:** M004

## Description

Propojit `Keymap` dispatch s existujícím command pipeline ve workspace. Smazat ad-hoc keyboard handlery z `workspace/mod.rs` a nahradit je centrálním dispatch voláním. Aktualizovat menu shortcut labely, aby se generovaly z `KeyboardShortcut` dat místo hardcoded stringů. Rozšířit `execute_command()` o handling nových FocusEditor/FocusBuild/FocusClaude commandů.

## Steps

1. **Přidat `Keymap` field do `WorkspaceState`** v `state/mod.rs`. Inicializovat z `CommandRegistry` při vytváření workspace (v `init_workspace` nebo ekvivalentu).

2. **Rozšířit `execute_command()`** v `command_palette.rs` o nové CommandId varianty:
   - `FocusEditor` → nastavit `focused_panel = FocusedPanel::Editor`, volat `request_editor_focus()`
   - `FocusBuild` → nastavit `show_build_terminal = true`, `focused_panel = FocusedPanel::Build`
   - `FocusClaude` → nastavit `show_right_panel = true`, `focused_panel = FocusedPanel::Claude`
   - Tyto akce potřebují přímý přístup k `WorkspaceState`, ne přes `MenuActions` — buď přidat nové flagy do MenuActions, nebo refaktorovat execute_command tak, aby měl &mut WorkspaceState.

3. **Přidat dispatch volání do `render_workspace()`** v `workspace/mod.rs`:
   - Po kontrole focus/minimized stavu (řádek ~460) a PŘED widget renderingem
   - `let dispatch_result = ctx.input_mut(|input| ws.keymap.dispatch(input));`
   - Pokud `Some(cmd_id)` → `execute_command(CommandAction::Internal(cmd_id), &mut actions, shared)` → `process_menu_actions()`
   - Pozor: dispatch musí být v `input_mut()` closure kvůli `consume_shortcut`

4. **Smazat ad-hoc keyboard handlery** z `workspace/mod.rs`:
   - Řádky 508-539: Ctrl+Alt+E/B/A (fokus panelů), Ctrl+S (save), Ctrl+B (build), Ctrl+R (run)
   - Funkci `consume_close_tab_shortcut()` (řádky 77-84) smazat — Ctrl+W je teď v keymapu
   - Volání `consume_close_tab_shortcut(ctx)` nahradit dispatch výsledkem

5. **Aktualizovat menu shortcut labely** v menubar souborech:
   - `edit.rs`: `.shortcut_text("Ctrl+B")` → `.shortcut_text(format_shortcut(&shortcut))` kde shortcut se vezme z CommandRegistry nebo Keymap
   - `file.rs`: totéž pro Ctrl+S, Ctrl+W, Ctrl+,
   - Alternativně: předat shortcut labely jako parametr z `render_menu_bar()` kde je přístup ke keymapu
   - Jednodušší varianta pro S01: helper funkce v menubar/mod.rs, která lookupne shortcut z keymapu a vrátí string

6. **Ověřit, že Ctrl+A/C/V/X/Z/Y nejsou v keymapu** — grep přes init_defaults a keymap bindings. Edit menu je renderuje jako disabled s hardcoded labely (to je OK — TextEdit je zpracuje sám).

7. **Spustit `./check.sh`** a ověřit:
   - `cargo fmt --check` čistý
   - `cargo clippy` bez warningů
   - `cargo test` — všechny testy (195+) plus nové keymap testy projdou
   - `grep "i.modifiers.ctrl" src/app/ui/workspace/mod.rs` vrátí 0 výsledků

## Must-Haves

- [ ] `Keymap` inicializovaný v `WorkspaceState`
- [ ] Dispatch volaný v `render_workspace()` před widget renderingem
- [ ] Ad-hoc keyboard handlery (workspace/mod.rs:508-539) kompletně smazány
- [ ] `consume_close_tab_shortcut()` smazána — Ctrl+W přes keymap
- [ ] FocusEditor/FocusBuild/FocusClaude handling v execute_command
- [ ] Menu shortcut labely generované z KeyboardShortcut dat
- [ ] `./check.sh` projde čistě

## Verification

- `./check.sh` projde (fmt + clippy + testy)
- `grep -c "i.modifiers.ctrl" src/app/ui/workspace/mod.rs` vrátí `0`
- `grep -c "consume_close_tab_shortcut" src/app/ui/workspace/mod.rs` vrátí `0`
- Menu shortcut labely v edit.rs/file.rs neobsahují hardcoded "Ctrl+" stringy pro commandy v keymapu

## Observability Impact

- **Dispatch signál v render_workspace:** `Keymap::dispatch()` vrací `Option<CommandId>` — `None` = žádná zkratka stisknuta, `Some(id)` = zkratka matchla a byla zpracována. Tento výstup je přímo propojený do `execute_command()`.
- **Grep diagnostika:** `grep "i.modifiers.ctrl" src/app/ui/workspace/mod.rs` musí vracet 0 výsledků — všechny ad-hoc handlery jsou odstraněny.
- **Grep diagnostika:** `grep "consume_close_tab_shortcut" src/app/ui/workspace/mod.rs` musí vracet 0 výsledků — Ctrl+W je přes keymap.
- **Menu labely:** shortcut labely v menu se generují z `Keymap` dat přes `format_shortcut()` — změna v keymapu se automaticky projeví v menu.
- **Failure visibility:** pokud dispatch nenapojí command správně, výsledek se projeví jako nefunkční zkratka — viditelné při manuálním testování i přes unit testy.

## Inputs

- `src/app/keymap.rs` — T01 výstup: Keymap struct, dispatch, format_shortcut
- `src/app/registry/mod.rs` — T01 výstup: Command.shortcut jako KeyboardShortcut
- `src/app/ui/widgets/command_palette.rs` — T01 výstup: nové CommandId varianty

## Expected Output

- `src/app/ui/workspace/mod.rs` — bez ad-hoc keyboard handlerů, s centrálním dispatch
- `src/app/ui/workspace/menubar/mod.rs` — shortcut label lookup z keymapu
- `src/app/ui/workspace/menubar/edit.rs` — dynamické shortcut labely
- `src/app/ui/workspace/menubar/file.rs` — dynamické shortcut labely
- `src/app/ui/workspace/state/mod.rs` — Keymap field ve WorkspaceState
- `src/app/ui/widgets/command_palette.rs` — rozšířený execute_command
