## [1.0.2-dev] - 2026-03-05

### Added
- **Sandbox Instant Apply**: Sandbox mode now applies immediately after clicking Save in Settings — no project reopen required. Terminals restart, file tree reloads, and open tabs are remapped to the new root.
- **Tab Remap Toast**: After sandbox mode switch, a toast offers "Remap Tabs" / "Keep Current" — unresolvable tabs (files missing in new root) are marked as deleted. Pending remap state cleans up automatically if the toast expires without interaction.
- **Sandbox OFF Confirmation**: Switching sandbox OFF now shows a confirmation dialog. If another dialog is open, the apply can be deferred via "Apply Later" toast.
- **Staged Files Guard**: Sandbox OFF is blocked when staged git changes exist — a dialog explains the block and prompts to resolve staged files first.
- **Sync on Sandbox ON**: Enabling sandbox mode offers an automatic project-to-sandbox sync dialog. Sync runs in a background thread; result is shown as a toast.
- **Persist Failure Recovery**: If sandbox mode fails to persist to disk, a toast offers "Revert" or "Keep Temporarily" — the runtime state never diverges silently from disk.
- **Multi-Window Propagation**: Sandbox mode changes propagate to all open windows of the same project via `settings_version` mechanism.

### Changed
- **Sandbox Mode Persistence**: Sandbox mode is now stored in `settings.toml` and applied consistently on project reopen (previously session-only).
- **Sandbox Tooltip**: The sandbox toggle in Settings now has a full-row hover target for easier tooltip discovery.
- **Sandbox Reopen Note**: The inline note about terminal restart on reopen is no longer visually suppressed (`small()` removed) — text is fully readable.
- **Terminal Label Timing**: Terminal label reflects the new mode immediately on creation of a new instance; the old process finishes gracefully in `retired_terminals`. This is intentional behavior, documented in `apply_sandbox_mode_change()`.

## [1.0.0] - 2026-03-05

### Added
- **Light Mode Variants**: Three selectable light palette variants — Warm Ivory (warm cream), Cool Gray (GitHub/VS Code style), and Sepia (brownish-beige). Selectable via card picker in Settings.
- **Light Variant Card Picker**: Visual card selector in Settings panel (visible only in light mode) with color swatch, localized name, and selected-state indicator. Selection applies instantly as live preview.
- **Terminal Variant Toning**: Terminal background is tonally adjusted per active light variant. Warm Ivory uses a dedicated warm cream base (`#f5f2e8`, blend 0.55) for a visually distinct feel.
- **Variant-Aware Git Colors**: File tree git status colors blend with the active variant's `panel_fill` and `faint_bg_color` for consistent tonal harmony across all three variants.

### Changed
- **Settings Persistence**: Theme and variant are now stored in canonical `settings.toml`. Legacy `settings.json` is read on first launch for migration, then replaced. Backward-compatible — old configs without `light_variant` default to Warm Ivory.
- **Settings Save/Cancel Semantics**: Opening Settings captures a snapshot of current theme. Cancel restores the snapshot immediately; Save persists only when theme fingerprint actually changed.
- **Live Preview**: Dark/light toggle and variant selection apply instantly across all open windows via `settings_version` bump — no restart required.
- **Syntax Highlighting**: Light mode now uses the `Solarized (light)` syntect theme instead of the dark `base16-ocean.dark`. Theme switches without editor restart; highlighter cache is invalidated only on actual theme change.
- **Terminal Theming**: Both Claude panel and Build terminal now use an explicit light palette in light mode (light background, dark foreground, sufficient contrast). Theme applies at runtime on every render frame without PTY restart.
- **Terminal Scrollbar**: Scrollbar colors are derived from active `egui::Visuals` instead of hardcoded dark values.
- **Floating Terminal Frame**: `StandardTerminalWindow` frame fill is derived from `ctx.style().visuals.panel_fill` — no longer hardcoded dark in light mode.
- **Status Bar Contrast**: Primary and secondary text in the status bar is derived from `ui.visuals()`. Diagnostic and save/LSP state accents branch by `dark_mode` for readable contrast in both themes.
- **File Tree Git Colors**: Git status colors (`M`/`A`/`??`/`D`) use an explicit semantic model (`GitVisualStatus`) with separate light/dark palettes. Untracked files are now clearly visible in light mode.
- **Tab Unsaved Indicator**: The `●` indicator in editor tabs renders through the active theme text color — no hardcoded light color.

## [0.9.1] - 2026-03-04

### Added
- **Discard Confirmation Dialog**: Implemented a global confirmation dialog when discarding unsaved changes in Settings, Plugins, or New Project Wizard.
- **Improved Localization**: Added new strings for confirmation dialogs across all supported languages (CS, EN, SK, DE, RU).

### Changed
- **Unified Button Layout**: Implemented a standardized button layout across all modal dialogs. "Close" or "Discard/Quit" buttons are now consistently positioned on the far right, with action buttons to their left.
- **Button Renaming**: Renamed "Cancel" button to "Discard" (EN) / "Storno" (CS/SK) to better reflect the action of discarding local drafts.
- **Modal Infrastructure**: Introduced `ui_footer_actions` and `ModalFooter` helper in `StandardModal` to eliminate code duplication and enforce UI consistency.
- **Refactored Dialogs**: Updated all major dialogs (Settings, Plugins, Project Wizard, Search, LSP, Startup, etc.) to use the new unified footer system and removed redundant/duplicate buttons.

