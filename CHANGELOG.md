# Changelog

All notable changes to the PolyCredo Editor project will be documented in this file.

## [0.5.6] - 2026-02-21

### Added
- **AI Diff Settings**: Added a setting to toggle the AI Diff Preview layout between "Inline" (default) and "Side-by-side" rendering modes.
- **AI Diff Preview (Initial)**: Introduced a foundational visual diff viewer for AI-proposed code changes. It uses the `similar` crate to calculate text differences and displays them in a modal dialog with inline color-coded `-` / `+` lines, allowing the user to accept or reject AI edits safely before they overwrite local editor tabs.
- **AI Context Awareness**: The AI assistant in the terminal now automatically receives context about the current project state.
  - **Open Files**: Automatically lists currently open files in the editor when starting an AI tool.
  - **Build Errors**: Provides details about the latest build errors and warnings to the AI agent.
  - **Manual Sync**: Added a "Sync" button in the AI panel to refresh the context (e.g., after opening more files or fixing errors) without restarting the tool.

### Changed
- **LSP Reference Picker**: Enhanced the 'Find References' modal to extract and display actual source code line previews, and automatically requests keyboard focus when opened.
- **Editor Refactoring**: Split the monolithic `src/app/ui/editor/mod.rs` and `render.rs` files (>2700 lines combined) into smaller, purpose-driven modules (e.g., `tabs.rs`, `files.rs`, `render_lsp.rs`). This significantly improves maintainability and aligns with the project's architecture guidelines.

### Fixed
- **Terminal Text Selection**: Improved copying of wrapped text in the terminal. The selection now correctly detects terminal wrap-lines, avoiding the insertion of artificial newlines and collapsing trailing spaces to maintain proper word separation.

## [0.5.5] - 2026-02-21

### Added
- **Smart Typing & Auto-indent**: Implemented intelligent text handling for a smoother coding experience.
  - **Auto-indent**: Pressing Enter now automatically preserves the indentation level of the previous line.
  - **Smart Indent**: Pressing Enter after an opening brace `{` automatically adds an extra level of indentation (4 spaces).
  - **Smart Un-indent**: Typing a closing brace `}` at the start of an indented line automatically removes 4 spaces, aligning it with the corresponding opening block.

## [0.5.4] - 2026-02-21

### Added
- **Precise LSP Navigation**: Jumps (F12 and Shift+F12) now place the cursor at the exact character position (line and column) provided by the LSP server.
- **Search Feedback**: Added a visible modal with a spinner when searching for references, providing immediate feedback that the operation is in progress.
- **LSP Status Messages**: Integrated a temporary status indicator in the bottom bar for LSP operations (e.g., "Searching...", "No references found").

### Fixed
- **Stable Reference Picker**: Fixed an issue where the references list would jump to the top on every interaction. Scrolling is now stable and only follows the selection when using keyboard navigation.
- **Editor Focus**: The editor now automatically requests focus after any LSP jump, ensuring the cursor is visible and the user can immediately continue typing.
- **Improved Cursor Placement**: Corrected the character index calculation to handle different line ending scenarios and multi-byte characters more reliably.

## [0.5.3] - 2026-02-21

### Added
- **Find References (Shift+F12)**: Pressing Shift+F12 sends a `textDocument/references` request. If multiple references are found, a modal picker is displayed showing file name, line, and character. Selecting an item jumps to that location. If only one reference is found, the editor jumps directly.

### Fixed
- **LSP Scoping**: Restricted LSP notifications (`didOpen`) and diagnostics display to `.rs` files. This prevents `rust-analyzer` from incorrectly reporting syntax errors in non-Rust files such as Markdown (`.md`), SVG (`.svg`), or plain text.
- **Diagnostics Isolation**: Even if the LSP server sends background diagnostics for ignored files, they are no longer displayed in the editor or status bar if the file's `lsp_version` is 0 (non-Rust).

## [0.5.2] - 2026-02-21

### Fixed
- **Terminal CPR (Cursor Position Reporting)**: `PtyEvent::PtyWrite` events (responses to `ESC[6n` DSR queries) were being silently discarded. They are now written back to the PTY, allowing programs like vim/nvim and bash prompts to correctly detect cursor position.
- **Terminal Confirmation**: Added a confirmation dialog before closing a running terminal in the AI panel to prevent accidental process termination.
- **AI Viewport**: Option to undock the AI terminal into a separate system window (viewport) that can be moved to another monitor.
- **Markdown Synchronized Scrolling**: Implemented proportional synchronized scrolling between the editor and the preview. Both panes can also be scrolled independently, and the preview now correctly reaches the end of the document regardless of content length.
- **Terminal Ctrl+X**: On Linux, egui converts Ctrl+X to `Event::Cut` instead of `Event::Key`, which `TerminalView` was not processing. Added an explicit handler that maps `Event::Cut` to control character `0x18` — nano and other TUI applications now correctly receive Ctrl+X.
- **Terminal Keyboard Input Out of Bounds**: `TerminalView` ignored keyboard input if the mouse was not hovering over the terminal. Added a fallback handler in `terminal.rs` that processes keys (text, Ctrl+letter, special keys) even when the terminal is focused but the mouse is elsewhere.

