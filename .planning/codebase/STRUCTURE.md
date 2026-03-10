# Codebase Structure

**Analysis Date:** 2025-03-04

## Directory Layout

```
src/
├── main.rs                    # Application entry point, CLI parsing, IPC singleton check
├── config.rs                  # Constants: font sizes, panel widths, limits, repaint intervals
├── settings.rs                # User settings persistence (theme, fonts, custom agents, blacklist)
├── i18n.rs                    # Localization: language selection, string lookups
├── ipc.rs                     # Inter-process communication (singleton coordination, recent projects)
├── watcher.rs                 # File and project change detection via notify crate
├── highlighter.rs             # Syntax highlighting using syntect
│
├── app/
│   ├── mod.rs                 # EditorApp: main app struct, viewport management, frame logic
│   ├── types.rs               # Shared types: AppShared, AppAction, Toast, PersistentState, ProjectType
│   ├── startup.rs             # Startup dialog logic and project discovery
│   ├── validation.rs          # Name/path validation helpers
│   ├── build_runner.rs        # Cargo/build execution and error parsing
│   ├── project_config.rs      # BuildProfile and ProjectProfiles loading
│   ├── sandbox.rs             # Project ↔ sandbox sync, staged changes, conflict detection
│   ├── local_history.rs       # Undo snapshots per file
│   ├── fonts.rs               # Custom font setup for egui
│   │
│   ├── ai/
│   │   ├── mod.rs             # AiManager: system prompt generation, context payload assembly
│   │   ├── types.rs           # AiContextPayload, AiFileContext, AiExpertiseRole, AiReasoningDepth
│   │   └── tools.rs           # Standard tools list for AI agents
│   │
│   ├── lsp/
│   │   ├── mod.rs             # LspClient: spawn/manage language server, message dispatch
│   │   ├── types.rs           # LSP state: hover, completion, diagnostics, references
│   │   └── ...                # Protocol message handling
│   │
│   ├── registry/
│   │   ├── mod.rs             # AgentRegistry, CommandRegistry, PanelRegistry
│   │   └── plugins/
│   │       ├── mod.rs         # PluginManager: WASM plugin loading and execution
│   │       ├── types.rs       # Plugin manifest, tool definitions, capabilities
│   │       ├── security.rs    # Capability enforcement, sandboxing
│   │       └── host/
│   │           ├── mod.rs     # Host bindings: file_system, search, system info
│   │           ├── fs.rs      # read_file, write_file, list_dir bindings
│   │           ├── search.rs  # Full-text project search binding
│   │           └── sys.rs     # System queries (git, clipboard, env)
│   │
│   ├── project_templates/
│   │   └── ...                # Rust, PHP (Symfony/Laravel/Nette), Python, Node templates
│   │
│   └── ui/
│       ├── mod.rs             # UI module setup, re-exports
│       ├── panels.rs          # Panel rendering functions (left, right)
│       ├── search_picker.rs   # File picker (Ctrl+P), project search dialog (Ctrl+Shift+F)
│       ├── background.rs      # Async event processing (watcher, git, LSP, plugins)
│       ├── ai_panel.rs        # AI chat panel rendering and state
│       │
│       ├── editor/
│       │   ├── mod.rs         # Editor: tabs, rendering orchestration
│       │   ├── ui.rs          # Main editor UI loop
│       │   ├── tabs.rs        # Tab management (open, close, switch, autosave)
│       │   ├── files.rs       # File loading, binary detection, image textures
│       │   ├── search.rs      # Find/replace logic (Ctrl+F, Ctrl+H)
│       │   ├── markdown.rs    # Markdown preview logic and caching
│       │   ├── diff_view.rs   # Side-by-side diff viewer
│       │   ├── render_helpers.rs
│       │   ├── render_binary.rs
│       │   ├── render_context.rs
│       │   ├── render_lsp.rs  # LSP integration rendering (hover, completion, diagnostics)
│       │   ├── render_markdown.rs
│       │   ├── render_normal.rs # Standard text editor rendering
│       │   └── render/
│       │       ├── mod.rs
│       │       ├── normal.rs
│       │       ├── binary.rs
│       │       ├── context.rs
│       │       ├── markdown.rs
│       │       ├── tabs.rs
│       │       ├── helpers.rs
│       │       └── lsp/
│       │           ├── mod.rs
│       │           ├── typing.rs
│       │           ├── completion.rs
│       │           ├── hover.rs
│       │           └── navigation.rs
│       │
│       ├── file_tree/
│       │   ├── mod.rs         # FileTree: tree state and UI orchestration
│       │   ├── node.rs        # FileNode: recursive tree structure with lazy loading
│       │   ├── render.rs      # Tree rendering, icons, git colors
│       │   ├── ops.rs         # Tree operations (create, rename, delete, expand/collapse)
│       │   └── dialogs.rs     # Context menu dialogs (new file/dir, rename, delete confirm)
│       │
│       ├── terminal/
│       │   ├── mod.rs         # Terminal: tabs, initialization
│       │   ├── window.rs      # Terminal window layout and focus
│       │   ├── right/
│       │   │   ├── mod.rs     # Right panel (Claude chat) rendering
│       │   │   └── ai_bar.rs  # AI toolbar: provider selector, settings
│       │   ├── bottom/
│       │   │   ├── mod.rs     # Bottom panel (build + git) layout
│       │   │   ├── build_bar.rs    # Build toolbar: buttons, error list
│       │   │   ├── compile_bar.rs  # Compile output display
│       │   │   └── git_bar.rs      # Git branch display
│       │   ├── instance/
│       │   │   ├── mod.rs     # Terminal state (PTY, scrollback)
│       │   │   ├── backend.rs # PTY backend interface
│       │   │   ├── input.rs   # Input handling (keyboard, paste)
│       │   │   └── render.rs  # Terminal rendering via egui_term
│       │   └── ai_chat/
│       │       ├── mod.rs     # AI chat conversation state
│       │       ├── logic.rs   # Message parsing, agent invocation
│       │       ├── render.rs  # Chat UI rendering
│       │       ├── approval.rs # Dangerous action approval dialog
│       │       └── inspector.rs # Token usage and payload inspection
│       │
│       ├── dialogs/
│       │   ├── mod.rs         # Dialog state machine and orchestration
│       │   ├── dependency_wizard.rs # Install missing tools (rust-analyzer, makepkg, bsdtar)
│       │   ├── about.rs       # About/version modal
│       │   ├── ai.rs          # AI provider/agent selection modal
│       │   ├── ai_dialogs.rs  # AI-specific dialogs (system prompt, token usage, etc.)
│       │   ├── settings.rs    # Settings modal (theme, fonts, custom agents)
│       │   ├── plugins.rs     # Plugins modal (enable/disable, view logs)
│       │   └── terminal.rs    # Terminal management dialog
│       │
│       ├── workspace/
│       │   ├── mod.rs         # render_workspace: main workspace rendering orchestrator
│       │   ├── index.rs       # ProjectIndex: fast file lookup by name or path
│       │   ├── semantic_index.rs # SemanticIndex: BERT-based semantic search
│       │   ├── modal_dialogs.rs  # Modal dialog rendering
│       │   ├── modal_dialogs/
│       │   │   ├── mod.rs
│       │   │   ├── about.rs
│       │   │   ├── ai.rs
│       │   │   ├── ai_dialogs.rs
│       │   │   ├── conflict.rs  # External change conflict resolution
│       │   │   ├── plugins.rs
│       │   │   ├── settings.rs
│       │   │   └── terminal.rs
│       │   ├── menubar/
│       │   │   ├── mod.rs     # Menu bar rendering and action dispatch
│       │   │   ├── file.rs    # File menu: open, save, close, quit
│       │   │   ├── project.rs # Project menu: open, new, recent, distributions
│       │   │   ├── edit.rs    # Edit menu: undo, redo, cut, copy, paste
│       │   │   ├── view.rs    # View menu: panel toggles, zoom
│       │   │   ├── build.rs   # Build menu: build, run, test, clean
│       │   │   └── help.rs    # Help menu: about, documentation
│       │   └── state/
│       │       ├── mod.rs     # WorkspaceState: central project state
│       │       ├── init.rs    # Workspace initialization
│       │       ├── types.rs   # Supporting types (FilePicker, ProjectSearch, etc.)
│       │       └── actions.rs # Actions: open file, jump to line, tool checks
│       │
│       └── widgets/
│           └── command_palette/
│               └── ...         # Command palette (Ctrl+Shift+P) and command execution
│
└── plugins/
    ├── hello/
    │   └── src/lib.rs         # Template plugin (WASM)
    ├── ollama/
    │   └── src/lib.rs         # Ollama API plugin (WASM)
    └── gemini/
        └── src/lib.rs         # Google Gemini plugin (WASM)
```