## [0.9.0] - 2026-03-04

### Added
- **Terminal Activity Indicator**: Added a visual cue (dot •) to terminal tab labels to indicate new unread output in background tabs.
- **Typing FPS Cap**: Implemented a smart repaint throttle that caps the UI at ~30 FPS during active typing, significantly reducing CPU spikes during rapid text entry.

### Changed
- **Repaint Gate (Performance)**: Major optimization of the UI render loop. The editor is now strictly event-driven in idle states, with a 2-second fallback repaint when unfocused or minimized.
- **Background Isolation**: Throttled all background-to-UI repaint requests (AI chat, indexing, plugin host, system stats) to a maximum of 10Hz (100ms), preventing "repaint storms" during intensive background tasks.
- **Terminal Throttling & Batching**: PTY event processing is now time-budgeted (max 2ms per frame) and batched into single writes to the terminal backend, ensuring a smooth UI even during massive console output (e.g., `cat` of large files).
- **PTY Lifecycle Management**: Improved process group termination. Closing a terminal tab now reliably kills the entire process group (SIGTERM to -PID), preventing zombie processes.

### Fixed
- **Accesskit Overhead**: Disabled `accesskit` and `web_screen_reader` features in `eframe` to eliminate unnecessary background accessibility processing and unsolicited repaints.
- **Path Detection Optimization**: Optimized clickable path detection in the terminal by caching results at the line level, reducing regex overhead during mouse movement.

## [0.8.5] - 2026-03-03

### Added
- **macOS cross-compilation support**: Added ability to build Apple Silicon (aarch64) and Intel (x86_64) binaries from Linux using `cargo-zigbuild`.
- **Universal Binary & DMG**: `build-all.sh` now creates a macOS `.app` bundle with a Universal Binary (via `lipo`) and packages it into a `.dmg` disk image (via `create-dmg` or `genisoimage`).
- **SDK Stub Generation**: The build script automatically generates TBD stubs for `libobjc` and core macOS frameworks (Foundation, AppKit, etc.) to satisfy the Zig linker without requiring a full macOS SDK.
- **macOS Dependency Wizard**: `DependencyWizard` now includes a specialized "macOS Build Dependencies" installer that handles `cargo-zigbuild`, `zig`, `rustup` targets, and `llvm` (lipo).
- **macOS Build Menu**: New submenu in **Build** menu for macOS-specific actions.

### Changed
- **BuildAllModal improvements**: Updated log parsing to correctly identify the current step from the script's box-drawing output.
- **Localization**: Added missing macOS-related strings to all supported languages (cs, en, de, ru, sk) and updated project templates in German.

## [0.8.4] - 2026-03-03

### Added
- **Selective package build**: `BuildAllModal` now opens with a ComboBox to choose between building all packages or a single format (.deb, .rpm, .flatpak, .snap, .AppImage, .exe, .pkg). Build no longer starts automatically — the user clicks **Spustit** to begin and **Znovu spustit** to rebuild with the same or different selection.
- **Live step indicator**: During a build, a highlighted header with spinner shows the currently active step (e.g. `3/7  Flatpak — .flatpak`), parsed from the script's box-drawing output. Before the first output arrives, the selected package label is shown instead.
- **`--only=<pkg>` flag in `build-all.sh`**: All seven build steps can be individually skipped via `--only=deb|rpm|flatpak|snap|appimage|exe|freebsd`. Steps not matching the selection are silently bypassed using the new `only_matches()` helper.

### Fixed
- **FreeBSD cross-compile path**: `cross build` now receives `--target-dir target`, ensuring the binary lands in `target/x86_64-unknown-freebsd/release/` (the default cross output location). `fpm` source path updated accordingly — previously pointed to `~/.cache/…` which was never populated by cross.

## [0.8.3] - 2026-03-03

### Added
- **Build All Packages** (`scripts/build-all.sh`): New script that builds all distribution formats in a single run — .deb, .rpm, .flatpak, .snap, .AppImage, .exe, and FreeBSD .pkg. Features live colored output, per-format logs in `target/dist-logs/`, graceful error handling (continues on failure), and a final summary listing successes, skipped (missing tools), failures, and required manual actions. Supports `--no-upload` flag.
- **"All Packages" menu item**: Added to the bottom of the Build menu (with separator) to trigger the new script from within the editor.
- **BuildAllModal**: New `StandardModal`-based dialog that runs `build-all.sh` in a background thread, streams live output with ANSI stripping and syntax-aware line coloring (green=success, red=error, yellow=skipped, blue=section headers), spinner during run, and a Close button that activates only after completion.

