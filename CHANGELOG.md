## [0.7.7] - 2026-02-24

### Added
- **AI Inspector**:
    - Introduced a dedicated **AI Inspector** panel within the Gemini modal (accessible via 🔍 icon).
    - Real-time visualization of the **full JSON payload** sent to the AI, including system prompts and raw conversation history.
    - Detailed **Context Trace**: Displays exactly what the AI sees, including lists of open files (with active tab indicator) and current build errors.
    - Functional utilities: One-click "Copy" to clipboard and "Clear" trace log.
- **Gemini CLI Branding & UX**:
    - **Polished ASCII Logo**: Implemented a dynamic, per-line color gradient for the PolyCredo logo (Sky Blue for "Poly", Mint Green for "Credo", and Gold for "CLI").
    - **Slant-aware Coloring**: Logic precisely follows the diagonal gap between letters for a professional, high-fidelity look.
    - **Refined Metadata**: Version, Model, and Plan info now use soft silver/gray tones to remain readable without distracting from the main logo.
- **Persistent AI Configuration**:
    - Added a **✔ Save** button to Gemini settings to persist custom system prompts and language preferences globally.
    - Settings are now stored in `~/.config/polycredo-editor/settings.toml` and survive application restarts.
    - New **Factory Reset** logic to quickly revert to default agent instructions.

### Changed
- **Conversational Continuity (Thread Support)**:
    - Updated the Gemini plugin to support **full conversation history**. The AI now "remembers" previous messages within a thread.
    - Refactored host-guest communication to pass history as a structured JSON payload instead of a single string.
- **UI Refinements**:
    - **"New Thread" (Nové vlákno)**: Rebranded "New Query" across all localizations (CS, SK, EN, DE, RU) to better reflect modern AI interaction patterns.
    - **Thread Reset**: Starting a new thread now clears both the UI and the underlying AI memory while restoring the welcoming ASCII logo.
    - **Enhanced Typography**: Increased Markdown font size to **120%** and expanded block spacing (12.0px) for significantly better readability of AI responses.
    - **Syntax Highlighting**: Enabled `better_syntax_highlighting` for the Markdown viewer, providing colored code blocks in AI answers.
- **Ergonomic Layout**:
    - Redesigned the Gemini modal footer to align agent controls to the left and the "Close" button to the far right.
    - Optimized the modal using `SidePanel` / `CentralPanel` internal layouts to ensure elements fill the entire window height and don't clump at the top.

### Fixed
- **Missing Context**: Resolved an issue where AI agents would lose track of the conversation after the first message.
- **UI Stability**: Fixed "stuck-at-top" layout bugs in the Gemini dialog through proper panel-inside-modal orchestration.

## [0.7.6] - 2026-02-23

### Added
- **AI Agent Autonomy (Gemini)**:
    - **Sandbox Execution**: Agents can now execute shell commands directly within the project sandbox via the new `exec_in_sandbox` host function.
    - **Real-time Monologue**: Introduced a "thinking aloud" feature where agents log their internal actions (e.g., reading files, running tests) to the UI in real-time.
    - **Advanced Tool Use**: Updated the Gemini plugin to support function calling for both file reading and command execution.
- **Unified AI CLI (StandardAI)**:
    - **Interactive Terminal UI**: Replaced standard Markdown rendering with a dedicated terminal-style output (monospace, dark theme, auto-scroll).
    - **Conversation History**: Full support for persistent chat history (Question/Answer pairs) within the current session.
    - **Precise Token Tracking**:
        - Introduced `log_usage` to report exact `totalTokenCount` from Gemini API.
        - Implemented cumulative **Session tokens** counter in the footer.
        - Detailed token breakdown (Input/Output) displayed in the real-time monologue.
    - **CLI-native Keyboard Handling**:
        - `Enter` to send queries (auto-clears prompt).
        - `Shift+Enter` / `Ctrl+J` for new lines.
        - `Arrow Up/Down` to cycle through command history.
