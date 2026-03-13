---
id: T02
parent: S01
milestone: M004
provides:
  - Centrální keymap dispatch v render_workspace() nahrazující ad-hoc keyboard handlery
  - Focus panel handling (FocusEditor/FocusBuild/FocusClaude) přes MenuActions pipeline
  - Dynamické menu shortcut labely generované z Keymap dat
  - Keymap inicializace v WorkspaceState z CommandRegistry
key_files:
  - src/app/ui/workspace/mod.rs
  - src/app/ui/workspace/state/mod.rs
  - src/app/ui/workspace/state/init.rs
  - src/app/ui/workspace/menubar/mod.rs
  - src/app/ui/workspace/menubar/edit.rs
  - src/app/ui/workspace/menubar/file.rs
  - src/app/ui/widgets/command_palette.rs
key_decisions:
  - Focus panelů (FocusEditor/FocusBuild/FocusClaude) implementovány přes MenuActions flagy + process_menu_actions, ne přes přímý &mut WorkspaceState v execute_command — zachovává existující pipeline
  - Keymap dispatch umístěn PŘED widget renderingem v render_workspace() — consume_shortcut konzumuje event, musí být první
  - Clipboard zkratky (Ctrl+C/V/A) a editor-interní zkratky (Ctrl+F/H/G) ponechány s hardcoded labely — nejsou v command registry a zpracovává je TextEdit/editor sám
patterns_established:
  - shortcut_label(keymap, CommandId) helper pro lookup shortcut labelů z keymapu
  - Keymap field v WorkspaceState inicializovaný z CommandRegistry při vytváření workspace
observability_surfaces:
  - "grep -c 'i.modifiers.ctrl' src/app/ui/workspace/mod.rs" musí vracet 0 — žádné ad-hoc handlery
  - "grep -c 'consume_close_tab_shortcut' src/app/ui/workspace/mod.rs" musí vracet 0
  - Keymap::dispatch() vrací Option<CommandId> — None/Some jako jasný signál
  - Menu labely se generují z Keymap dat — změna keymapu se automaticky projeví v menu
duration: 20m
verification_result: passed
completed_at: 2026-03-13
blocker_discovered: false
---

# T02: Napojení dispatch na workspace — nahrazení ad-hoc handlerů

**Centrální keymap dispatch napojen na workspace pipeline. Ad-hoc keyboard handlery (Ctrl+S/B/R/W, Ctrl+Alt+E/B/A) smazány z workspace/mod.rs. Menu shortcut labely generovány dynamicky z Keymap dat. Focus panel commandy plně funkční přes MenuActions.**

## What Happened

1. Přidán `keymap: Keymap` field do `WorkspaceState` v `state/mod.rs`. Inicializace z `CommandRegistry` přes `Keymap::from_commands()` v `init_workspace()` — umístěna před `spawn_semantic_indexer()` kvůli ownership `shared` Arc.

2. Rozšířen `execute_command()` v `command_palette.rs` — `FocusEditor/FocusBuild/FocusClaude` nyní nastavují nové `MenuActions` flagy (`focus_editor`, `focus_build`, `focus_claude`).

3. V `process_menu_actions()` v `menubar/mod.rs` přidán handling pro fokus flagy:
   - `focus_editor` → nastaví `FocusedPanel::Editor` + `request_editor_focus()`
   - `focus_build` → nastaví `show_build_terminal = true` + `FocusedPanel::Build`
   - `focus_claude` → nastaví `show_right_panel = true` + `FocusedPanel::Claude`

4. V `render_workspace()` nahrazen celý blok ad-hoc keyboard shortcutů (řádky 508-539) centrálním dispatch voláním:
   ```rust
   if let Some(cmd_id) = ctx.input_mut(|input| ws.keymap.dispatch(input)) {
       // → execute_command → process_menu_actions
   }
   ```

5. Smazána `consume_close_tab_shortcut()` funkce — Ctrl+W je nyní v keymapu.

6. Vytvořena `shortcut_label()` helper funkce v `menubar/mod.rs` pro lookup shortcut labelů z keymapu.

7. `file.rs` a `edit.rs` přepsány — shortcut labely pro commandy v keymapu (Save, CloseTab, Settings, Build, Run, OpenFile, ProjectSearch) se generují dynamicky přes `shortcut_label()`. Clipboard a editor-interní zkratky ponechány s hardcoded labely.

