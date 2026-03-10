# Technology Stack

**Analysis Date:** 2025-03-04

## Languages

**Primary:**
- Rust 2024 Edition - Core application, all backend logic, UI framework integration
- WASM (WebAssembly) - Plugin system via Extism, includes Ollama and Gemini plugins

**Secondary:**
- TOML - Configuration format (settings.toml, plugin manifests)
- JSON - Data serialization (recent.json, session.json, serde_json for IPC)
- Markdown - Documentation and markdown preview support via `pulldown-cmark`
- Fluent (.ftl) - Internationalization strings in `locales/{lang}/ui.ftl` (5 languages: cs, en, de, sk, ru)

## Runtime

**Environment:**
- Linux (primary), Windows, macOS support
- Target directory cached to `~/.cache/polycredo-editor/target`

**Package Manager:**
- Cargo (Rust package manager)
- Lockfile: `Cargo.lock` present
- Edition: 2024 (latest Rust edition)
- Build parallelism: Dynamically calculated at build time from available cores (see `build.rs`)

## Frameworks

**Core GUI:**
- `eframe` 0.31 - Desktop application framework with persistence and multi-viewport support
  - Features: persistence, default_fonts, glow, wayland, x11 backend support
  - `egui_extras` 0.31 - Additional egui widgets (image, svg rendering)

**Editor & Terminal:**
- `egui` - Immediate-mode GUI library (transitive via eframe)
- `egui_term` 0.1 - Terminal emulator widget (vendored at `vendor/egui_term`)
- `alacritty_terminal` 0.25 - PTY/terminal backend

**Async & Concurrency:**
- `tokio` 1.x - Async runtime with multi-threaded executor, process spawning, I/O utilities
- `tokio-util` 0.7 - Tokio utilities for codec/compat
- `async-lsp` 0.2.2 - Language Server Protocol client
- `tower` 0.4 - Service abstraction framework

**Serialization & Config:**
- `serde` 1.x with derive macros - Data serialization
- `serde_json` 1 - JSON parsing and generation
- `toml` 0.8 - TOML configuration parsing
- `bincode` 1.3 - Binary serialization (for session/state)

**Text & Syntax:**
- `syntect` 5 - Syntax highlighting with language support
- `regex` 1 - Pattern matching
- `pulldown-cmark` 0.12 - Markdown parsing and rendering
- `egui_commonmark` 0.20 - Markdown widget for egui with syntax highlighting
- `similar` 2.7.0 - Diff algorithm for code changes

**Plugin System:**
- `extism` 1.5 - WASM plugin host/runtime
- `url` 2.5.8 - URL parsing for plugin host allowlists

**Hashing & Collections:**
- `xxhash-rust` 0.8.15 with xxh3 - Fast hashing for config and scratch data
- `smallvec` 1 - Small vector optimization
- Standard collections (HashMap, BTreeMap)

**I/O & File Operations:**
- `notify` 7 - File system watcher for reload detection
- `walkdir` 2 - Recursive directory traversal
- `rfd` 0.15 - Native file dialogs (cross-platform)
- `dirs` 6 - System paths (home, config directories)
- `libc` 0.2 - Unix system calls

**System & Localization:**
- `fluent-bundle` 0.15 - i18n message bundles
- `unic-langid` 0.9 - Language ID parsing (BCP 47)
- `arboard` 3.6.1 with wayland-data-control - Clipboard support (Linux, Windows, macOS)

**Images & Rendering:**
- `image` 0.25 with png, jpeg, gif, webp - Image format support
- `glow` - OpenGL wrapper (transitive via eframe)

**AI & ML (Embedded Models):**
- `candle-core` 0.9 - Tensor operations
- `candle-nn` 0.9 - Neural network layers
- `candle-transformers` 0.9 - Transformer models
- `tokenizers` 0.21 - Tokenization for LLMs
- `hf-hub` 0.4 - Hugging Face model hub integration
- GGUF model: `all-minilm-l6-v2-q4_k_m.gguf` (21MB, embedded in repo)

**Error Handling:**
- `anyhow` 1 - Flexible error handling with context

## Configuration

**Environment:**
- Runtime env set via `main.rs`: `TERM=xterm-256color`, `COLORTERM=truecolor`
- Build env config in `.cargo/config.toml`:
  - `jobs`: Parallelism = `cores * 2 / 3`
  - `RAYON_NUM_THREADS = cores / 2` (for parallel iterators)
  - `TOKIO_WORKER_THREADS = cores / 2`
  - Target directory: `~/.cache/polycredo-editor/target`

**Key configs required:**
- `~/.config/polycredo-editor/settings.toml` - Persistent user settings (theme, font sizes, plugin config)
- `~/.config/polycredo-editor/recent.json` - Recent projects list (IPC-managed)
- `~/.config/polycredo-editor/session.json` - Open windows state (multi-instance restoration)
- `~/.config/polycredo-editor/plugins_dir/` - User plugin directory for `.wasm` plugins

**Build:**
- `build.rs` - Handles version auto-increment, platform-specific resources
  - Windows: `.rc` resource script → `llvm-rc` or `winres` fallback
  - Icon handling for all platforms

## Platform Requirements

**Development:**
- Rust 1.74+ (Edition 2024)
- `llvm-rc` or `winres` (Windows resource compilation)
- System libraries:
  - X11 + XCB (Linux X11)
  - Wayland client (Linux Wayland)
  - EGL and OpenGL (rendering)

**Production (Runtime Dependencies - via Debian metadata):**
```
libc6, libgcc-s1, libx11-6, libxcb1, libxkbcommon0,
libwayland-client0, libwayland-cursor0, libwayland-egl1,
libegl1, libgl1, libssl3 | libssl1.1, ripgrep, debconf
```

**Packaging Support:**
- DEB (Debian/Ubuntu) via `cargo-deb`
- RPM (Fedora/RHEL) via `cargo-generate-rpm`
- AppImage support
- Windows NSIS installer
- Tar.gz archive (BSD/manual installs)

## Profile Configuration

**Debug:**
- `debug = 1` - Line table debug info only (faster compile)
- `incremental = true` - Incremental compilation enabled

**Release:**
- `lto = "thin"` - Thin Link-Time Optimization
- `strip = true` - Strip debug symbols automatically
- `opt-level = 3` - Maximum optimization

---

*Stack analysis: 2025-03-04*