- **Enhanced AI Security & Governance**:
    - **Hardcoded Security Mandates**: Implemented immutable safety rules in both Rust (Host) and WASM (Guest) to strictly prevent agents from accessing paths outside the sandbox.
    - **Path Traversal Protection**: Added technical validation to block `..` and absolute paths in agent-executed commands.
    - **Contextual Authorization**: Moved plugin permission requests (e.g., internet access) directly into the agent's dialog for a less intrusive workflow.
- **Agent Personalization & Polish**:
    - **Dynamic Role Configuration**: Added "Agent Settings" to the Gemini modal, allowing users to customize the system prompt and communication language.
    - **Visual Feedback**: Added an animated, color-changing activity spinner in the footer.
    - **Localized AI Instructions**: Created `ai.ftl` files for all supported languages (CS, EN, SK, DE, RU) with factory-default agent instructions.
    - **Persistence**: Agent settings (role, language), token counts, and window position are now saved per-project in `PersistentState`.
    - **UI Refinements**: Shortened sandbox root path in the footer with '~' and enlarged directory label for better readability.

### Fixed
- **UI Focus Stability**: Resolved an issue where the main AI prompt would steal focus from settings input fields.
- **Window Positioning**: Implemented robust window centering and project-unique IDs to ensure AI dialogs remember their last position correctly.

## [0.7.5] - 2026-02-23

### Added
- **Draggable Modals**: All modal windows are now freely draggable across the editor workspace. Multiple modals can be opened simultaneously without losing state.
- **Dynamic Plugins Submenu**: The File > Plugins menu is now hierarchical and dynamically generated. It mirrors the Plugin Manager's categories (AI Agents, General) and automatically updates when new plugins are loaded.
- **Menu-based Plugin Execution**: Added the ability to trigger WASM plugin functions directly from the menu, with results displayed as interactive toasts.

### Changed
- **Unified Modal Framework**: Refactored all dialogs (About, Privacy, Startup, Conflict, Git, Search, LSP, SVG, Settings, Plugins) to use the `StandardModal` framework. This ensures a consistent UI with standardized headers, fixed button footers, and stable dimensions.
- **Settings Refactoring**: Converted the Settings modal to use the `StandardModal` framework with a split-view layout (General, Editor).
- **Menu Reorganization**: Moved "Settings" from the Help menu to the File menu for better discoverability.
- **Shortcuts**: Added default keyboard shortcuts: `Ctrl+,` for Settings and `Ctrl+Shift+L` for Plugins.
- **Linux Compatibility**: Changed Plugin shortcut from `Ctrl+Shift+X` to `Ctrl+Shift+L` to avoid conflicts with system "Cut" commands.

### Fixed
- **Missing Editor Tabs**: Resolved a regression in version 0.7.4 b74 where file tabs were not being rendered in the editor UI. Re-integrated the `tab_bar` and `goto_line_bar` components into the main editor render loop.
- **Command Palette Metadata**: Added missing shortcut hints to the Command Palette (`Ctrl+Shift+P`) for Settings and Plugins.
- **Borrow Conflicts**: Resolved complex mutable borrow conflicts in `egui` by decoupling the rendering of the modal body and footer.

## [0.7.4] - 2026-02-23

### Added
- **Standardized Modal Framework**: Introduced the `StandardModal` component to ensure a consistent look and feel across all major dialogs (Settings, Plugins, Gemini). Features built-in panel-based layout, fixed footers, and dimension stability.
- **Enhanced Plugin Manager**:
    - **Hierarchical Navigation**: Replaced the flat list with a category-based tree view (Left Panel).
    - **Unified AI Hub**: Moved all AI-related configuration (Diff modes, AI fonts, security blacklists) into the Plugin Manager under the "AI Agents" category.
    - **Overview Page**: Added an educational landing page explaining PolyCredo's WASM-based plugin security and sandboxing.
    - **Independent State Management**: Settings and Plugin modals now use separate draft buffers, allowing both to be open simultaneously without state interference.