## Directory Purposes

**src/app:**
- Purpose: Application logic, state management, frame updates
- Key files: `mod.rs` (EditorApp), `types.rs` (shared types), workspace/ui submodules

**src/app/ui:**
- Purpose: All user interface components and rendering
- Key files: Editor, FileTree, Terminal, workspace state, modal dialogs

**src/app/ui/editor:**
- Purpose: Multi-tab text editor with language features
- Key files: `mod.rs` (orchestration), `tabs.rs` (tab state), `render/` (rendering engines)

**src/app/ui/file_tree:**
- Purpose: Directory navigation and file operations
- Key files: `mod.rs` (tree state), `node.rs` (tree structure), `render.rs` (rendering)

**src/app/ui/terminal:**
- Purpose: PTY-based shell and AI chat interfaces
- Key files: `instance/` (PTY state), `ai_chat/` (conversation), `bottom/`, `right/` (layout)

**src/app/ui/workspace:**
- Purpose: Orchestrate all panels for a single project workspace
- Key files: `mod.rs` (render_workspace), `state/` (WorkspaceState), `menubar/`, `modal_dialogs/`

**src/app/registry:**
- Purpose: Plugin system and extensibility
- Key files: `mod.rs` (registries), `plugins/` (WASM loader and execution)

**src/app/lsp:**
- Purpose: Language server integration for code intelligence
- Key files: `mod.rs` (LspClient), message dispatch and state management

