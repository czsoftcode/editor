# External Integrations

**Analysis Date:** 2025-03-04

## APIs & External Services

**AI/LLM Providers:**
- **Ollama** - Local LLM inference via HTTP API
  - SDK/Client: Custom HTTP client via `extism-pdk` (WASM plugin)
  - Plugin: `src/plugins/ollama/` (Cargo.toml, Cargo.lock, ollama.toml, src/lib.rs)
  - Default endpoint: `http://localhost:11434`
  - Config variable: `API_URL` (env var in plugin config, defaults to localhost:11434)
  - Auth: Optional Bearer token via `API_KEY` environment variable
  - Endpoint: `POST {API_URL}/api/chat` for completion requests

- **Google Gemini** - Cloud LLM API
  - SDK/Client: Custom HTTP client via `extism-pdk` (WASM plugin)
  - Plugin: `src/plugins/gemini/` (Cargo.toml, Cargo.lock, gemini.toml, src/lib.rs)
  - Auth: Required `API_KEY` environment variable (x-goog-api-key header)
  - Endpoint: `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`
  - Integration type: Google Cloud Generative Language API

- **Claude Code CLI** - CLI-based AI agent integration
  - Configured as custom agent in `src/settings.rs`
  - Command: `claude` (detected via PATH check)
  - Not a direct API integration; runs as subprocess for AI Chat panels
  - Tool detection in `src/app/ai/tools.rs`

- **Aider** - Code-generation CLI tool
  - Configured as custom agent in `src/settings.rs`
  - Command: `aider` (detected via PATH check)
  - Runs as subprocess for AI-assisted editing
  - Tool detection in `src/app/ai/tools.rs`

- **Gemini CLI** - Command-line Gemini access tool
  - Configured as custom agent in `src/settings.rs`
  - Command: `gemini` (detected via PATH check)
  - Custom CLI agents allow user-defined commands and args

## Plugin System (Host Allowlist)

**WASM Plugin Communication:**
- Plugin manifest format: `.toml` (e.g., `ollama.toml`, `gemini.toml`)
- Allowed hosts whitelist: Plugins declare `allowed_hosts = ["host1", "host2"]` for network access
- Ollama plugin allowed hosts: `["localhost", "127.0.0.1"]`
- Gemini plugin allowed hosts: Requires explicit allowlist (typically `["generativelanguage.googleapis.com"]`)
- Host-to-plugin interface: 20 host functions via Extism (read_file, write_file, exec_in_sandbox, etc.)
- Dynamic host allowlist: If `API_URL` config is provided, its host is automatically added to manifest

**Plugin Directories:**
- System plugins: Distributed with editor (Ollama, Gemini, Hello)
- User plugins: `~/.config/polycredo-editor/plugins/` (loaded at startup)
- Plugin discovery: Files with `.wasm` extension + corresponding `.toml` manifest
- Plugin priority: Sandbox > Project > Global (first match wins)

## Data Storage

**Databases:**
- Not applicable - No traditional database (SQLite, PostgreSQL, etc.)

**File Storage:**
- **Local filesystem only** - All projects stored on local disk
- Project root: User-selected directory (e.g., `~/MyProject/Rust/project-name`)
- Sandbox copy: `.polycredo/sandbox/` (working copy for editor)

**Configuration Storage:**
- Settings: `~/.config/polycredo-editor/settings.toml` (user preferences, plugin config)
- Recent projects: `~/.config/polycredo-editor/recent.json` (list of file paths)
- Session state: `~/.config/polycredo-editor/session.json` (open windows for restoration)
- Plugin cache: `~/.config/polycredo-editor/plugins/` (WASM binaries + manifests)

**Session Management:**
- Per-window state: `Arc<Mutex<WorkspaceState>>` in memory
- Session persistence: JSON array of project paths saved/restored via IPC
- Format: `["path1", "path2", ...]` (absolute canonical paths only)

**Caching:**
- Build output: `~/.cache/polycredo-editor/target/` (cargo build artifacts)
- No other explicit caching layer

## Authentication & Identity