- **Modernized Gemini UI**:
    - Refactored the Gemini modal using the new standard framework.
    - **Improved Chat Layout**: Repositioned the prompt input to the bottom of the window, providing a more intuitive "chat-like" experience.
    - **Dynamic Expansion**: The response area now automatically fills the available space above the input field.
    - **Direct Modal Control**: Replaced command-level hacks with an explicit `show_gemini` flag in the workspace state.
- **Full Multi-Language Support**: Completed missing translations for all new Plugin Manager components and menu items in CS, EN, SK, DE, and RU.

### Fixed
- **UI Stability**: Resolved issues where modal windows would "jump" or resize during mouse movement by enforcing strict panel-based sizing.
- **Fixed Egui Panic**: Corrected a layout assertion failure caused by negative height calculations in high-density UI scenarios.

## [0.7.3] - 2026-02-23

### Fixed
- **Performance Optimizations (Audit S-1, S-4, S-5)**:
  - **Per-Tab Markdown Cache**: Moved the markdown preview cache into individual tabs. This prevents full reparsing of markdown documents when switching between tabs and ensures each document maintains its own scroll and render state.
  - **Versioned Settings Application**: Settings (theme, fonts) are now only applied to the UI context when they actually change. This eliminates redundant style calculations in every frame across all viewports.
  - **Pre-Calculated Canonical Paths**: Added pre-calculated canonical paths to editor tabs. File system watcher events now resolve to tabs instantly without triggering expensive `canonicalize()` calls in the UI loop.

## [0.7.2] - 2026-02-23

### Fixed
- **UI Freeze Resolution (Deadlock Prevention)**: Refactored `PluginManager` to use a granular locking strategy. The main plugin list is no longer held locked during WASM execution or network calls, ensuring the UI remains responsive even during long AI operations.
- **Optimized Blacklist Performance**: Introduced pre-compiled regex caching for the file blacklist. Security checks are now significantly faster, especially in projects with thousands of files.
- **Efficient File Scanning**: Optimized `host_list_files` by implementing directory pruning (filter_entry). The plugin host now skips blacklisted directories (like `target` or `.git`) entirely, drastically reducing disk I/O.
- **Gemini API Compatibility**: Resolved "thought_signature" errors in the Gemini plugin by ensuring all API-provided metadata is preserved in chat history.

### Added
- **AI Function Calling**: The internal Gemini plugin can now dynamically request to read files using the `read_project_file` tool, allowing it to analyze code outside the active editor tab.
- **Robust API Error Handling**: Enhanced plugin error reporting to display full API responses in case of failures (e.g., safety blocks or invalid arguments).

## [0.7.1] - 2026-02-23

### Added
- **Context-Aware AI Assistant**: The internal Gemini plugin now has "eyes" into the project.
  - **Host Functions**: Implemented `read_project_file`, `list_project_files`, `get_active_file_path`, and `get_active_file_content`.
  - **Automatic Context**: Content of the active editor tab is automatically included in AI queries.
  - **Project Awareness**: AI can now see the project structure and answer questions about other files.
- **Capability-Based Security Model**:
  - **Plugin Blacklist**: Global setting to block plugins from accessing sensitive files (e.g., `.env`, `*.key`).
  - **Git-Native Blocking**: Automatically inherits ignore patterns from `.gitignore`.
  - **Dynamic Enforcement**: Security checks are performed within the Host Functions before data is passed to WASM.
- **Enhanced UI & UX**:
  - **Flexible Modals**: Gemini dialog is now resizable and persists across user interactions.
  - **Font Synchronization**: AI modal font size now matches the editor's global font settings.
  - **Improved Focus Logic**: Prevented the terminal from stealing keyboard input when the AI modal is active.
- **Multi-language Localization**: Full support for CS, EN, SK, DE, RU in Plugin Manager and AI components.