### Changed
- **Cargo artifacts relocated**: `build.rs` now writes `target-dir = "~/.cache/polycredo-editor/target"` into `.cargo/config.toml`, moving all Cargo compilation artifacts out of the project tree. Only `target/debian/` (deb staging) and `target/dist/` (final packages) remain in the project.
- **`target/dist/` cleanup**: `build-all.sh` removes stale packages from `target/dist/` before starting a new release build.
- **`target/debian/` layout**: `.deb` build staging (`PKG_BUILD_ROOT`) now lives in `target/debian/` (intermediate files); the final release `.deb` goes to `target/dist/` (`DEB_BUILD_TYPE=deb`), while dev builds with build number suffix go to `target/debian/` (`DEB_BUILD_TYPE=deb-dev`).
- **Cross-compile paths updated**: Exe, FreeBSD, and AppImage build commands now reference binaries from `~/.cache/polycredo-editor/target/` instead of `target/`.
- **`build-deb.sh`**: `BIN_SOURCE` now respects `CARGO_TARGET_DIR` env variable.

### Fixed
- **FreeBSD `fpm`**: Moved `-p` output path flag before the source argument to prevent misinterpretation as a positional path.
- **Privilege elevation**: All wizard install commands now use a standardized `pkexec` → `sudo -n` → error fallback via `apt_install_cmd()`.
- **RPM on Debian**: Wizard now detects `dnf` vs `apt-get` and uses the correct package manager.
- **Flatpak artifacts**: Builder state, repo, and build directories relocated to `~/.cache/polycredo-editor/flatpak/` to keep the project tree clean.
- **Snap build**: Added `/snap/bin` to PATH in the snap build command.
- **LXD configure**: Now opens the Dependency Wizard modal instead of running in the build terminal.

## [0.8.0] - 2026-02-28

### Added
- **Multi-Platform Distribution**: Implemented comprehensive packaging support for all major platforms: Windows (.exe), Debian/Ubuntu (.deb), Fedora (RPM), Arch Linux (AUR), AppImage, Flatpak, Snap, and Archive (.tar.gz).
- **Snapcraft & LXD Integration**: 
    - Added automated installation of `snapd`, `lxd`, and `snapcraft` via the Dependency Wizard.
    - Implemented LXD configuration tool in the Build menu to simplify build environment setup.
    - Enhanced Snap build process with automatic fallback to `--destructive-mode` and `sg lxd` group switching for immediate permission access.
    - Created standard `snap/snapcraft.yaml` configuration using modern `core24` base.
- **Unified Build Protection**: The Build menu is now intelligently disabled when in "Sandbox ON" mode or when unpromoted changes exist, preventing incomplete builds.
- **Unified Output**: All distribution packages are now automatically collected in a single `target/dist/` directory.
- **Windows Integration**: High-quality multi-resolution icons embedded into Windows executables with full cross-compilation support.
- **Real-time Dependency Tracking**: Submenus in the Build menu now show live status (✅/❌) of required system tools.
- **Dependency Wizard**: Extended to support automated installation for all new packaging formats across different Linux distributions.
- **Icon Generation**: Automated generation of vibrant, high-contrast PNG icons (16px to 256px).

### Changed
- **Modular Menubar**: Refactored the monolithic `menubar.rs` (900+ lines) into a clean, modular directory structure under `src/app/ui/workspace/menubar/`, improving maintainability and code clarity.
- **UI/UX**: Completely reorganized the Build menu into platform-specific submenus for each platform (Debian, Arch, Fedora, AppImage, Flatpak, Snap, Windows, Archive).
- **Tool Availability Check**: Implemented automatic startup and periodic background checks for all packaging tools, showing real-time status (✅/❌) in submenus.
- **Windows IPC**: Switched to local TCP sockets (127.0.0.1) for enhanced reliability on Windows.
- **Localization**: Synchronized all localization keys across Czech, English, German, Russian, and Slovak.
- **RPM/AUR Packaging**: Optimized configuration for native package generation.

## [0.7.31] - 2026-02-28

### Added
- **Build Menu**: Introduced a new top-level "Sestavit" (Build) menu for streamlined access to packaging tools.
- **Cross-Platform Packaging**: Added direct build actions for `.deb`, `.rpm`, `.AppImage`, `.tar.gz` (Linux/BSD), and `.exe` (Windows) formats.
- **Improved Menu Organization**: Moved system dependency installation tools (NSIS, rpmbuild, appimagetool) from the Help menu to the new Build menu for better logical grouping.
- **Multi-language Support**: Fully localized the new Build menu and its actions into Czech, English, Slovak, German, and Russian.

## [0.7.30] - 2026-02-28

### Added
- **AI Semantic Indexing**: Integrated **Stop Indexing** functionality with a UI button and progress indicator.
- **AI Shared Model**: Implemented BERT model sharing across multiple project viewports via `AppShared`, significantly reducing RAM usage when multiple projects are open.
- **AI Methodology**: Translated `AI_GUIDE.md` into English to maintain technical consistency across the codebase.

### Changed
- **Performance**: Optimized semantic vectorization by switching from `F32` to `F16` (`DType::F16`), reducing model weight memory footprint by 50%.
- **Performance**: Implemented **incremental indexing** using `xxh3` file hashing and `HashMap` lookups, reducing project re-scan complexity from O(N²) to O(N).
- **Behavior**: Disabled automatic full semantic re-indexing on startup; it now only triggers if the cache is empty or manually requested.
- **Maintenance**: Moved all audit logs to a dedicated `.audit/` directory to clean up the project root.

