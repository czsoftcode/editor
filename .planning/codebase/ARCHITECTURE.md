# Architecture

**Analysis Date:** 2025-03-04

## Pattern Overview

**Overall:** Single-process multi-window GUI application with viewport-based architecture

**Key Characteristics:**
- Single process manages multiple workspace viewports via egui's deferred viewport system
- Shared state protected by `Arc<Mutex>` allows communication between viewports
- Layered architecture: UI rendering → workspace state → file I/O and backend services
- Modal dialog system for project creation, settings, and user confirmations
- Plugin registry system supporting WASM modules and CLI-based AI agents
- Sandbox environment for secure file operations with project root isolation

## Layers

**Application Layer:**
- Purpose: Entry point and main event loop orchestration
- Location: `src/main.rs`, `src/app/mod.rs`
- Contains: `EditorApp` struct, viewport initialization, IPC server setup
- Depends on: All lower layers
- Used by: eframe native window system

**Workspace Layer:**
- Purpose: Single-project editing environment with file tree, editor, terminals
- Location: `src/app/ui/workspace/`
- Contains: `WorkspaceState`, panel rendering, menubar, modal dialogs, project-specific settings
- Depends on: Editor, FileTree, Terminal, LSP Client, Watcher
- Used by: Application Layer (root and secondary viewports)

**UI/Presentation Layer:**
- Purpose: Render components and handle direct user interaction
- Location: `src/app/ui/editor/`, `src/app/ui/file_tree/`, `src/app/ui/terminal/`, `src/app/ui/dialogs/`
- Contains:
  - `Editor`: Multi-tab editor with syntax highlighting, search, LSP integration, diff viewing
  - `FileTree`: Directory tree with context menu (new, rename, delete, copy)
  - `Terminal`: PTY-based terminal (Claude panel + Build terminal)
  - Modal dialogs: About, Settings, AI, Plugins, Dependencies
- Depends on: Highlighter, LSP Client, Watcher, Build Runner
- Used by: Workspace Layer

**AI Integration Layer:**
- Purpose: Plugin management, AI agent execution, context generation
- Location: `src/app/registry/`, `src/app/ai/`
- Contains:
  - `Registry`: WASM plugin loader, agent registration, command registry
  - `AiManager`: System prompt generation, context payload assembly
  - Plugins: WASM modules (Gemini, Ollama) + host bindings (FS, Search, Security)
  - Agents: External CLI tools (Claude, Aider) with context-aware execution
- Depends on: Workspace State, File System, Build errors
- Used by: Terminal (AI Chat panel), AI dialogs

**File I/O & Watching Layer:**
- Purpose: Project file management, change detection, synchronization
- Location: `src/watcher.rs`, `src/app/sandbox.rs`, `src/app/local_history.rs`
- Contains:
  - `FileWatcher`: Monitors individual file changes via `notify` crate
  - `ProjectWatcher`: Detects project-level changes (new/deleted files)
  - `Sandbox`: Syncs between project root and `.polycredo/sandbox` working copy
  - `LocalHistory`: Stores undo snapshots of file content
- Depends on: File system
- Used by: Workspace State, Editor

**Build & Compilation Layer:**
- Purpose: Execute and parse build output
- Location: `src/app/build_runner.rs`
- Contains: `run_build_check`, error parsing for Rust/Symfony, build profile execution
- Depends on: File system, Terminal spawning
- Used by: Workspace (build toolbar, error list)

**Language Server Protocol Layer:**
- Purpose: Real-time code intelligence (hover, completion, diagnostics)
- Location: `src/app/lsp/`
- Contains: `LspClient`, message dispatch, hover state, completion state, diagnostics
- Depends on: External LSP server (e.g., rust-analyzer)
- Used by: Editor rendering, search, go-to-definition

**Persistence Layer:**
- Purpose: Settings, session state, recent projects
- Location: `src/settings.rs`, `src/ipc.rs`
- Contains:
  - `Settings`: User configuration (theme, fonts, custom agents, plugins blacklist)
  - Session storage: Recent projects, panel visibility, window geometry
  - IPC server: Coordinate singleton instance and recent projects list
- Depends on: File system (`~/.config/polycredo-editor/`)
- Used by: Application Layer (startup), Workspace Layer (settings panel)

**Configuration & Constants:**
- Purpose: Centralized tuning parameters
- Location: `src/config.rs`
- Contains: Font sizes, panel dimensions, repaint intervals, limits
- Used by: All layers

## Data Flow

**Project Open Flow:**