8. Test `unsaved_close_guard_ctrl_w_consumes_shortcut` přepsán na `unsaved_close_guard_ctrl_w_via_keymap_dispatch` — testuje Ctrl+W přes Keymap dispatch místo smazané funkce.

9. Dva testovací WorkspaceState literály v `app/mod.rs` doplněny o `keymap` field (prázdná keymapa pro testy).

## Verification

- `./check.sh` — fmt + clippy + testy (156 testů) projdou čistě ✅
- `grep -c "i.modifiers.ctrl" src/app/ui/workspace/mod.rs` → `0` ✅
- `grep -c "consume_close_tab_shortcut" src/app/ui/workspace/mod.rs` → `0` ✅
- Menu shortcut labely v edit.rs/file.rs neobsahují hardcoded "Ctrl+" pro commandy v keymapu ✅
- Ctrl+A/C/V/X/Z/Y nejsou v keymapu — ověřeno grepem ✅
- Test `unsaved_close_guard_ctrl_w_via_keymap_dispatch` ověřuje Ctrl+W dispatch + event consumption ✅

### Slice verification status (T02 — 2. z 2 tasků, finální):
- ✅ `cargo test --all-targets --all-features` — 156 testů projde
- ✅ `./check.sh` — fmt + clippy + testy bez warningů
- ✅ Dispatch ordering test: trojkombinace matchne před dvoukombinací (test_dispatch_ordering)
- ✅ parse_shortcut("Ctrl+Shift+F") vrátí správný KeyboardShortcut
- ✅ format_shortcut vrátí "Ctrl+Shift+F" na Linuxu
- ✅ Žádný ad-hoc `ctx.input(|i| i.modifiers.ctrl && ...)` v workspace/mod.rs (grep vrací 0)
- ✅ Diagnostické testy projdou (parse_shortcut_invalid → None, empty_keymap_dispatch → None)

## Diagnostics

- `grep -c "i.modifiers.ctrl" src/app/ui/workspace/mod.rs` — musí být 0
- `grep -c "consume_close_tab_shortcut" src/app/ui/workspace/mod.rs` — musí být 0
- `cargo test keymap` — spustí 9 keymap unit testů
- `cargo test unsaved_close_guard_ctrl_w` — ověří Ctrl+W dispatch přes keymap
- `Keymap.bindings` je pub — inspektovatelný obsah keymapy za runtime

## Deviations

- `consume_close_tab_shortcut()` test přepsán na Keymap dispatch verzi místo prostého smazání — lepší diagnostické pokrytí.
- Focus panel handling implementován přes MenuActions flagy místo přímého `&mut WorkspaceState` v `execute_command()` — čistší architektura, zachovává existující pipeline.
- Odstraněn nepoužívaný import `run_build_check` z workspace/mod.rs (dispatch to teď řeší přes MenuActions).

## Known Issues

- Žádné.

## Files Created/Modified

- `src/app/ui/workspace/mod.rs` — smazány ad-hoc keyboard handlery, přidán centrální keymap dispatch, smazána consume_close_tab_shortcut()
- `src/app/ui/workspace/state/mod.rs` — přidán `keymap: Keymap` field do WorkspaceState
- `src/app/ui/workspace/state/init.rs` — inicializace keymapy z CommandRegistry
- `src/app/ui/workspace/menubar/mod.rs` — přidány focus_editor/focus_build/focus_claude do MenuActions, shortcut_label() helper, handling fokus flagů v process_menu_actions()
- `src/app/ui/workspace/menubar/edit.rs` — dynamické shortcut labely pro commandy v keymapu
- `src/app/ui/workspace/menubar/file.rs` — dynamické shortcut labely pro commandy v keymapu
- `src/app/ui/widgets/command_palette.rs` — FocusEditor/FocusBuild/FocusClaude nastavují MenuActions flagy
- `src/app/ui/workspace/tests/unsaved_close_guard.rs` — test přepsán na Keymap dispatch verzi
- `src/app/mod.rs` — testovací WorkspaceState literály doplněny o keymap field
- `.gsd/milestones/M004/slices/S01/tasks/T02-PLAN.md` — přidána sekce Observability Impact
