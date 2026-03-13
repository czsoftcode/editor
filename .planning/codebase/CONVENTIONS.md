# Coding Conventions

**Analysis Date:** 2026-03-04

## Naming Patterns

**Files:**
- Module files use snake_case: `build_runner.rs`, `ai_panel.rs`, `file_tree.rs`
- Main module file is `mod.rs` in directory
- Example structure: `src/app/ui/editor/mod.rs` exports submodules like `files.rs`, `render/`, `render_lsp.rs`

**Functions:**
- Public functions use snake_case: `pub fn run_profile()`, `pub fn is_path_modified()`, `pub fn render_diff_modal()`
- Private functions use snake_case: `fn parse_build_messages_json()`, `fn default_tool_type()`
- Initialization functions often named `new()`: `impl Editor { pub fn new() -> Self { ... } }`
- Rendering functions prefixed with `render_`: `render_workspace()`, `render_diff_modal()`

**Types:**
- Structs use PascalCase: `struct BuildError`, `struct Tab`, `struct EditorApp`, `struct PluginManager`
- Enums use PascalCase: `enum ProjectType`, `enum ErrorParserType`, `enum FocusedPanel`, `enum AppAction`
- Enum variants use PascalCase: `ProjectType::Rust`, `ProjectType::Symfony74`, `AppAction::OpenInNewWindow`

**Variables:**
- Local variables use snake_case: `let mut full_command = profile.command.clone()`, `let working_dir = ...`
- Constants in config.rs use SCREAMING_SNAKE_CASE: `const MAX_RECENT_PROJECTS: usize = 10`, `const EDITOR_FONT_SIZE: f32 = 14.0`
- Boolean field flags follow convention: `pub show_left_panel: bool`, `pub modified: bool`, `pub deleted: bool`

## Code Style

**Formatting:**
- No explicit formatter configured in codebase (no .rustfmt.toml, no Prettier)
- Code follows standard Rust formatting conventions
- Lines use 4-space indentation
- Struct initialization spreads across multiple lines for readability:
  ```rust
  Self {
      plugins: Arc::new(Mutex::new(Vec::new())),
      sandbox_root,
      blacklist: Arc::new(Mutex::new(Blacklist::default())),
      current_context: Arc::new(Mutex::new(initial_context)),
      action_sender: Arc::new(Mutex::new(None)),
      egui_ctx: Arc::new(Mutex::new(None)),
  }
  ```

**Linting:**
- No explicit linter configuration (.eslintrc, clippy.toml, etc.)
- Some clippy directives in place: `#[allow(clippy::too_many_arguments)]` observed in `src/app/ui/editor/render_lsp.rs:11`
- Code is checked via standard `cargo build` and `cargo check`

## Import Organization

**Order:**
1. Standard library imports (`use std::...`)
2. External crate imports (`use serde_json::...`, `use eframe::egui...`)
3. Local module imports (`use super::...`, `use crate::...`)

**Example from `src/app/ui/editor/render_lsp.rs`:**
```rust
use super::*;
use eframe::egui;
```

**Example from `src/app/build_runner.rs`:**
```rust
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use serde_json::Value;

use super::types::{BuildProfile, path_env};
use crate::app::ui::terminal::Terminal;
```

**Path Aliases:**
- No path aliases configured (no `paths = {...}` in Cargo.toml)
- Explicit module paths used: `use crate::app::...`, `use super::...`
- Wildcard imports common in render contexts: `use super::*;` in UI rendering modules

## Error Handling

**Patterns:**
- Result types returned from fallible operations: `pub fn run_build_check(root_path: PathBuf) -> mpsc::Receiver<Vec<BuildError>>`
- `anyhow::Result` used for library-level errors: `pub fn load_from_dir<P: AsRef<Path>>(&self, dir_path: P) -> anyhow::Result<()>`
- `expect()` and `unwrap()` used in initialization contexts where failure is fatal (445 instances detected in codebase):
  - `let langid: LanguageIdentifier = lang.parse().expect("i18n: invalid language code (must be BCP 47)")`
  - `let mut ctx = self.current_context.lock().expect("lock")`
- `if let` patterns for fallible operations:
  ```rust
  if let Ok(output) = output {
      let stdout = String::from_utf8_lossy(&output.stdout);
  }
  ```
- JSON parsing with try_recv pattern:
  ```rust
  if let Some(rx) = &self.lsp_hover_rx
      && let Ok(result) = rx.try_recv()
  {
      self.lsp_hover_rx = None;
      if let Some(hover) = result { ... }
  }
  ```

## Logging

**Framework:** println!, eprintln!, no structured logging framework detected

