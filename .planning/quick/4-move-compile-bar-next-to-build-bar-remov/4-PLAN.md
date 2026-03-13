---
phase: quick
plan: 4
type: execute
wave: 1
depends_on: []
files_modified:
  - src/app/ui/terminal/bottom/build_bar.rs
  - src/app/ui/terminal/bottom/mod.rs
  - src/app/ui/terminal/bottom/compile_bar.rs
autonomous: true
requirements: [QUICK-4]
must_haves:
  truths:
    - "Compile buttons (Create .deb on Linux, WIP labels on Win/Mac) appear in the build bar row after profile dropdown"
    - "compile_bar.rs no longer exists"
    - "No compile_bar references remain in mod.rs"
    - "Project compiles without errors"
  artifacts:
    - path: "src/app/ui/terminal/bottom/build_bar.rs"
      provides: "Combined build + compile controls in single horizontal bar"
      contains: "btn-create-deb"
    - path: "src/app/ui/terminal/bottom/mod.rs"
      provides: "Bottom panel without compile_bar module"
  key_links:
    - from: "src/app/ui/terminal/bottom/build_bar.rs"
      to: "WorkspaceState"
      via: "compile button spawns terminal"
      pattern: "build-deb"
---

<objective>
Move compile_bar buttons into build_bar (same horizontal row), remove "Compile" title, and delete compile_bar.rs.

Purpose: Simplify the bottom panel layout by consolidating compile actions into the build bar instead of a separate row.
Output: Single build bar with both run profiles and compile buttons; compile_bar.rs removed.
</objective>

<execution_context>
@/home/stkremen/.claude/get-shit-done/workflows/execute-plan.md
@/home/stkremen/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/app/ui/terminal/bottom/build_bar.rs
@src/app/ui/terminal/bottom/compile_bar.rs
@src/app/ui/terminal/bottom/mod.rs

<interfaces>
From src/app/ui/terminal/bottom/build_bar.rs:
```rust
pub fn render_build_bar(ui: &mut egui::Ui, ws: &mut WorkspaceState, i18n: &crate::i18n::I18n)
```

From src/app/ui/terminal/bottom/compile_bar.rs:
```rust
pub fn render_compile_bar(ui: &mut egui::Ui, ws: &mut WorkspaceState, _i18n: &crate::i18n::I18n)
```

Key: build_bar.rs already imports `FocusedPanel` unconditionally (line 1), so no new imports needed for the linux compile button logic.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Move compile buttons into build_bar and remove compile_bar</name>
  <files>src/app/ui/terminal/bottom/build_bar.rs, src/app/ui/terminal/bottom/mod.rs, src/app/ui/terminal/bottom/compile_bar.rs</files>
  <action>
1. In `build_bar.rs` — insert compile button logic AFTER the profile ComboBox block (after line 57, the closing `});` of `combo.show_ui`) and BEFORE `ui.add_space(28.0)` (line 60). Add a `ui.separator();` then the platform-conditional block:
   - `#[cfg(target_os = "linux")]` block: "Create .deb" button using `_i18n.get("btn-create-deb")` with hover text `_i18n.get("hover-create-deb")`, on click spawns terminal with `"export DEB_BUILD_TYPE=deb-dev && ./packaging/deb/build-deb.sh"`, sets `ws.build_terminal`, `ws.show_build_terminal = true`, `ws.focused_panel = FocusedPanel::Build`. Copy the exact logic from compile_bar.rs lines 14-30.
   - `#[cfg(target_os = "windows")]` block: `ui.weak("MSI Installer (WIP)");`
   - `#[cfg(target_os = "macos")]` block: `ui.weak("DMG Bundle (WIP)");`
   - Rename `i18n` parameter to `i18n` (keep as is — compile_bar used `_i18n` but build_bar already uses `i18n`, so use `i18n` for the new code).

2. In `mod.rs`:
   - Remove `pub mod compile_bar;` (line 2)
   - Remove both calls to `compile_bar::render_compile_bar(ui, ws_arg, i18n);` (line 43 in `render_bottom_panel`) and `compile_bar::render_compile_bar(ui, ws, i18n);` (line 127 in `render_bottom_content`)

3. Delete `compile_bar.rs` file entirely.
  </action>
  <verify>
    <automated>cd /home/stkremen/MyProject/Rust/polycredo_editor && cargo check 2>&1 | tail -5</automated>
  </verify>
  <done>Compile buttons appear in build_bar row, compile_bar.rs deleted, no compile_bar references in mod.rs, cargo check passes.</done>
</task>

</tasks>

<verification>
- `cargo check` passes with no errors
- `grep -r "compile_bar" src/` returns no results
- `ls src/app/ui/terminal/bottom/compile_bar.rs` fails (file deleted)
- `grep "btn-create-deb" src/app/ui/terminal/bottom/build_bar.rs` finds the button
</verification>

<success_criteria>
- Build bar contains both run profile dropdown and compile buttons in single horizontal row
- compile_bar.rs is deleted
- No references to compile_bar module remain
- Project compiles cleanly
</success_criteria>

<output>
After completion, create `.planning/quick/4-move-compile-bar-next-to-build-bar-remov/4-SUMMARY.md`
</output>