## [0.7.29] - 2026-02-28

### Fixed
- **Linux Distribution (Discover)**: Resolved an issue where KDE Discover failed to install the `.deb` package due to blocking `debconf` prompts in non-interactive mode.
- **License Visibility**: Added **AppStream metadata** (`metainfo.xml`) to the Debian package, enabling software centers like Discover to correctly identify the AGPL-3.0 license and project details.
- **Package Metadata**: Added an explicit `License` field and homepage to the `.deb` control file for better compatibility with various Linux package managers.

### Changed
- **Cargo Metadata**: Added the `license` field to `Cargo.toml`.

## [0.7.28] - 2026-02-28

### Added
- **Legal Framework**: Established a robust legal foundation for the project to support future commercial growth and potential sale.
- **AGPLv3 License**: Adopted the GNU Affero General Public License v3.0 to ensure the community edition remains open and transparent.
- **Contributor License Agreement (CLA)**: Implemented a mandatory CLA for all contributors, securing the right for project owners to relicense or sell the project in the future.
- **Dual-Licensing Strategy**: Introduced `LICENSING.md` to clearly communicate the "Open Core" model (AGPL for community, custom licenses for enterprise/sponsors).

## [0.7.27] - 2026-02-28

### Fixed
- **.deb Package Startup**: Resolved a critical issue where the application failed to start on some Linux systems due to an invalid `AppID` property in the `systemd-run` wrapper. Added a robust fallback to direct execution if `systemd-run` or the user session bus is unavailable.
- **Icon Duplication**: Confirmed and unified `app_id` in the editor core with `StartupWMClass` in the `.desktop` file to ensure the application is correctly grouped under a single icon in desktop launchers.

### Added
- **Support & Sponsorship Modal**: Introduced a dedicated "Support Development" modal dialog to the Help menu and status bar. This provides users with a clear way to follow the project on GitHub and contribute to its future development.
- **Support Entry Points**: Added a heart icon (❤️) to the right side of the status bar and a new "Support Development" item to the Help menu for easy access to the sponsorship information.
- **Internationalization**: Fully localized the support modal and all related UI strings into Czech, English, Slovak, German, and Russian.

### Changed
- **Inclusive AI Vision**: Updated all "Local AI" references in the support dialogue to "Secure AI Assistant Integration" to better reflect the editor's support for various AI backends (Ollama, Gemini, etc.) while maintaining a focus on privacy and security.

## [0.7.26] - 2026-02-28

### Added
- **Dynamic Resource Management**: Integrated automatic CPU/RAM limit calculation into `build.rs` to maintain system responsiveness during development (66% cores for build, 50% for runtime).
- **Auto-Configurator**: The build system now automatically generates and maintains `.cargo/config.toml` based on the current machine's hardware.
- **Resource-Aware Packaging**: Updated `.deb` build scripts with a dynamic wrapper that calculates and enforces resource limits (via systemd-run) at the OS level on the target machine.
- **Enhanced Dev Runner**: Added `run_limited.sh` with automatic `--new-instance` flag and dynamic resource throttling.

## [0.7.25] - 2026-02-27

### Changed
- **AI Chat Prompt — grows upward**: The prompt input now expands upward (shrinking the conversation history) instead of pushing content downward. Height is measured each frame via `ui.memory` so the history area adjusts dynamically with no fixed reservation.
- **AI Chat Prompt — scrollbar after 5 lines**: The prompt is capped at 5 visible rows. Beyond that a vertical scrollbar appears inside the input box and the history area stops shrinking. `stick_to_bottom` keeps the cursor row always visible while typing.
- **AI Chat — terminal-style layout**: The window now behaves like a terminal. History starts at the top and grows downward; the prompt is pinned immediately below the last message. The remaining space below the prompt is an explicit gap (`ui.add_space`) that shrinks as history grows. Once history fills the available height the scrollbar appears and the gap disappears. The gap is computed each frame from `scroll_out.content_size.y` stored in `ui.memory` (one-frame lag, imperceptible). The window always fills its full height.

## [0.7.24] - 2026-02-27

### Added
- **AI Plugin Quick-Launch Bar**: Added a compact toolbar at the bottom of the file tree panel (left sidebar). The bar contains a ComboBox listing all enabled AI plugins (`ai_agent` type), a **▶ Spustit** button to instantly launch a new chat session with the selected plugin, and a **⚙** button to jump directly to that plugin's settings in the Plugin Manager. The bar is hidden automatically when no AI plugins are registered.

### Fixed
- **Plugin Bar — Provider Switch**: The **▶ Spustit** button now correctly resets the conversation and initializes a new session for the plugin selected in the ComboBox. Previously it would reopen the last-used provider instead of the one chosen in the dropdown (missing `NewQuery` reset and `ai_focus_requested` flag, matching the behaviour of the Plugins menu).

## [0.7.23] - 2026-02-27

