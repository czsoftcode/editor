# Phase 10: UI & State Cleanup - Research

**Researched:** 2026-03-05
**Domain:** Rust/egui UI code removal, dead code cleanup
**Confidence:** HIGH

## Summary

Phase 10 is a focused deletion phase -- removing sandbox UI elements from settings dialog, file tree, build bar, and modal dialogs. The codebase is already partially cleaned by Phase 9 (WorkspaceState, types.rs, ToastActionKind are sandbox-free). What remains are UI rendering remnants and one dead module file (`sandbox.rs` modal).

The work is straightforward: delete code blocks, remove parameters, and fix compilation. The file tree `is_sandbox` parameter is always passed as `false` from the single caller in `panels.rs`, so removing it is safe. The line count / large file highlighting logic currently gated behind `is_sandbox` should become unconditional (global feature).

**Primary recommendation:** Delete sandbox UI blocks in order of dependency depth (modal file first, then settings/build_bar, then file_tree parameter chain), compile after each step.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Settings dialog: Smazat celý sandbox blok v kategorii "general" (řádky 272-294): separator, disabled checkbox stub, tooltip, hint text a terminal note
- Kategorii "general" ponechat -- po odstranění zůstane výběr jazyka + default project path
- I18n klíče (settings-safe-mode, settings-safe-mode-tooltip, settings-safe-mode-hint, settings-safe-mode-terminal-note) ponechat pro Phase 12
- File tree: Odebrat parametr `is_sandbox` z `render()` a `render_node()`. Lazy line count a zvýraznění velkých souborů (500+ řádků) ponechat jako globální feature
- Odebrat podmínku `if is_sandbox` a nechat logiku běžet pro všechny soubory
- Žádný dynamický label "Soubory (Sandbox)" neexistuje -- nic k odstranění
- Modal dialogy: Smazat celý soubor `modal_dialogs/sandbox.rs` (80 řádků, 100% sandbox-only)
- Odebrat TODO komentář a `mod sandbox` deklaraci z `modal_dialogs.rs`
- Build bar: Smazat celý label "Terminal" i s hover textem `hover-build-sandbox`
- State: WorkspaceState a types.rs jsou už čisté (Phase 9). V gitignore filtru odebrat jen `"sandbox"` podmínku, ponechat `.polycredo` filtr
- Pořadí a kompilační chyby jsou Claude's Discretion

### Deferred Ideas (OUT OF SCOPE)
- I18n klíče sandbox -- Phase 12
- Runtime cleanup .polycredo/sandbox/ adresářů -- Phase 12
- File operations sandbox remapping -- Phase 11
- Watcher sandbox logika -- Phase 11
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| UI-01 | Settings toggle pro sandbox mode odstraněn ze Settings dialogu | Settings.rs lines 272-294: celý blok k smazání (separator, checkbox stub, tooltip, hint, terminal note) |
| UI-02 | Sandbox sync confirmation modal odstraněn | Soubor `modal_dialogs/sandbox.rs` k smazání + mod deklarace v `modal_dialogs.rs` |
| UI-03 | Sandbox OFF confirmation dialog odstraněn ze settings | Součást bloku 272-294 v settings.rs (stub checkbox) |
| UI-04 | File tree "Sandbox" toggle button a label odstraněny | Parametr `is_sandbox` v `file_tree/mod.rs` a `render.rs` -- žádný toggle/label neexistuje, jen parametr |
| UI-05 | Build bar "Sandbox ON/OFF" indikátor odstraněn | `build_bar.rs` řádky 14-16: "Terminal" label + hover text |
| UI-06 | Toast akce odstraněny | Již hotovo v Phase 9 (ToastActionKind odstraněn) |
| STATE-01 | Sandbox fieldy z WorkspaceState odstraněny | Již hotovo v Phase 9 -- žádné sandbox reference v state/mod.rs |
| STATE-02 | SandboxApplyRequest etc. odstraněny | Již hotovo v Phase 9 -- žádné sandbox reference v types.rs |
| STATE-03 | ToastActionKind varianty odstraněny | Již hotovo v Phase 9 |
| STATE-04 | AppShared.sandbox_off_toast_shown odstraněno | Již hotovo v Phase 9 |
</phase_requirements>

## Architecture Patterns

### Deletion Strategy (from Phase 9)
Phase 9 established the pattern: aggressive deletion followed by compilation fix. Same approach applies here.