### Fixed
- **Extism Runtime Stability**: Resolved low-level memory management issues in Host Functions for Extism 1.13.
- **Async UI Pacing**: Ensure non-blocking plugin calls to keep the editor responsive during network/WASM execution.

## [0.7.0] - 2026-02-22

### Added
- **WASM Plugin Foundation**: Implemented a robust, sandboxed plugin system based on WebAssembly using the Extism runtime.
  - **Plugin Manager**: Automated loading of `.wasm` modules from the `~/.config/polycredo-editor/plugins` directory.
  - **Host-Guest Communication**: Support for calling WASM functions with string-based input/output.
  - **Plugin Commands**: Extended `CommandAction` to allow plugins to register and handle editor commands.
  - **SDK Sample**: Added a "Hello World" plugin sample in `docs/samples/hello-plugin` to demonstrate WASM plugin development using Rust and `extism-pdk`.
- **UI Toasts for Plugins**: Responses from WASM plugins are now displayed as informative toasts in the editor UI.

### Fixed
- **Command Palette Input Leakage**: The Command Palette now correctly blocks keyboard input (like arrow keys) from leaking into background terminals.
- **Improved Palette Scrolling**: Fixed an issue where the Command Palette would reset its scroll position to the top when using the mouse wheel.
- **Command Palette Visibility**: Switched from `Modal` to `Window` with explicit anchoring and visual dimming for more reliable rendering across different window states.

### Changed
- **Command Palette Refactoring**: Updated the Command Palette to handle both internal and external (plugin) actions, with improved safety using `Clone` for command metadata.
- **Version Milestone**: Bumped version to 0.7.0 to mark the introduction of the external plugin system.

## [0.6.9] - 2026-02-22

### Added
- **Plugin Foundation (Internal)**: Introduced a centralized `Registry` system for managing commands and UI panels.
  - **Command Registry**: Commands are now registered with unique string IDs (e.g., `editor.open_file`), i18n keys, and shortcuts.
  - **Panel Registry**: Added infrastructure for managing dynamic panels in different UI areas (Left, Right, Bottom).
- **Decoupled Command Palette**: Refactored the Command Palette to pull its actions directly from the global command registry.

### Fixed
- **Terminal Busy-Loop Prevention**: Implemented a graceful shutdown for terminals in `Drop`. By sending `exit\n` to the shell before closing, we ensure the PTY event loop in the vendor code (egui_term) terminates correctly, preventing a "zombie" thread from consuming 100% CPU on a disconnected channel.

### Changed
- **Simplified Sandbox Sync**: Removed manual synchronization toggles from the File Tree and AI panel. Synchronization between the project and sandbox is now always automatic (both on file changes and tool startup), ensuring a seamless and reliable AI agent experience with less UI clutter.

## [0.6.8] - 2026-02-22

### Changed
- **AI Panel UI**: Removed the "Synchronizovat nyní" button to reduce clutter.
- **Enhanced AI Synchronization**: Added a synchronization toggle (🔄/📁) directly to the AI panel. When enabled, the editor automatically synchronizes the project state to the sandbox before starting any AI tool, ensuring the assistant always works with the latest code.

## [0.6.7] - 2026-02-22

### Fixed
- **Terminal CPU Usage**: Implemented a `dirty` flag for the terminal backend. This eliminates deep-cloning of the entire Alacritty grid (potentially several MBs) 60 times per second when the terminal is idle, dramatically reducing CPU load.
- **Editor Render Optimization**: Removed redundant cloning of search matches and tab paths in the main render loop for both normal and markdown editors.
- **Search Highlight Performance**: Optimized the search highlight overlay algorithm from $O(N \cdot M)$ to $O(N + M)$ by leveraging sorted sections and matches, ensuring smooth rendering even with thousands of search results in large files.