### Added
- **AI Chat Assistant Terminal**: Implemented a new, fully independent AI Chat module (`ui/terminal/ai_chat/`) built on `StandardTerminalWindow`. The assistant now opens as a non-blocking floating window alongside the editor, replacing the old modal dialog. Includes conversation history, monologue view, prompt input with history navigation, settings panel, tool approval UI, and an AI Inspector panel (raw payload viewer).

### Fixed
- **Terminal Keyboard Focus (Left Click)**: Bottom build terminal and other docked terminals now correctly receive keyboard focus on a left-click, not just right-click. The focus reset logic was too aggressive — it now only transfers focus back to the editor on an explicit click elsewhere.
- **Floating Window Focus Stealing**: Fixed a critical bug in `StandardTerminalWindow` where multiple open floating windows (e.g., AI Chat + Claude Float + Build Float) would steal focus from each other. The root cause was `ui.ui_contains_pointer()` inside the render closure, which does not account for egui's z-ordering. Replaced with `inner.response.rect.contains(pos)` + `ctx.layer_id_at(pos)` to correctly identify the topmost window under the pointer.

### Removed
- **Old AI Chat Modal**: Deleted the legacy `modal_dialogs/ai_chat/` module, which has been fully superseded by the new `terminal/ai_chat/` implementation.

## [0.7.22] - 2026-02-27

### Fixed
- **Ollama Stability**: Fixed "Internal Server Error" crashes in Ollama caused by context overflow and RAM exhaustion during large project searches.
    - **Search Result Limiting**: Reduced the number of search snippets returned to the AI to prevent massive prompt growth.
    - **Binary File Exclusion**: Updated `search_project` to strictly ignore large binary files (`.gguf`, `.bin`, `.model`) even if they are untracked.
    - **Plugin-side Truncation**: Added a safety truncation buffer in the Ollama WASM plugin to ensure tool outputs never exceed safe communication limits.
    - **Improved .gitignore**: Added AI model files to the project's ignore list to prevent them from being indexed or searched by mistake.

## [0.7.21] - 2026-02-27

### Changed
- **AI Chat UX**: AI Chat Assistant no longer blocks the background editor and panels. It now functions as a non-blocking floating window, allowing simultaneous code editing and chatting.

## [0.7.20] - 2026-02-27

### Fixed
- **Dependency Resolution**: Fixed a compilation error caused by a non-existent `quantized` feature in `candle-core` version 0.8.
- **Code Quality**: Resolved a Clippy warning in `src/app/ui/terminal/right/ai_bar.rs` regarding inefficient single-character `push_str` usage.
- **Formatting**: Unified codebase formatting across all modules to satisfy the project's quality gate.

## [0.7.19] - 2026-02-26

### Added
- **Custom AI CLI Agents**: Users can now define their own AI assistants (Gemini, Claude, Aider, etc.) in Settings under a new "AI" category.
- **Dynamic AI Bar**: The AI bar now dynamically loads agents from settings and supports custom launch commands with arguments.

### Changed
- **Simplified AI Bar**: Removed periodic status checks for AI tools to ensure a faster and cleaner UI.

## [0.7.18] - 2026-02-26

### Refactored
- **Massive Architectural Modularization**: Executed a comprehensive restructuring of the most complex UI and backend components to strictly adhere to the < 700 lines-per-file guideline, dramatically improving maintainability.
    - **Plugin Registry (`plugins.rs`)**: Split 1200+ lines into `types.rs`, `security.rs`, and a dedicated `host/` module for WASM capabilities.
    - **AI Chat Dialog (`ai_chat.rs`)**: Decomposed into `mod.rs`, `render.rs`, `approval.rs` (complex diff logic), `inspector.rs`, and `logic.rs`.
    - **AI Chat Widget (`chat.rs`)**: Broken down into `input.rs`, `conversation.rs`, `render.rs`, and `settings.rs`.
    - **Workspace State (`state.rs`)**: Separated into `types.rs`, `actions.rs`, and isolated `init.rs` containing complex semantic indexer background threads.
    - **File Tree (`file_tree.rs`)**: Divided into `node.rs`, `render.rs`, `ops.rs` (I/O logic), and `dialogs.rs`.
    - **LSP Renderer (`render_lsp.rs`)**: Split 860+ lines into `hover.rs`, `completion.rs`, `navigation.rs`, and `typing.rs` under a new `editor/render/` directory structure.
    - **Global Dialogs (`dialogs.rs`)**: Categorized into `privacy.rs`, `wizard.rs`, `startup.rs`, and `confirm.rs`.
- **Unified Terminal System**: Consolidated fragmented terminal logic (AI and Build) into a unified `src/app/ui/terminal/` architecture.
    - **`instance/`**: Core PTY, rendering, and input handling.
    - **`right/`**: AI agent panel with context-aware command bar.
    - **`bottom/`**: Universal build and utility panel featuring segmented `build_bar`, `compile_bar` (OS specific), and `git_bar`.
    - **Floating Terminals**: Added ability to undock the build terminal into a floating workspace window.
    - **Focus Stability**: Improved egui pointer intersection logic to reliably maintain terminal focus on hover across all panel configurations.