## [0.5.1] - 2026-02-21

### Added
- **LSP Hover Documentation**: Hovering the mouse over code triggers a `textDocument/hover` request (600 ms debounce). Result is displayed in a floating popup that correctly renders markdown — code fences are shown monospaced in blue, prose text in gray. Popup dismisses on mouse movement.
- **Go-to-Definition (F12)**: Pressing F12 at the cursor position sends a `textDocument/definition` request. The editor opens the target file and jumps to the correct line. Handles Scalar, Array and LocationLink response formats.
- **Autocomplete (Ctrl+Space)**: Pressing Ctrl+Space sends a `textDocument/completion` request. Results are shown in a floating dropdown (max 25 items) with kind labels (`fn`, `st`, `kw`, …). Navigate with ↑↓ arrows, accept with Enter or Tab, dismiss with Escape, or click an item.
- **LSP Capabilities**: Added `hover`, `definition` and `completion` capabilities to `InitializeParams` so rust-analyzer knows the editor supports these features.

### Fixed
- **Diagnostic Gutter Dots**: Colored indicator dots were overlapping line numbers. Dots are now placed on the left side of the gutter (left + 6 px), numbers remain right-aligned and unobstructed.
- **Diagnostic Underlines (Squiggles)**: Added 2 px colored underlines beneath lines with LSP diagnostics — red for errors, orange for warnings, blue for information, green for hints.
- **Diagnostic Count in Status Bar**: The status bar now shows `✕ N` (red) and `⚠ N` (orange) counts for errors and warnings in the active file. Counts are only shown when greater than zero.
- **Hover Popup Markdown Rendering**: The hover popup now correctly parses fenced code blocks instead of displaying raw ` ```rust ``` ` markers.

## [0.5.0] - 2026-02-21

### Added
- **LSP Client MVP**: Integrated Language Server Protocol (LSP) support via `async-lsp`.
- **Rust Integration**: Automatic detection and startup of `rust-analyzer` for Rust projects.
- **Inline Diagnostics**: Real-time visualization of compilation errors and warnings directly in the editor gutter.
- **Diagnostic Tooltips**: Detailed error messages displayed on hover over the line numbers.
- **Asynchronous Architecture**: Implemented a robust, non-blocking LSP communication layer using Tokio.

### Fixed
-   **LSP Client Stability**: Corrected a critical panic on startup by properly entering the Tokio runtime context before spawning the language server process.
-   **LSP Initialization Loop**: Resolved an infinite retry loop in `render_workspace` that occurred when LSP client initialization failed, preventing system resource exhaustion.
-   **LSP Failure Handling**: Improved `init_workspace` to correctly mark `lsp_binary_missing` when LSP client initialization fails, ensuring the "Install" prompt is shown and retries are prevented.

## [0.4.2] - 2026-02-20

### Added
- **Build Runner Profiles**: Introduced project-specific build/run configurations via `.polycredo/profiles.toml`. Supports environment variables, custom working directories, and automated Rust error parsing.
- **Collapsible UI**: Integrated runners into the build panel as a space-saving collapsible menu.

### Fixed
- **Terminal Text Selection**: Improved text selection and copying. Newlines are now preserved, trailing spaces are trimmed, and the implementation is now part of the editor to avoid modifying vendored code.
- **Debian Version Format**: Corrected `.deb` package versioning to `MAJOR.MINOR.PATCH-BUILD_NUMBER` (e.g., 0.4.2-48).

## [0.4.0] - 2026-02-20

### Added
- **TOML Configuration**: Switched from JSON to TOML for application settings (`settings.toml`) with automatic migration of existing configuration.
- **CI/CD Quality Gate**: Introduced automated code quality checks (formatting, clippy, tests) using GitHub Actions and a local `check.sh` script.
- **Shared File Index**: Implemented `ProjectIndex` for asynchronous and incremental project file indexing. Unifies data for Ctrl+P, global search, and file tree.
- **Command Palette (Ctrl+Shift+P)**: Added a central command menu with i18n support for quick keyboard-driven editor control.
- **Quick File Open (Ctrl+P)**: Implemented fuzzy file search with automatic scrolling to the selected item.

### Fixed
- **Scrolling in Ctrl+P**: Fixed an issue where the selected item disappeared outside the visible list area during arrow navigation.
- **Search Performance**: Global search (`Ctrl+Shift+F`) now utilizes the shared index instead of repeated disk scanning.