1. `main.rs` parses CLI args or detects existing instance
2. If secondary instance → `Ipc::open_in_new_window()` → primary receives request
3. Primary `EditorApp` creates new `WorkspaceState` via `init_workspace()`
4. `WorkspaceState` initializes:
   - `FileTree` loads root directory structure
   - `Editor` creates first empty tab
   - `FileWatcher` and `ProjectWatcher` begin monitoring
   - `Sandbox` syncs project root ↔ `.polycredo/sandbox`
   - `LspClient` spawns language server process (if available)
5. Secondary viewport created via `show_viewport_deferred()`
6. Session saved to `~/.config/polycredo-editor/session.json`

**File Edit & Save Flow:**

1. User types in editor → `Tab.content` modified, `Tab.modified = true`
2. After 500ms inactivity → autosave triggered
3. `Editor::autosave()` writes to `Tab.path` and disk
4. `SaveStatus` progresses: `Modified` → `Saving` → `Saved`
5. `FileWatcher` detects external changes (from Sandbox sync)
6. If conflict: show "External Change" modal (accept/reject/discard)
7. `LocalHistory` records undo snapshot after each successful save

**Build Execution Flow:**

1. User clicks Build/Run/Test in toolbar → `ProcessMenu::BuildAction`
2. `build_runner::run_build_check()` spawns cargo process
3. STDOUT/STDERR captured and parsed for errors (Rust parser recognizes `error[`, `warning:`)
4. Build terminal displays live output via PTY
5. Errors appear in left panel error list
6. Clicking error jumps editor to file:line and scrolls

**AI Agent Execution Flow:**

1. User types prompt in Claude panel
2. Selects provider (Gemini, Ollama, or CLI agent)
3. `AiManager::generate_context()` assembles payload:
   - Open files + active file content
   - Build errors (file + line + message)
   - Git branch and status
   - Settings (language, expertise role, reasoning depth)
   - Project structure snapshot
4. Payload sent to plugin (WASM) or CLI tool (external process)
5. Plugin responds with tool calls, thoughts, or structured actions
6. Actions execute: `replace_file`, `run_command`, `store_scratch`, `ask_user`
7. Responses displayed in AI chat panel
8. Token usage and reasoning tracked in inspector

**Sandbox Sync Flow:**

1. Project root and sandbox may diverge due to:
   - User edits in both locations
   - AI modifications in sandbox
   - External changes from git pull, dependency installs
2. `Sandbox` periodically scans both trees
3. Conflict detection: same file edited in both → show sync modal
4. User chooses: accept sandbox changes, keep project root, or manual merge
5. After resolution: files promoted/demoted accordingly

**Repaint Throttling Flow:**

1. If window unfocused or minimized → repaint every 2 seconds
2. If user typing → repaint every 33ms (capped at ~30 FPS)
3. If background work active (AI loading, git status, semantic indexing) → repaint every 2 seconds
4. Otherwise → only repaint on input (eframe default)
5. This prevents UI thread starvation during intense operations

**State Management:**

- **Persistent:** `Settings`, recent projects, session viewports (saved via eframe and custom JSON)
- **Ephemeral:** `Toast` notifications (4s auto-dismiss)
- **Shared:** `AppShared` holds `Registry`, settings version, BERT model (for semantic search)
- **Local:** `WorkspaceState` holds editor state, UI toggles, modal states; each viewport owns its `WorkspaceState`
- **Cross-viewport:** IPC channels for focus/actions/plugins; `Arc<Mutex>` for shared data

## Key Abstractions

**WorkspaceState:**
- Purpose: Encapsulates all state for a single open project
- Examples: `src/app/ui/workspace/state/mod.rs`
- Pattern: Central state machine updated by `render_workspace()` each frame

**Editor (Multi-tab):**
- Purpose: Multi-file editing with tabs, search, markdown preview, LSP integration
- Examples: `src/app/ui/editor/mod.rs`, `src/app/ui/editor/tabs.rs`
- Pattern: Vector of `Tab` structs; autosave debounced by 500ms

**FileTree:**
- Purpose: Directory navigation with context menu operations
- Examples: `src/app/ui/file_tree/mod.rs`, `src/app/ui/file_tree/node.rs`
- Pattern: Recursive `FileNode` tree with lazy loading and git status coloring

**Terminal (Dual):**
- Purpose: PTY-based shell + AI chat in separate tabs
- Examples: `src/app/ui/terminal/mod.rs`, `src/app/ui/terminal/instance/mod.rs`
- Pattern: Each terminal owns PTY instance; `egui_term` widget for rendering