- **Search Performance Upgrade**: Replaced the standard `grep` with **ripgrep (`rg`)** for the `search_project` tool. This provides significantly faster search results and better integration with project structure (respecting `.gitignore`).
- **Build System**: Added `ripgrep` as a mandatory dependency for the `.deb` package to ensure the search functionality works out of the box on Linux systems.

### Fixed
- **WASM Plugin Stability**: Resolved a critical issue where the Gemini plugin would crash with a "wasm backtrace" error at the end of an action.
    - **Log Auto-Approval**: The host now automatically approves writing to `.gemini_trace.log` and other `.log` files, preventing the UI from blocking or failing during the final trace save.
    - **Safe Shutdown**: Fixed a trap condition in the Extism guest by ensuring the host doesn't return errors for non-critical background operations.

## [0.7.17] - 2026-02-25

### Added
- **AI Monologue UX Improvements**: Significant enhancements to the "thinking" process visualization.
    - **Proper Word Wrapping**: Implemented strict width constraints for thought blocks, fixing the "single long line" issue prevalent with Gemini 1.5 Pro outputs.
    - **Visual Hierarchy**: "Step" lines (tokens usage) are now rendered with a smaller font (80% size) and italicized to distinguish them from the AI's actual reasoning.
    - **Rich Markdown Support**: Replaced restrictive blockquotes with a manual sidebar line, restoring full color and bold text support within the monologue.
    - **Paragraph Preservation**: Monologues now correctly handle empty lines and multiple parts, rendering clear paragraph breaks instead of merging text.
- **Advanced Code Review**: The `replace` tool approval dialog is now much more professional.
    - **Smart Context**: Automatically extracts and displays 3 lines of code before and after the targeted block directly from the file.
    - **Real Line Numbers**: Every line in the diff (including context) now displays its actual line number from the source file.
    - **Color-Coded Diffs**: Added full-width background highlighting for added (green) and removed (red) lines, improving readability during surgical edits.
- **Dynamic Language Enforcement**: 
    - The AI agent is now strictly instructed using the human-readable language name (e.g., "Czech / Čeština") dynamically retrieved from the "Agent Settings".
    - Removed hardcoded language lists, making the system future-proof for any new translations.
    - Added a "STRICT LANGUAGE RULE" to the Gemini plugin to prevent the model from switching back to English during tool use.

### Changed
- **Clean UI**: Removed redundant gray backgrounds and separators in the chat history for a more transparent and modern look.
- **Surgical Mandate**: Strengthened instructions against using `write_file` for existing source code, making `replace` the primary tool for modifications.

## [0.7.16] - 2026-02-25

### Added
- **AI Chat UI Redesign**: Major UX overhaul for a professional terminal-like feel.
    - **Flowing Layout**: Conversation and prompt now grow dynamically from the top.
    - **Upward-Expanding Prompt**: Input field grows upwards as you type, keeping the current line stable.
    - **Unified Dark Theme**: Entire modal body matches the viewer's dark background.
    - **Consistent History**: User questions in history are now wrapped in blue-gray blocks matching the prompt style.
    - **Bottom Settings**: Agent settings moved to the footer with a "push-up" effect on the history.
- **Visual Refinements**: 
    - Visible separators between messages and above the status bar.
    - Improved ASCII logo rendering with tight line spacing.
    - Custom padding and light text color for the prompt area.

## [0.7.15] - 2026-02-25

### Added
- **AI Subsystem Refactoring**: Complete modularization of AI logic.
    - Centralized engine in `src/app/ai/` (manager, types, tools).
    - Unified `AiChatWidget` for consistent terminal-like UI.
    - Generic `AiChatModal` replacing the hardcoded Gemini modal.
- **Large File Visualization**: Visual warnings in sandbox file tree.
    - Highlighting for files > 500 lines (white underline + count).
    - Extra emphasis for files > 1000 lines (thick underline).

### Changed
- Improved code modularity by separating UI widgets from AI core logic.
- Increased visibility of internal editor types to allow unified context gathering.

## [0.7.14] - 2026-02-25

### Added
- **Surgical Code Replacement Tool**: Introduced a new `replace` tool for AI agents that allows precise text replacement in existing files.
    - **Reduced Token Usage**: Agents no longer need to rewrite entire files for small changes, significantly saving context window and API costs.
    - **Ambiguity Protection**: The tool requires matching an exact block of code (with 3+ lines of context recommended) before applying changes, preventing accidental overwrites.
    - **Host-Guest Protocol**: Implemented `replace_project_file` host function in Rust and exposed it to WASM plugins via the Extism PDK.
- **Improved AI Mandates**: Updated system instructions for all agents to prioritize `replace` for code modifications, reserving `write_file` exclusively for new files and reports.