### Changed
- **High-Performance Sandbox Sync**: Optimized the sandbox-to-project file comparison to use file size metadata first, then `xxh64` hashing for identical sizes. This replaces the expensive "read-to-string" content comparison and eliminates significant disk I/O and memory allocations.
- **Sandbox Scan Debouncing**: Implemented a time-based debounce for sandbox staged files scanning (minimum 1s after the last change and 3s between scans) to prevent CPU and I/O spikes during rapid file activity by AI agents.

## [0.6.6] - 2026-02-22

### Added
- **Git Safety Gate**: Git operations that modify the repository (add, commit, push, pull, checkout, reset) are now automatically disabled in the left panel if there are pending sandbox changes. This prevents accidental commits of partial or unreviewed AI-generated work.
- **UI Tooltips**: Added informative tooltips explaining why Git buttons are disabled and how to re-enable them (by promoting or discarding sandbox changes).

### Changed
- **Localizations**: Added `hover-git-disabled-sandbox` string to English and Czech localization files.

## [0.6.5] - 2026-02-22

### Fixed
- **Infinite Sync Loop (Performance)**: Fixed a critical bug where changes in the `.polycredo` directory (including the sandbox) were being synced back to the sandbox in an infinite loop, causing ~20% CPU usage.
- **Optimized Project Watcher**: Refactored the file watcher path filter in `src/watcher.rs` to avoid redundant string allocations and expensive path component iterations for every file system event, resulting in a significantly lower CPU footprint (from ~20% to <3% in idle).

### Changed
- **Sync Logic Safeguard**: Added explicit checks to ensure that the project-to-sandbox synchronization ignores all paths within the `.polycredo` directory.

## [0.6.4] - 2026-02-22

### Added
- **Mandatory Deletion Modal**: Implemented a confirmation dialog for sandbox file deletions. Users must now explicitly choose between deleting the file in the project or restoring it from the project backup, ensuring 100% directory consistency and preventing accidental data loss.
- **Project-to-Sandbox Sync**: Added real-time synchronization from the project to the sandbox via FS events, ensuring AI agents always have up-to-date data.

### Fixed
- **LSP Busy-Loop Prevention**: Implemented a 30s debounce for automatic LSP restarts to eliminate high CPU usage (originally ~26%) during persistent initialization failures.
- **Sandbox Watcher Filter**: Corrected the file watcher filter that was erroneously ignoring the `.polycredo/sandbox` directory, enabling reliable event-driven UI updates for AI-generated changes.

### Changed
- **Synchronization Logic**: Removed automatic deletion logic that previously synced changes from sandbox to project without confirmation.
- **Localizations**: Added English and Czech translation strings for synchronization, conflict resolution, and deletion dialogs.

## [0.6.3] - 2026-02-22

### Changed
- **Highlighter Optimization (V-4, V-7)**: Migrated Highlighter cache to use Arc<egui::text::LayoutJob>. This eliminates expensive cloning of large layout structures during rendering, resulting in a ~1500x performance gain (from ~8ms to ~0.9ms) when scrolling and rendering unchanged files with 10k+ lines.
- **Improved UI Responsiveness**: Updated editor rendering logic in both normal and markdown modes to efficiently handle shared layout jobs, ensuring smooth interaction even in extremely large projects.

# Changelog

All notable changes to the PolyCredo Editor project will be documented in this file.

## [0.6.2] - 2026-02-22

### Fixed
- **Mutex Safety**: Replaced all dangerous `.lock().unwrap()` calls with `.expect("context")` throughout the codebase. This ensures that any potential mutex poisoning results in a descriptive error message instead of a silent or confusing crash.
- **Data Integrity (K-1)**: Implemented a `read_error` flag for editor tabs. The editor now correctly detects if a file failed to read and enters a safety read-only mode, preventing the accidental overwriting of original files with error messages.
- **Terminal Stability**: Fixed a potential integer underflow in terminal grid indexing that could lead to crashes when scrolling through large history buffers.
- **LSP Initialization Timeout**: Added a 10-second timeout for the LSP `initialize` request to prevent the application from hanging if `rust-analyzer` becomes unresponsive.
- **Git Status Parsing**: Fixed an off-by-one error when parsing Git rename/copy status entries, ensuring the correct destination path is displayed in the file tree.