**Patterns:**
- Error output via `eprintln!()` for plugin loading errors: `eprintln!("Error loading plugin: {}", e);`
- No explicit log levels (debug, info, warn, error) implemented
- Plugin trace data saved to file when needed: `fn save_trace(trace: &str)`

## Comments

**When to Comment:**
- Module-level documentation comments explain purpose: `//! Internationalization using [Project Fluent](https://projectfluent.org/).`
- Complex logic receives line comments explaining intent
- Sections marked with separators: `// -----------------------------------------------------------------------`
- Example from `src/app/ui/editor/render_lsp.rs:5-7`:
  ```
  // -----------------------------------------------------------------------
  // LSP interaction logic
  // -----------------------------------------------------------------------
  ```

**Documentation:**
- Function-level doc comments explain parameters and behavior:
  ```rust
  /// Handles all LSP interactions: hover, go-to-definition (F12), completion (Ctrl+Space).
  /// Called after the TextEdit frame is rendered so self can be freely borrowed.
  pub(super) fn process_lsp_interactions(...)
  ```
- Type documentation in `src/app/types.rs`: `/// Display name of the profile (e.g., "Run Server", "Cargo Test")`
- No JSDoc/TSDoc style (this is Rust, not TypeScript)

## Function Design

**Size:**
- Functions range from 20 to 400+ lines depending on context
- Largest files: `src/app/mod.rs` (872 lines), `src/app/ui/editor/render_lsp.rs` (863 lines)
- Rendering functions tend to be longer due to egui UI structure requirements
- Parsing functions factored into separate functions: `parse_build_messages_json()`, `parse_build_errors_legacy()`

**Parameters:**
- Functions accept owned `PathBuf` when ownership needed
- Borrow `&Path` for read-only file paths
- UI functions receive `&egui::Ui` context parameter
- Multiple arguments handled with `#[allow(clippy::too_many_arguments)]` annotation when necessary:
  ```rust
  pub(super) fn process_lsp_interactions(
      &mut self,
      ui: &egui::Ui,
      _idx: usize,
      tab_path: &std::path::Path,
      lsp_client: Option<&crate::app::lsp::LspClient>,
      saved_response: &Option<egui::text_edit::TextEditOutput>,
      i18n: &crate::i18n::I18n,
  )
  ```

**Return Values:**
- Simple operations return `bool` or owned values
- Complex operations return `Result<T, E>` or `anyhow::Result<T>`
- UI functions often return custom result structs:
  ```rust
  pub struct EditorUiResult {
      pub clicked: bool,
      pub diff_action: Option<(String, DiffAction, String)>,
  }
  ```
- Background operations return `mpsc::Receiver<T>` for async results:
  ```rust
  pub(crate) fn run_build_check(root_path: PathBuf) -> mpsc::Receiver<Vec<BuildError>>
  ```

## Module Design

**Exports:**
- Public module items re-exported at parent level using `pub use`:
  - `pub use self::types::{HostContext, LoadedPlugin, PluginMetadata, PluginStatus};`
  - `pub use self::security::Blacklist;`
- Submodules declared with `pub mod` for public modules, `mod` for private
- Example structure in `src/app/registry/plugins/mod.rs`:
  ```rust
  pub mod host;
  pub mod security;
  pub mod types;
  ```

**Barrel Files:**
- `src/app/ui/mod.rs` contains just: `pub mod panels;` etc. (229 bytes)
- Rendering submodule exports in `src/app/ui/editor/render/mod.rs`:
  ```rust
  pub use helpers::{editor_line_count, goto_centered_scroll_offset, restore_saved_cursor};
  ```
- Test modules declared with `#[cfg(test)]` and `mod tests { ... }`

## Visibility

**Visibility Modifiers:**
- `pub` for public API
- `pub(crate)` for crate-level internal API: `pub(crate) fn run_build_check(...)`
- `pub(super)` for module-level internal API: `pub(super) fn process_lsp_interactions(...)`
- No explicit `pub(self)` (equiv to private)
- Private by default unless marked pub

## Struct Design

**Derive Macros:**
- Common derives: `#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]`
- Example from `src/app/types.rs`:
  ```rust
  #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq)]
  pub(crate) enum ErrorParserType {
      #[default]
      None,
      Rust,
  }
  ```
- GUI-related structs omit serialization, keep Clone: `#[derive(Clone)]` for UI state

**Field Documentation:**
- Important fields documented with `///` doc comments:
  ```rust
  pub struct BuildProfile {
      /// Display name of the profile (e.g., "Run Server", "Cargo Test")
      pub name: String,
      /// Main command to execute
      pub command: String,
      /// Working directory (relative to project root)
      #[serde(default)]
      pub working_dir: Option<String>,
  }
  ```

---

*Convention analysis: 2026-03-04*