**Order of operations:**
1. Delete standalone files first (sandbox.rs modal -- has `#[cfg(never)]` so won't cause compile errors)
2. Remove references to deleted files (mod declarations, use statements)
3. Remove UI code blocks (settings sandbox block, build bar label)
4. Remove function parameters (file_tree `is_sandbox` chain)
5. Modify gitignore filter logic
6. Compile and fix

### File Tree Parameter Removal Chain
The `is_sandbox` parameter flows through 3 points:
1. **Caller**: `panels.rs:48` -- `ws.file_tree.ui(ui, i18n, false)` -- remove 3rd arg
2. **Method**: `file_tree/mod.rs:92-96` -- `pub fn ui(..., is_sandbox: bool)` -- remove param
3. **Inner call**: `file_tree/mod.rs:125` -- `Self::show_node(..., is_sandbox)` -- remove arg
4. **Renderer**: `file_tree/render.rs:31` -- `pub fn show_node(..., is_sandbox: bool)` -- remove param
5. **Recursive**: `file_tree/render.rs:61` -- recursive `show_node` call -- remove arg

### Line Count Logic Transformation
Currently gated by `is_sandbox`:
- `render.rs:115`: `if is_sandbox && node.line_count.is_none()` -- remove `is_sandbox &&`
- `render.rs:132`: `if is_sandbox && let Some(count)...` -- remove `is_sandbox &&`

This makes lazy line counting and large-file highlighting a global feature for all files.

### Anti-Patterns to Avoid
- **Don't remove i18n keys** -- they are explicitly deferred to Phase 12. The code will have orphaned i18n keys and that's intentional.
- **Don't touch editor/files.rs sandbox references** -- Phase 11 scope.
- **Don't touch terminal/mod.rs sandbox functions** -- Phase 11 scope.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Compile error tracking | Manual grep for errors | `cargo check 2>&1` | Compiler catches all broken references |

## Common Pitfalls

### Pitfall 1: Removing too much from gitignore filter
**What goes wrong:** Removing the entire `if` condition instead of just the `sandbox` part
**Why it happens:** The line `if line.contains(".polycredo") || line.contains("sandbox")` has two conditions
**How to avoid:** Only remove `|| line.contains("sandbox")` -- keep `.polycredo` filter
**Result:** `if line.contains(".polycredo") { continue; }`

### Pitfall 2: Breaking file tree recursive calls
**What goes wrong:** Removing `is_sandbox` from outer call but not inner recursive call
**Why it happens:** `show_node` calls itself recursively at line 61
**How to avoid:** Remove `is_sandbox` from all 5 points in the chain simultaneously

### Pitfall 3: Scope creep into Phase 11/12 territory
**What goes wrong:** Touching `editor/files.rs`, `terminal/mod.rs`, `editor/ui.rs` sandbox references
**Why it happens:** Grep shows many sandbox references across the UI codebase
**How to avoid:** Strict scope: only settings.rs, sandbox.rs modal, build_bar.rs, file_tree, modal_dialogs.rs, state/init.rs

## Code Examples

### Settings dialog: Block to remove (lines 272-294)
```rust
// DELETE everything from line 272 to 294 inclusive:
// ui.separator();
// ui.add_space(10.0);
// let sandbox_mode_row = ui.allocate_ui_with_layout(...)
// ...through...
// ui.label(i18n.get("settings-safe-mode-terminal-note"));
```

### File tree: Parameter removal
```rust
// BEFORE (mod.rs):
pub fn ui(&mut self, ui: &mut eframe::egui::Ui, i18n: &crate::i18n::I18n, is_sandbox: bool) -> FileTreeResult {

// AFTER:
pub fn ui(&mut self, ui: &mut eframe::egui::Ui, i18n: &crate::i18n::I18n) -> FileTreeResult {
```

### File tree: Line count becomes global
```rust
// BEFORE (render.rs:115):
if is_sandbox && node.line_count.is_none() {

// AFTER:
if node.line_count.is_none() {

// BEFORE (render.rs:132):
if is_sandbox && let Some(count) = node.line_count && count >= 500 {

// AFTER:
if let Some(count) = node.line_count && count >= 500 {
```

### Build bar: Remove label and hover
```rust
// DELETE lines 14-16 from build_bar.rs:
// let mode_label = "Terminal".to_string();
// ui.label(egui::RichText::new(mode_label))
//     .on_hover_text(i18n.get("hover-build-sandbox"));
```

### Gitignore filter: Remove sandbox condition
```rust
// BEFORE (init.rs:218):
if line.contains(".polycredo") || line.contains("sandbox") {

// AFTER:
if line.contains(".polycredo") {
```

### Modal dialogs cleanup
```rust
// DELETE from modal_dialogs.rs:
// // TODO: Phase 10 — remove modal_dialogs/sandbox.rs entirely
// #[cfg(never)]
// mod sandbox;

// Also remove comment on line 85:
// // 6a. Sandbox sync dialog removed (Phase 9)
```

## Scope Boundary -- What NOT to Touch

These files contain sandbox references but belong to later phases:

| File | Sandbox References | Phase |
|------|-------------------|-------|
| `editor/files.rs` | `.polycredo/sandbox` path checks (lines 62-66, 96-99, 170) | Phase 11 |
| `editor/ui.rs` | `sandbox_mode_enabled` parameter (line 15, 88-90) | Phase 11 |
| `terminal/mod.rs` | `terminal_mode_label`, `sandbox_root` functions (lines 30-78) | Phase 11 |
| `ai_panel.rs` | Comment only (line 161) | Cosmetic, low priority |
| `locales/*.ftl` | ~40+ sandbox i18n keys | Phase 12 |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + cargo test |
| Config file | Cargo.toml |
| Quick run command | `cargo check` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UI-01 | Settings sandbox block removed | compile | `cargo check` | N/A (deletion) |
| UI-02 | sandbox.rs modal deleted | compile | `cargo check` | N/A (deletion) |
| UI-03 | Sandbox OFF dialog removed | compile | `cargo check` | N/A (part of UI-01) |
| UI-04 | File tree is_sandbox param removed | compile | `cargo check` | N/A (deletion) |
| UI-05 | Build bar label removed | compile | `cargo check` | N/A (deletion) |
| UI-06 | Toast akce removed | manual-only | N/A | Already done Phase 9 |
| STATE-01-04 | State fields removed | manual-only | N/A | Already done Phase 9 |

### Sampling Rate
- **Per task commit:** `cargo check`
- **Per wave merge:** `cargo test`
- **Phase gate:** `cargo test` green

### Wave 0 Gaps
None -- this is a deletion phase. Successful `cargo check` after all removals is the primary validation. Existing `cargo test` suite (including `all_lang_keys_match_english`) will catch regressions.

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection of all referenced files
- Phase 9 commit history confirming STATE-01 through STATE-04 completion

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - pure Rust/egui deletion, no new libraries
- Architecture: HIGH - all files inspected, call chains verified
- Pitfalls: HIGH - scope boundaries clearly defined by CONTEXT.md

**Research date:** 2026-03-05
**Valid until:** 2026-04-05 (stable -- no external dependencies)