### Changed
- **Performance Optimizations (V-3, V-4, V-5)**:
  - **Asynchronous File I/O**: Refactored synchronous `read_to_string` calls in the background event loop into asynchronous tasks, preventing UI micro-stutters during file system activity.
  - **Efficient Render Loop**: Shared `Settings` and `ProjectIndex` via `Arc` across viewports to eliminate redundant cloning of large structures in every UI frame.
  - **Syntax Highlighting Cache**: Implemented an MRU (Most Recently Used) cache in the `Highlighter` to avoid expensive full-text re-highlighting when the content hasn't changed.
  - **LSP Notification Throttling**: Introduced a 500ms debounce for `didChange` notifications, significantly reducing IPC traffic and CPU load during rapid typing.
  - **Event-Driven Sandbox**: Fully transitioned from 3s periodic sandbox polling to an efficient event-driven refresh triggered by file system watchers.
  - **Optimized Deduplication**: Improved sandbox file comparison performance by replacing O(n²) `Vec::contains` checks with an O(n) `HashSet` implementation.
- **Improved Watcher Logic**: Removed the sandbox from the global watcher ignore list to support real-time UI updates for AI-generated changes.
- **UI Enhancements**:
  - Added a "LSP initializing..." status indicator in the bottom bar to provide better feedback during startup.
  - Implemented automatic recovery of `.tmp` session and recent project files after an application crash.

## [0.6.1] - 2026-02-22

### Fixed
- **Critical CPU Usage Optimization**: 
  - **Asynchronous Sandbox Scanning**: Moved `get_staged_files()` scan from the main UI thread to a background thread. This eliminates periodic UI stuttering every 3 seconds during sandbox updates.
  - **Intelligent Staging Cache**: Implemented a "dirty" flag system for the sandbox cache. The scan is now triggered only periodically or immediately after a change is detected in the sandbox directory.
  - **LSP Diagnostic Throttling**: Limited UI repaints from LSP diagnostics to a maximum of 2 per second (500ms throttle), preventing CPU spikes during initial project indexing.
  - **Reduced UI Polling**: Increased the default `REPAINT_INTERVAL_MS` to 2000ms and Git refresh interval to 15 seconds.
  - **Terminal Event Limiting**: Capped terminal PTY event processing to 64 events per frame to maintain UI responsiveness during high-output build processes.
  - **Efficient Project Indexing**: Replaced full disk rescans with incremental updates for file system changes in the `ProjectIndex`.

### Added
- **Git UI Section**: Added a dedicated row for Git operations (status, diff, commit, etc.) in the left panel, visible when Sandbox mode is OFF.
- **Improved Promotion UX**: Bulk promotion of sandbox files no longer automatically opens new tabs for previously closed files, keeping the workspace clean.

## [0.6.0] - 2026-02-21

### Added
- **Complete Sandbox Mode**:
  - **Sandbox File Tree**: Added a "Project | Sandbox" toggle above the file tree to browse AI-generated files and directories without polluting the main workspace view.
  - **Sandbox Build/Run**: Introduced a "Sandbox ON/OFF" toggle in the Build panel. When active, all `cargo` commands and build runners execute within the sandbox.
  - **Error Parsing in Sandbox**: Build errors from sandbox execution are correctly parsed, allowing navigation to code proposed by AI before promotion.
- **Visual Feedback**: The UI now uses high-visibility indicators (yellow themes) when viewing or building in sandbox mode to prevent accidental confusion.
- **Enhanced Bulk Actions**: Completed localization and improved stability of the "Promote All" feature.

### Changed
- **Version Milestone**: Incremented major minor version to reflect the stability and completeness of the AI Sandbox integration.