**src/app/ai:**
- Purpose: AI agent management and context generation
- Key files: `mod.rs` (AiManager), `types.rs` (context structures), `tools.rs` (tool defs)

## Key File Locations

**Entry Points:**
- `src/main.rs`: Application startup, CLI argument parsing
- `src/app/mod.rs`: EditorApp creation and frame update loop
- `src/app/ui/workspace/mod.rs:67`: Main workspace rendering function

**Configuration:**
- `src/config.rs`: All constants (fonts, dimensions, intervals)
- `src/settings.rs`: User settings persistence
- `src/i18n.rs`: Localization strings and lookups

**Core Logic:**
- `src/app/ui/editor/mod.rs`: Editor state and autosave
- `src/app/ui/file_tree/mod.rs`: File tree operations
- `src/app/ui/workspace/state/mod.rs`: WorkspaceState definition
- `src/app/sandbox.rs`: Project ↔ sandbox synchronization
- `src/app/build_runner.rs`: Build execution and error parsing

**Testing & Validation:**
- `src/app/validation.rs`: Name/path validation
- Test files: Located alongside implementations with `_test.rs` suffix

## Naming Conventions

**Files:**
- Module files: `mod.rs` for directory modules
- Logical units: Descriptive names matching primary type/function (e.g., `editor.rs` for `Editor`, `build_runner.rs` for build logic)
- Rendering functions: `render.rs` or `render_*.rs` for UI components
- Tests: Inline in files as `#[cfg(test)]` modules, not separate test files

**Directories:**
- UI components: `src/app/ui/{component_name}/` (e.g., `editor/`, `file_tree/`, `terminal/`)
- Workspace subsystems: `src/app/ui/workspace/{system}/` (e.g., `menubar/`, `modal_dialogs/`, `state/`)
- Feature areas: `src/app/{feature}/` (e.g., `ai/`, `lsp/`, `registry/`)