**Auth Provider:**
- Custom / Per-plugin configuration
- CLI tools (claude, aider, gemini): Assumed authenticated via their own config (~/.config/*, env vars)
- Plugin auth: Passed via environment variables in plugin config map
- IPC multi-instance: No authentication (same user, local sockets/TCP 127.0.0.1 only)

**Credential Handling:**
- Credentials NOT stored by editor - delegated to plugins and external tools
- Ollama: API_KEY optional (Bearer token, passed to plugin at plugin startup)
- Gemini: API_KEY required (read from environment, passed to plugin)
- Blacklist for sensitive files: Default blacklist in `src/settings.rs`:
  - `.env*`, `*.key`, `id_rsa*`, `Cargo.lock`
  - User-customizable via `blacklist: Vec<String>` in settings
  - Glob patterns matched against all files accessed by plugins

## Monitoring & Observability

**Error Tracking:**
- Not detected - No error tracking service integration (Sentry, Rollbar, etc.)
- Local logging: stderr and app::Toast system for user-facing errors

**Logs:**
- Standard output: stderr for debug messages (eprintln!)
- UI toasts: Short-lived error notifications in bottom-right corner (auto-dismiss 4s)
- File watcher events: Logged to stderr on file changes/deletions
- IPC errors: Logged to stderr when connection fails
- Plugin execution: Errors logged via `host_log_monologue` function (WASM → host)

## CI/CD & Deployment

**Hosting:**
- GitHub repository (`https://github.com/czsoftcode/editor`)
- Packager: `cargo-packager` for cross-platform builds

**CI Pipeline:**
- Not detected in codebase - No GitHub Actions, GitLab CI, etc. configured
- Manual builds only (via `cargo build --release`)

**Distribution Formats:**
- DEB (Debian/Ubuntu)
- RPM (Fedora/RHEL)
- AppImage (universal Linux)
- NSIS (Windows installer)
- AUR (Arch Linux, via cargo-aur)
- Tar.gz (BSD, manual installs)
- macOS support (via eframe)

## Environment Configuration

**Required env vars:**
None strictly required - all are optional or defaulted.

**Optional for CLI tools:**
- For system tools: Standard system env (PATH, HOME, etc.)
- For Claude Code: Typically requires Claude API key (not handled by editor)
- For Aider: Requires API keys for its configured LLM (not handled by editor)
- For Gemini: `GEMINI_API_KEY` or plugin-configured key

**Secrets location:**
- Plugin configuration: Passed at plugin activation time (not persisted by editor)
- Ollama: API_KEY injected into plugin manifest at runtime
- Gemini: API_KEY injected into plugin manifest at runtime
- User credentials: Stored in plugin config maps (HashMap<String, String>)
- Sensitive files in project: Protected by blacklist (glob patterns)

**Example plugin config (settings.toml):**
```toml
[plugins.ollama]
enabled = true
config.API_URL = "http://localhost:11434"
config.API_KEY = "optional-bearer-token"

[plugins.gemini]
enabled = true
config.API_KEY = "goog-xxx..."
```

## Webhooks & Callbacks

**Incoming:**
- Not applicable - Editor is a client application, not a server

**Outgoing:**
- Not applicable - No outgoing webhooks to external services
- File watcher triggers internal events only (reload UI, re-parse syntax, etc.)
- IPC commands: Internal multi-instance communication only (not external)

## File System Integration

**File Watching:**
- `notify` crate monitors project directory for changes
- Events: file created, modified, deleted
- Triggers: Auto-reload files from disk, rebuild file tree, update git status

**File Dialog:**
- `rfd` crate for native file dialogs
- Used for: Select project folder, select files to open
- Platform: Uses native OS file picker (GTK on Linux, Cocoa on macOS, Windows API on Windows)

## Git Integration

**Git Status Monitoring:**
- Subprocess-based: Runs `git status --porcelain` periodically (5s interval)
- Provides: Branch name, file status (M/A/D/??)
- Display: Color-coded file tree, branch indicator in status bar
- No Git mutations from editor (read-only integration)

## System Integration

**Clipboard:**
- `arboard` 3.6.1 with wayland-data-control feature
- Copy/paste support in editor and terminal

**PATH Extension:**
- Auto-extends PATH to include `~/.cargo/bin` and `~/.local/bin`
- Allows detection and execution of user-installed CLI tools

**Process Spawning:**
- `tokio::process::Command` for async subprocesses
- Used for: cargo build/run/test, git status, custom CLI agents
- Terminal emulation: `alacritty_terminal` for PTY management

---

*Integration audit: 2025-03-04*