## [0.5.9] - 2026-02-21

### Added
- **Bulk AI Promotion**: New "Promote All" button in the AI Staging Bar to approve all sandbox changes at once.
- **AI Diff Auto-show Toggle**: Added a global setting to enable/disable the automatic appearance of the AI Diff modal.
- **Multilingual Support**: Fully localized all new AI Sandbox and settings strings into Czech, Slovak, English, German, and Russian.

### Changed
- **Major Architecture Refactoring**:
  - Split the monolithic `src/app/mod.rs` by moving font management to `src/app/fonts.rs` and startup logic to `src/app/startup.rs`.
  - Refactored `src/app/ui/workspace/modal_dialogs.rs` into a modular structure under `src/app/ui/workspace/modal_dialogs/` (about, settings, ai, conflict, terminal).
- **Code Quality Improvements**:
  - Resolved multiple Clippy warnings regarding nested `if` statements (collapsible if).
  - Cleaned up unused imports and dead code in `LocalHistory` and `Startup` modules.
- **UI Enhancements**:
  - New checkbox in Settings for AI Diff behavior.
  - Improved feedback for bulk file promotion with summary toasts.

## [0.5.8] - 2026-02-21

### Added
- **AI Staging Bar**: Introduced a high-visibility yellow notification bar below the main menu that appears whenever the AI Sandbox contains unapproved changes.
- **Sandbox Staged Modal**: A new dialog listing all pending AI changes (New, Modified, and Deleted), allowing users to review them individually.
- **Sandbox Deletion Support**: The system now correctly detects when an AI agent deletes a file in the sandbox and propagates the deletion to the main project upon approval.
- **Auto-directory Creation**: When promoting new files from the sandbox, the editor now automatically creates any missing parent directories in the project root.
- **Success Confirmation Dialog**: Added a dedicated modal window confirming successful application of AI changes into the main project.

### Changed
- **Diff Flow Refactoring**: Moved the AI Diff modal handling to the start of the editor loop to ensure it works even when no files are open.
- **UI Render Order**: Relocated `render_dialogs` to the end of the workspace render cycle, ensuring modals have access to the latest state and improving interaction stability.
- **Improved Change Detection**: Transitioned from simple size-based checks to strictly time-based (mtime) comparison to prevent false positives during project-to-sandbox synchronization.

### Fixed
- **Stuck Modals**: Resolved an issue where AI Diff and Success modals could become unresponsive or block the UI after an action was taken.
- **Terminal Stretching**: Fixed the floating AI terminal window sometimes expanding to unreasonable heights by enforcing max-height limits based on the screen size.

## [0.5.7] - 2026-02-21

### Added
- **AI Safety Sandbox**: Implemented a "Shadow Sandbox" in `.polycredo/sandbox/`. All AI terminal tools now run in this isolated directory, preventing them from directly modifying the main project files and ensuring the workspace remains stable.
- **Local File History**: Introduced a Git-independent versioning system in `.polycredo/history/`. It automatically creates snapshots of files when they are opened, sent to AI context, or modified externally, providing a safety net for "undoing" AI changes.
- **Automatic AI Diff Gatekeeper**: The editor now automatically detects when an AI tool modifies a file in the sandbox. It then presents the changes in the AI Diff Preview modal, allowing the user to review, accept, or reject them before they are applied to the main project.

### Changed
- **DiffAction Refactoring**: Refactored the AI Diff logic to use a unified `DiffAction` enum and `EditorUiResult` structure. This ensures safer communication between the Editor and Workspace, correctly handling promotion of sandbox files to the real project.
- **Dead Code Cleanup**: Removed unused `#[allow(dead_code)]` attributes from `LocalHistory` and `Sandbox` modules, and implemented automatic 50-version history cleanup on project initialization.

### Fixed
- **Compilation Stability**: Resolved kaskading compilation errors caused by inconsistent return types in the AI Diff modal integration.

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