**Types and Structs:**
- Main types: PascalCase (e.g., `EditorApp`, `WorkspaceState`, `FileTree`)
- Enums: PascalCase variants (e.g., `FocusedPanel`, `ProjectType`)
- State structs: Usually suffixed with "State" (e.g., `WorkspaceState`, `WizardState`)

**Functions:**
- Public API: snake_case (e.g., `render_workspace`, `init_workspace`, `open_file_in_ws`)
- Rendering functions: `render_*` prefix (e.g., `render_menu_bar`, `render_left_panel`)
- Action handlers: `on_*` or `handle_*` prefix (e.g., `on_file_saved`, `handle_error`)

**Constants:**
- SCREAMING_SNAKE_CASE (e.g., `MAX_RECENT_PROJECTS`, `EDITOR_FONT_SIZE`, `REPAINT_INTERVAL_MS`)
- Located in `src/config.rs` for UI constants

**Modules:**
- Private modules: No prefix (e.g., `startup`, `validation`)
- Public modules: Re-exported from parent `mod.rs` with `pub use`

## Where to Add New Code

**New Feature (e.g., new panel or major functionality):**
- Primary code: `src/app/ui/{feature_name}/mod.rs` (and submodules)
- State integration: Add field to `WorkspaceState` in `src/app/ui/workspace/state/mod.rs`
- Workspace rendering: Add call to render function in `src/app/ui/workspace/mod.rs:render_workspace`
- Tests: Inline `#[cfg(test)]` module in the main file

**New UI Component (e.g., custom widget):**
- Implementation: `src/app/ui/widgets/{component_name}/mod.rs`
- Integration: Reference from parent UI component (e.g., editor or file tree)
- Pattern: Return `ComponentResult` struct with click/action information

**New Command or Menu Item:**
- Command definition: `src/app/registry/mod.rs` (CommandRegistry)
- Handler: `src/app/ui/widgets/command_palette.rs` (execute_command function)
- Menu binding: `src/app/ui/workspace/menubar/{menu_file}.rs`
- Keyboard shortcut: Add to command definition with `Some("Ctrl+X")` shortcut field

**New Settings Option:**
- Type definition: `src/settings.rs` (Settings struct field)
- UI dialog: `src/app/ui/workspace/modal_dialogs/settings.rs`
- Persistence: Automatically serialized via serde
- Localization: Add keys to `locales/{lang}/ui.ftl`

**New AI Plugin or Agent:**
- WASM plugin: Create `src/plugins/{plugin_name}/` with Cargo.toml and src/lib.rs
- CLI agent: Register in `EditorApp::new()` from settings or detected from PATH
- Host bindings: Add to `src/app/registry/plugins/host/{system}.rs` if needed
- Context awareness: Update `AiManager::generate_context()` to include relevant project state

**Utilities and Helpers:**
- Shared helpers: `src/app/utils.rs` (create if doesn't exist)
- Format-specific parsing: Create module in relevant layer (e.g., `src/app/highlighter.rs` for syntax)
- Validation logic: `src/app/validation.rs`

## Special Directories

**src/plugins/:**
- Purpose: WASM plugin implementations
- Generated: No (source code)
- Committed: Yes
- Note: Each plugin has its own Cargo.toml; compiled separately to `.wasm`

**.polycredo/sandbox/:**
- Purpose: Isolated working copy of the project
- Generated: Yes (sync'd from project root)
- Committed: No (.gitignore)
- Contents: Copy of all source files, excluding node_modules, target/, .git/

**locales/:**
- Purpose: Localization strings in fluent format
- Generated: No (manually maintained)
- Committed: Yes
- Structure: `locales/{lang}/ui.ftl` (one file per language: cs, en, de, sk, ru)

**~/.config/polycredo-editor/:**
- Purpose: User configuration and runtime data (outside sandbox)
- Generated: Yes
- Contents: `settings.json`, `recent.json`, `session.json`, `plugins/`, `polycredo-editor.sock`

---

*Structure analysis: 2025-03-04*