### Fixed
- **Gemini Plugin Reliability**: Enhanced error handling in the Gemini WASM plugin to gracefully report replacement failures (e.g., when a code block isn't found) back to the agent for self-correction.

## [0.7.13] - 2026-02-25

### Added
- **Global Focus Shortcuts**: Introduced system-wide keyboard shortcuts for rapid navigation:
    - `Ctrl+Alt+E`: Focus Editor.
    - `Ctrl+Alt+G`: Focus Gemini CLI Agent.
    - `Ctrl+Alt+B`: Focus Build Terminal.
    - `Ctrl+Alt+A`: Focus AI Terminal (Claude).
- **Auto-Authorization for Plugins**: Plugins that require network access (like Gemini) are now automatically authorized upon startup if they are enabled in the user settings, streamlining the "First Run" experience.
- **Smart Focus Tracking**: Implemented `FocusedPanel::Gemini` to explicitly track and manage keyboard focus for the CLI agent.

### Changed
- **Dialog-First Rendering**: Refactored the main workspace render loop to process floating dialogs and windows *before* the editor and side panels. This ensures that modals like Gemini correctly intercept input and maintain focus without flickering.
- **"Focus Follows Mouse" for Agents**: Unified the focus behavior across the editor. Moving the mouse over the Gemini agent window now automatically activates it, matching the behavior of built-in terminals.
- **Improved Plugin Loading**: Updated the loading logic to prioritize local `plugins/` directories relative to the project root, ensuring isolation between production and sandbox environments.

### Fixed
- **Plugin Duplication**: Resolved a bug where the Gemini plugin could be loaded multiple times from different directory levels. Added a robust ID-based duplicate check in the `PluginManager`.
- **Terminal Visibility in Modals**: Fixed an issue where the Build and AI terminals would stop rendering or become unresponsive when the Gemini modal was open.
- **Aggressive Focus Theft**: Corrected terminal focus logic to prevent "hover-based" focus stealing while a floating window is actively being used.

## [0.7.12] - 2026-02-25

### Performance
- **CPU Idle Optimizations**: Resolved the root cause of ~10% CPU usage and laptop heating in idle state. Combined effect of all changes is estimated at 70–80% reduction in idle CPU load.
    - **egui_term Repaint Throttle**: Changed `request_repaint()` to `request_repaint_after(16ms)` in the PTY event loop. This was the dominant cause — each idle shell generated continuous PTY events triggering immediate full-UI repaints at near 60fps even without user activity.
    - **Resize Guard**: `TerminalView::resize()` now compares the new layout size against the last known size before issuing a `BackendCommand::Resize`. Previously, `process_command` set `dirty = true` unconditionally every frame even when the terminal size had not changed.
    - **Lazy Terminal Initialization**: Shell processes (AI CLI, build terminal) are now spawned only when their respective panels are first opened, not immediately on workspace load.
    - **Terminals Hidden by Default**: `show_right_panel` and `show_build_terminal` now default to `false`. Terminals start on demand, reducing base resource usage from first launch.
    - **Conditional Repaint**: The periodic `request_repaint_after(2s)` is now conditional — UI is only force-refreshed when an active background operation is in progress (AI loading, build, git fetch, semantic indexing).
- **Semantic Indexer Throttle**: Inter-chunk sleep during BERT embedding increased from 2ms to 10ms, reducing thermal load during indexing. Added per-file debounce (30s cooldown) for re-indexation triggered by file watcher events, preventing the AI-agent write cycle from causing continuous CPU spikes.
- **Git Polling Interval**: Background git status polling interval increased from 10s to 30s.
- **File Watcher Deduplication**: Sandbox path is no longer added as a separate watcher root when it is already a subdirectory of the project root, eliminating duplicate filesystem events.

## [0.7.11] - 2026-02-25

### Changed
- **Deep-Muted AI Theme**: Updated the AI Assistant's color palette to a more comfortable, professional deep-muted theme. Base text is now a soft gray (`rgb 175, 175, 175`), and accent colors (purple, blue, green) have been toned down for extended use.
- **Polished Monologue UI**:
    - **Clean Formatting**: Completely removed Markdown blockquote markers (`>`) from reasoning traces, replacing them with a single, clean vertical accent line (`│`).
    - **Visual Structure**: Monologue steps are now grouped into subtle gray frames with **12px padding** and a **2px left border** in the text color, creating a clear logical separation from the main conversation.
    - **Step Headers**: "Step X" markers are now rendered in discrete italics without redundant symbols.

### Fixed
- **Monologue Syntax Highlighting**: Resolved an issue where file paths and backticked code would lose their purple highlighting when inside reasoning blocks. Highlighting is now robustly preserved across all UI elements.

## [0.7.10] - 2026-02-24

### Fixed
- **AI Modal Overflow**: Added a dedicated `ScrollArea` for agent action approval details. This prevents long code proposals or command outputs from pushing the approval buttons off-screen.
- **Improved Approval Visibility**: Enhanced the "waiting for approval" state with a yellow border, warning icon, and clearer status messaging.
- **Monologue Readability**: Increased the max height of the AI's internal reasoning monologue (thinking trace) and ensured it is properly scrollable and wrapped, resolving issues where tool outputs were hard to navigate.

## [0.7.9] - 2026-02-24

### Added
- **True Safe Mode (Read-Only Editor)**:
    - **Physical Write Protection**: The editor now physically prevents typing in files outside the sandbox when Safe Mode is enabled. Text can still be selected and navigated, but any changes are discarded before reaching the internal buffer.
    - **Visual Clarity**: Smart typing features (auto-indent, brace completion) are automatically disabled in read-only mode to prevent confusing UI behavior.
- **AI Agent Governance (Human-in-the-loop)**:
    - **Manual Action Approval**: Introduced a mandatory approval flow for "dangerous" AI actions. When an agent requests to write a file or execute a command, the UI displays a yellow warning bar with action details.
    - **Interactive Controls**: Added keyboard shortcuts for approval: `1` (Approve), `2` (Approve Always for this thread), `3`/`Esc` (Deny).
    - **Instant Cancellation**: Pressing `Esc` during the AI "thinking" phase now immediately terminates the agent process using a cancellation token, preventing infinite loops or unwanted exploration.

### Fixed
- **Notification Spam**: Resolved the "waterfall" effect where failed autosave attempts in Safe Mode would generate hundreds of error toasts. Autosave now silently ignores protected files, while manual save (Ctrl+S) still provides feedback.

## [0.7.8] - 2026-02-24

### Added
- **Sandbox-First Semantic RAG**:
    - **Dynamic Indexing**: The RAG system now indexes exclusively the `.polycredo/sandbox/` directory, ensuring the AI always works with its own most recent changes.
    - **Smart Exclusion**: Integrated `.gitignore` and user blacklist into the indexer. Added logic to prevent the indexer from ignoring its own sandbox root.
    - **Line-Segment Reading**: Added `line_start` parameter to `read_project_file`, allowing AI to navigate and read large files in chunks (avoiding 10k character truncation limits).
- **Professional Agent Mandates**:
    - **Hardcoded Integrity**: Added unchangeable system instructions to the Gemini plugin that mandate accuracy, prohibit hallucinations, and enforce the use of RAG for code discovery.
    - **Thinking Trace**: AI now logs its step-by-step reasoning to `.gemini_trace.log` in the sandbox for transparent debugging.
- **Enhanced UI Features**:
    - **Thread Export**: Added a "📋 Copy Thread" button to each AI response for easy clipboard export of Q&A pairs.
    - **Better Readability**: Enabled automatic text wrapping for both AI responses and the thinking monologue.

### Fixed
- **Plugin Protocol Reliability**:
    - **Multi-step Tool Calling**: Fixed a critical bug where tool definitions were lost after the first iteration, causing "malformed function call" errors.
    - **API Compatibility**: Updated tool responses to always return JSON objects (never raw arrays) to satisfy strict Google Gemini API requirements.
    - **Message Splitting**: Implemented automatic splitting of mixed AI responses (text + tool calls) into separate turns to prevent API rejection.
- **Branding Consistency**: Corrected the Gemini logo to display the actual model ID from settings instead of a hardcoded default.
- **Code Stability**:
    - Resolved compilation errors in `terminal.rs` caused by Alacritty 0.25 API changes.
    - Fixed Clippy warnings regarding collapsible if-statements and unused imports.

## [0.7.7] - 2026-02-24

### Added
- **Local Semantic RAG (Retrieval-Augmented Generation)**:
    - **Vectorized Project Search**: Integrated a local ML stack (`candle` + `BERT`) to enable meaning-based code search.
    - **Automatic Indexing**: Background process that chunks and vectorizes source code (30-line windows) with real-time progress modal.
    - **Smart Caching**: Index is persisted to `.polycredo/semantic_index.bin` and updated incrementally based on file modification times (`mtime`).
    - **Strict Filtration**: Optimized indexer to skip hidden files, binary data, and internal editor history to ensure high-quality search results and low CPU usage.
- **Project "Safe Mode" (Human-in-the-loop 2.0)**:
    - **Read-Only Protection**: The main project is now write-protected by default. Users are encouraged to work in the AI Sandbox.
    - **Sandbox-First Workflow**: In Safe Mode, the File Tree and Build Terminal automatically default to the Sandbox environment upon startup.
    - **Safety Overrides**: Safe Mode can be toggled in General Settings for users requiring direct project RW access.
- **AI Inspector**:
    - Introduced a dedicated **AI Inspector** panel within the Gemini modal (accessible via 🔍 icon).
    - Real-time visualization of the **full JSON payload** sent to the AI, including system prompts and raw conversation history.
    - Detailed **Context Trace**: Displays exactly what the AI sees, including lists of open files and current build errors.
- **Gemini CLI Branding & UX**:
    - **Polished ASCII Logo**: Implemented a dynamic, per-line color gradient for the PolyCredo logo.
    - **Metadata Refinement**: Consistent styling for Version, Model, and Plan info.
    - **"New Thread" (Nové vlákno)**: Rebranded "New Query" across all languages and implemented full thread/token reset logic.

### Changed
- **Conversational Continuity**: Updated Gemini plugin to support full conversation history via structured JSON payloads.
- **Typography**: Increased Markdown font size to **120%** and expanded line spacing for superior readability.
- **Token Efficiency**: Drastically reduced token usage by removing automatic file list/content from every prompt (moved to on-demand Tool use).

### Fixed
- **Focus Stealing**: Resolved issues where the AI modal would aggressively grab focus from background terminals.
- **UI Stability**: Fixed layout "jumping" in the Gemini dialog through proper SidePanel/CentralPanel orchestration inside modals.

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