**Registry (Plugin + Agent):**
- Purpose: Central authority for WASM plugins, CLI agents, commands, panels
- Examples: `src/app/registry/mod.rs`, `src/app/registry/plugins/mod.rs`
- Pattern: Separate registries for agents, commands, panels; plugins loaded from `.polycredo/sandbox/plugins/`

**Sandbox:**
- Purpose: Manage isolated working copy of project
- Examples: `src/app/sandbox.rs`
- Pattern: Track staged changes, detect conflicts, coordinate promotion back to project root

**LspClient:**
- Purpose: Communicate with language server process (e.g., rust-analyzer)
- Examples: `src/app/lsp/mod.rs`
- Pattern: Async message dispatch, state machines for hover/completion/references

## Entry Points

**main.rs:**
- Location: `src/main.rs`
- Triggers: Application startup
- Responsibilities:
  - Parse CLI arguments (project path, `--new-instance`)
  - Singleton check via IPC
  - Setup eframe native options
  - Initialize `EditorApp`

**EditorApp::new():**
- Location: `src/app/mod.rs:88`
- Triggers: eframe creation context
- Responsibilities:
  - Load persistent state (window geometry, panel visibility)
  - Start IPC server for singleton coordination
  - Load settings and i18n
  - Initialize plugin registry
  - Restore or open initial projects
  - Setup fonts and image loaders

**render_workspace():**
- Location: `src/app/ui/workspace/mod.rs:67`
- Triggers: Each UI frame (via EditorApp::update)
- Responsibilities:
  - Lazy-initialize terminals (Claude panel, build terminal)
  - Process background events (watcher, git, LSP)
  - Render menu bar, panels, editor, file tree, dialogs
  - Handle keyboard shortcuts and menu actions
  - Throttle repaints based on focus and activity
  - Return action to open project in new window

**init_workspace():**
- Location: `src/app/ui/workspace/state/init.rs`
- Triggers: Opening a new project
- Responsibilities:
  - Create `WorkspaceState` struct
  - Load project files into FileTree
  - Load user's build profiles and project config
  - Initialize LSP client (if available)
  - Start file watcher threads
  - Restore editor scroll/cursor positions from last session

## Error Handling

**Strategy:**
- **UI Errors:** Toast notifications (4s dismissal) propagate I/O issues to user
- **Internal Errors:** Logged to stderr; non-fatal (editor continues)
- **Plugin Errors:** Displayed in modal; user can disable plugin
- **LSP Errors:** Graceful fallback to basic syntax highlighting
- **Build Errors:** Parsed and listed; user can click to jump to source

**Patterns:**

1. **File I/O:**
   ```rust
   match std::fs::read_to_string(path) {
       Ok(content) => { /* process */ },
       Err(e) => {
           ws.toasts.push(Toast::error(format!("Failed to read: {}", e)));
           // Continue without the file
       }
   }
   ```

2. **Plugin Execution:**
   - Wrap in try-catch equivalent
   - Catch panics with `std::panic::catch_unwind`
   - Send error result back through `AppAction::PluginResponse`

3. **LSP Client:**
   - If startup fails, set `ws.lsp_binary_missing = true`
   - Show "Install" button in UI
   - Continue with fallback syntax highlighting

4. **Build Errors:**
   - Parser extracts file, line, column, message
   - If parse fails, display raw output to user

## Cross-Cutting Concerns

**Logging:**
- Strategy: eprintln! to stderr; structured logging via plugin system
- Key points: File operations, plugin lifecycle, IPC messages
- Location: Throughout codebase with explicit `eprintln!("msg: {:?}", data)`

**Validation:**
- Filenames: `src/app/validation.rs` checks for invalid characters
- Project names: `[a-zA-Z0-9_-]`, no leading `-`
- Paths: Canonicalize to prevent symlink attacks
- Build profile names: No reserved keywords

**Authentication:**
- AI Agents: Custom agents configured in settings with command + args
- Plugins: Blacklist mechanism in `Settings`; capability-based security (host bindings)
- IPC: Unix socket (Linux/Mac) or TCP (Windows); local-only
- No credentials stored in codebase (use env vars or host config)

**Concurrency:**
- `FileWatcher` spawned in background thread
- Build execution in thread pool
- Plugin calls wrapped in `Arc<Mutex>` for thread-safe access
- IPC server uses thread pool (max 16 workers)
- All channel-based; no raw mutex contention in hot path

**Internationalization:**
- Strings in `locales/{lang}/ui.ftl` (fluent format)
- `I18n` struct loaded at startup
- Fallback to English if key missing
- All UI strings routed through `tr!()` macro

---

*Architecture analysis: 2025-03-04*
