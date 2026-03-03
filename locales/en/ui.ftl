# General user interface elements

## Panels
panel-files = Files
panel-files-sandbox = Files (Sandbox)
btn-tree-project = Project
btn-tree-sandbox = Sandbox
panel-runners = Runners
panel-build = Build
panel-git = Git
panel-build-errors =
    { $count ->
        [one] Error (1)
       *[other] Errors ({ $count })
    }

## Build buttons
btn-build = ▶ Build
btn-build-sandbox-on = Sandbox ON
btn-build-sandbox-off = Sandbox OFF
hover-build-sandbox = Toggle between running in project root and AI sandbox
btn-run = ▶ Run
btn-run-new = ▶ Run+
btn-test = ▶ Test
btn-clean = ✖ Clean
btn-create-deb = Create .deb
hover-create-deb = Build and create a development .deb package with build number
hover-create-deb-disabled = Cannot create package in sandbox mode. Switch to Sandbox OFF.
hover-build-menu-disabled = Building is disabled in Sandbox ON mode or if there are unpromoted files in the sandbox.
btn-run-profile = ▶ Run Profile...
btn-git-profile =  Git...
btn-edit-profiles = ⚙ Edit
runner-none = No profiles defined.
menu-build-windows = Windows

## Git operations
git-add-all = git add .
git-commit = git commit -m "..."
git-push = git push
git-status = git status
git-diff = git diff
git-checkout-file = git checkout (file)
git-checkout-branch = git checkout (branch)
git-pull = git pull
git-reset-hard = git reset --hard
hover-git-disabled-sandbox = Git operations are disabled until all sandbox changes are resolved (use 'Review Changes' or 'Promote All' in the yellow bar).

## Status bar
statusbar-line-col = Line { $line }, Column { $col }
statusbar-encoding = UTF-8
statusbar-unsaved = Unsaved
statusbar-saving = Saving…
statusbar-saved = Saved
statusbar-lsp-initializing = LSP initializing...
statusbar-filetype-plain = Plain Text

## Editor tabs
tab-unsaved-indicator = ●
tab-deleted-indicator = ⚠

## Find and replace
search-label = Find:
replace-label = Replace:
search-replace-expand = Replace…
search-placeholder = Search…
replace-placeholder = Replace…
search-prev = ▲
search-next = ▼
search-replace-one = Replace
search-replace-all = Replace All
search-results =
    { $count ->
        [one] 1 result
       *[other] { $count } results
    }
search-no-results = No matches found

## Editor
editor-empty-hint = Open a file from the file tree on the left
editor-preview-label = Preview

# LSP / rust-analyzer
lsp-missing-title = Missing rust-analyzer
lsp-missing-msg = Smart features (autocomplete, diagnostics) require rust-analyzer. Would you like to install it?
lsp-install-btn = Install
lsp-installing = Installing rust-analyzer...
lsp-install-success = rust-analyzer installed successfully. Restarting LSP...
lsp-install-error = Installation failed: { $error }

## Find References (Shift+F12)
lsp-references-heading = References
lsp-references-searching = Searching for references...
lsp-references-none = No references found.
lsp-references-found =
    { $count ->
        [one] 1 reference found.
       *[other] { $count } references found.
    }
lsp-references-error = LSP: Error searching for references.

## Terminal
terminal-unavailable = Terminal is not available.
terminal-retry = Try again
terminal-exited = [Process exited — press R to restart]
terminal-close-confirm-title = Close terminal?
terminal-close-confirm-msg = A process is still running in this terminal. Do you really want to terminate it?

## Go to Line dialog (Ctrl+G)
goto-line-prompt = Go to line:
goto-line-placeholder = line number

## Command Palette (Ctrl+Shift+P)
command-palette-heading = Commands
command-palette-placeholder = Search command…
command-palette-no-results = No results

command-name-open-file = Open File
command-name-project-search = Search in Project
command-name-build = Build
command-name-run = Run
command-name-save = Save Current File
command-name-close-tab = Close Current Tab
command-name-new-project = New Project
command-name-open-project = Open Project (in new window)
command-name-open-folder = Open Folder (in this window)
command-name-toggle-left = Toggle File Panel
command-name-toggle-right = Toggle AI Panel
command-name-toggle-build = Toggle Build Terminal
command-name-toggle-float = Toggle Floating AI Panel
command-name-show-about = About
command-name-show-settings = Settings
command-name-quit = Quit PolyCredo Editor
command-name-plugin-hello = Plugin: Say Hello
command-name-plugin-gemini = Plugin: Ask Gemini

## Quick file open (Ctrl+P)
file-picker-heading = Open File
file-picker-placeholder = Quick Open File…
file-picker-no-results = No results
file-picker-count = { $count } files
file-picker-count-filtered = { $filtered }/{ $total } files
file-picker-more = … and { $count } more

## Project-wide search (Ctrl+Shift+F)
project-search-heading = Search in Project
project-search-placeholder = Search in project…
project-search-hint = Search term…
project-search-btn = Search
project-search-loading = Searching…
project-search-result-label = Results for "{ $query }" ({ $count })
project-search-results =
    { $count ->
        [one] 1 result
       *[other] { $count } results
    }
project-search-no-results = No results
project-search-max-results = Showing max. { $max } results

## Common buttons
btn-ok = OK
btn-confirm = Confirm
btn-cancel = Cancel
btn-close = Close
btn-browse = Browse…
btn-create = Create
btn-open = Open
btn-refresh = Refresh
btn-save = Save
btn-rename = Rename
btn-copy = Copy
btn-paste = Paste
btn-delete = Delete
btn-name-label = Name:

## AI panel
ai-panel-title = AI Terminal
ai-tool-not-found = Tool { $tool } not found in PATH.
ai-tool-detecting = Detecting AI tools…
ai-label-assistant = Assistant:
ai-tool-status-checking = { $tool } (checking…)
ai-tool-status-available = { $tool } (installed)
ai-tool-status-missing = { $tool } (not in PATH)
ai-hover-reverify = Re-verify AI CLI tools availability
ai-hover-checking = Checking AI CLI tools availability…
ai-hover-start = Starts { $tool } (`{ $cmd }`) in terminal
ai-hover-missing = Command `{ $cmd }` not found in PATH. Install the tool and click ↻.
ai-btn-start = ▶ Start
ai-btn-sync = ⟳ Sync
ai-hover-sync = Send context (open files, build errors) to the AI agent
ai-diff-heading = Review AI Proposed Changes
ai-diff-new-file = New file proposed
ai-plugin-bar-label = AI:
ai-plugin-bar-settings = ⚙
ai-plugin-bar-start-hover = Launch AI chat with selected plugin
ai-plugin-bar-settings-hover = Open settings for the selected AI plugin
ai-float-dock = Dock to panel
ai-float-undock = Undock to floating window
ai-viewport-open = Open in separate window
ai-tab-close-hover = Close tab
ai-tab-new-hover = New terminal tab
ai-staged-bar-msg = AI suggested changes to the project
ai-staged-bar-review = Review Changes
ai-staged-bar-promote-all = Promote All
ai-staged-modal-hint = Click a file to review differences and accept changes:
ai-staged-files = Suggested Changes (Sandbox)
ai-staged-new = [NEW]
ai-staged-mod = [MOD]
ai-staged-del = [DELETED]
ai-promotion-success-title = Changes Applied
ai-promotion-success-body = The following file has been successfully updated in your project:
ai-promotion-success = Changes successfully applied to the project.
ai-promotion-all-success = Successfully promoted { $count } files to project.
ai-promotion-failed = Failed to apply changes: { $error }

## Sync before starting AI
ai-sync-title = Sync before start
ai-sync-msg = Differences detected between project and sandbox. Latest versions should be synchronized.
ai-sync-to-sandbox = Update Sandbox ({ $count } newer in project)
ai-sync-to-project = Promote to Project ({ $count } newer in sandbox)
ai-sync-btn-sync = Sync and Start
ai-sync-btn-skip = Start without sync

## Plugin Permissions
plugin-auth-bar-msg = Plugin "{ $name }" requests internet access ({ $hosts }).
plugin-auth-bar-allow = Allow and Start
plugin-auth-bar-deny = Deny

## Settings
settings-title = Settings
settings-category-general = General
settings-category-editor = Editor
settings-category-ai = AI Agents
settings-language = Language
settings-language-restart = Language changes take effect immediately.
settings-theme = Theme
settings-theme-dark = Dark
settings-theme-light = Light
settings-auto-show-diff = Automatically open AI change preview
settings-safe-mode = Safe Mode (Project Read-Only)
settings-safe-mode-hint = When enabled, file tree and build default to Sandbox, and direct project saving is blocked.
settings-diff-mode = AI Diff Layout
settings-diff-inline = Inline (+ / -)
settings-diff-side-by-side = Side-by-side
settings-editor-font = Editor — font size
settings-ai-font = AI Terminal — font size
settings-default-path = Default project path
settings-creates-in = Will be created at:
settings-ai-name = Assistant Name
settings-ai-command = Command (binary)
settings-ai-args = Arguments (optional)
settings-ai-add = Add agent
settings-ai-hint = Here you can define your own CLI tools (e.g., gemini, claude, aider). If the list is empty, defaults will be used.

## Plugins
plugins-title = Plugin Manager
plugins-list-label = Plugins List
plugins-no-selection = Select a plugin from the list on the left
plugins-enabled-label = Enable this plugin
plugins-config-label = Plugin Configuration:
plugins-unknown-agent = Unknown Agent
plugins-category-ai = 🤖 AI Agents
plugins-category-general = ⚙ General
plugins-item-settings = Settings
plugins-item-welcome = Overview
plugins-welcome-title = Welcome to Plugin Manager
plugins-welcome-text = PolyCredo Editor utilizes a modern plugin system based on WebAssembly (WASM). This ensures high performance and maximum security — plugins run in an isolated environment (sandbox) and only have access to what you explicitly authorize.
plugins-welcome-hint = Select a category or a specific plugin from the list on the left to configure it.
plugins-security-info = 🛡 Security: You can manage the file/directory blacklist in the main Settings.
plugins-settings-saved = Plugin settings saved. Restart recommended for some changes.
plugins-placeholder-api-key = API Key (e.g. Gemini, Anthropic)
plugins-placeholder-model = Model ID (e.g. gemini-1.5-flash)
command-name-show-plugins = Plugins

## AI Chat
ai-chat-title = AI Chat Assistant
ai-chat-label-response = Response:
ai-chat-loading = AI is thinking…
ai-chat-label-prompt = Your prompt:
ai-chat-placeholder-prompt = Enter instructions for AI (e.g. "Explain this code")...
ai-chat-btn-send = Send
ai-chat-btn-new = New Thread
ai-chat-settings-title = AI Settings
ai-chat-label-language = Language:
ai-chat-btn-reset = Reset
ai-chat-label-system-prompt = System Prompt:
ai-chat-default-prompt = Expert Rust Developer.
command-name-plugin-gemini = Plugin: Ask Gemini
command-name-plugin-ollama = Plugin: Ask Ollama
command-name-plugin-ai-chat = Plugin: Ask AI Agent

## Semantic Indexing (RAG)
semantic-indexing-title = Semantic Project Indexing
semantic-indexing-init = Initializing ML model (downloading)...
semantic-indexing-processing = Processing: { $processed } / { $total }
semantic-indexing-btn-bg = Run in Background
semantic-indexing-status-bar = Indexing project...

## Settings
settings-suggested-patterns = Suggested patterns:

## Plugin Error
plugin-error-title = Plugin Error
plugin-error-heading = Plugin Failure

## File tree
file-tree-new-file = New File
file-tree-new-dir = New Folder
file-tree-rename = Rename
file-tree-copy = Copy
file-tree-paste = Paste
file-tree-delete = Delete
file-tree-confirm-delete = Delete { $name }?
file-tree-unsafe-name = Invalid name: must not contain /, \ or ..
file-tree-outside-project = Path would lead outside the project
file-tree-paste-error = Cannot paste: { $reason }
file-tree-create-dir-error = Cannot create folder: { $reason }
file-tree-create-file-error = Cannot create file: { $reason }
file-tree-rename-error = Cannot rename: { $reason }
file-tree-delete-error = Cannot delete: { $reason }

## External conflict dialog
conflict-title = File Changed Externally
conflict-message = File "{ $name }" was changed (likely by sandbox promotion), but has unsaved changes in the editor.
conflict-choose = Choose which version you want to keep:
conflict-load-disk = Overwrite from Sandbox
conflict-keep-editor = Keep Project Version
conflict-dismiss = Cancel
conflict-hover-disk = Discard unsaved editor changes and load the version just promoted from sandbox
conflict-hover-keep = Keep your work-in-progress changes in the editor; the sandbox version on disk will be overwritten when you save (Ctrl+S)
conflict-hover-dismiss = Close notification without making changes

md-open-external = ↗ Open in External Viewer

svg-open-external = ↗ Open preview in viewer

svg-modal-title = SVG File
svg-modal-body = This file is an SVG image. Do you want to open it in the system viewer, or edit it as XML text?
svg-modal-edit = Edit as text

## Sandbox Deletion Sync Dialog
sandbox-delete-title = File Deleted in Sandbox
sandbox-delete-msg = The file "{ $name }" was deleted in the AI sandbox, but still exists in the project. What would you like to do?
sandbox-delete-keep-project = Keep in Project (Restore to Sandbox)
sandbox-delete-also-project = Delete from Project Too

## Support Modal
support-modal-title = Support PolyCredo Development
support-modal-body = PolyCredo Editor is developed with a vision of privacy, speed, and secure AI assistant integration. If you like the project, we would be grateful for any support. Your contributions help us dedicate more time to developing new features and maintenance.
support-modal-github = Follow on GitHub
support-modal-donate = Donate to Development


## Settings updates
settings-blacklist = Plugin Blacklist (blocked files)
settings-blacklist-add = Add pattern
settings-blacklist-hint = Patterns like *.env, secret/*. Auto-blocks .gitignore.
semantic-indexing-btn-stop = Stop indexing

# Dependency Wizard
dep-wizard-title = Install Missing Tools
dep-wizard-appimagetool-desc = To create an AppImage package, the { $tool } utility is required. This tool is not part of your system standard repositories.
dep-wizard-install-question = Do you want to download it from the official source and install it to { $path }?
dep-wizard-btn-install = Download and Install
dep-wizard-status-downloading = Downloading...
dep-wizard-status-success = Tool installed successfully.
dep-wizard-status-error = Installation error: { $error }
command-name-install-appimagetool = Install appimagetool

dep-wizard-nsis-desc = The NSIS system utility is required to create Windows .exe installers.
dep-wizard-rpm-desc = The rpmbuild utility is required to create .rpm packages.
dep-wizard-install-cmd-question = This tool can be installed using your system package manager. Do you want to start the installation?
dep-wizard-btn-run-cmd = Start Installation (requires sudo)
dep-wizard-status-running = Installing...
command-name-install-nsis = Install NSIS
command-name-install-rpm = Install rpm-build (dnf)
command-name-install-generate-rpm = Install cargo-generate-rpm
command-name-install-appimage = Install cargo-appimage
command-name-install-flatpak = Install flatpak-builder
command-name-install-snap = Install snapcraft
command-name-configure-lxd = Configure LXD (for build)
command-name-install-deb-tools = Install Debian Build Tools
command-name-install-freebsd-target = Install FreeBSD Target (rustup)
command-name-install-cross = Install cross (cross-compilation)
command-name-install-fpm = Install fpm (gem install fpm)
command-name-install-podman = Install Podman (container engine for cross)
command-name-install-windows-target = Install Windows Target (rustup)
command-name-install-xwin = Install cargo-xwin
command-name-install-clang = Install Clang (LLVM)
command-name-install-lld = Install LLD (Linker)
dep-wizard-xwin-desc = The cargo-xwin tool is required for cross-compiling for Windows MSVC from Linux.
dep-wizard-generate-rpm-desc = The cargo-generate-rpm tool is required to create an .rpm package directly from your Rust project.
dep-wizard-appimage-desc = The cargo-appimage tool is required to create a portable AppImage package directly from your Rust project.
dep-wizard-flatpak-desc = The flatpak-builder tool is required to build and package the application into the Flatpak format.
dep-wizard-snap-desc = The snapcraft tool is required to create Snap packages for Ubuntu and other distributions.
dep-wizard-lxd-desc = LXD is a container system required by snapcraft to build Snap packages. Adds your user to the lxd group and initializes LXD.
dep-wizard-deb-desc = System tools like dpkg-dev, build-essential, and fakeroot are required to create .deb packages.
dep-wizard-freebsd-target-desc = The Rust standard library for x86_64-unknown-freebsd is required to cross-compile for FreeBSD.
dep-wizard-cross-desc = The cross tool enables cross-compilation for FreeBSD and other platforms using Docker/Podman containers.
dep-wizard-fpm-desc = fpm (Effing Package Manager) allows creating native FreeBSD .pkg packages from Linux.
dep-wizard-podman-desc = Podman is a container engine required by the cross tool for cross-compilation. A daemon-free alternative to Docker.
dep-wizard-clang-desc = Clang compiler is required for building native C/C++ dependencies for Windows.
dep-wizard-lld-desc = LLD linker is required for linking Windows binaries on Linux.
dep-wizard-windows-target-desc = Rust standard library for x86_64-pc-windows-msvc is required for compilation.
dep-wizard-zigbuild-desc = cargo-zigbuild is required for cross-compiling for macOS (Intel + Apple Silicon) from Linux. Uses the Zig compiler as linker.
dep-wizard-macos-targets-desc = Rust standard libraries for x86_64-apple-darwin and aarch64-apple-darwin are required to cross-compile for macOS (Intel + Apple Silicon).
dep-wizard-genisoimage-desc = genisoimage is used to create a .dmg disk image from the macOS .app bundle.
dep-wizard-macos-deps-desc = Installs all tools required for macOS builds: cargo-zigbuild (cross-compiler), zig (linker), Rust targets for x86_64-apple-darwin and aarch64-apple-darwin, and LLVM (lipo for Universal Binary).
dep-wizard-llvm-desc = LLVM provides the lipo tool for merging Intel (x86_64) and Apple Silicon (aarch64) binaries into a Universal Binary (.app / .dmg).

menu-build-macos-sub = macOS
menu-build-macos-dmg = Build .dmg (macOS)
command-name-install-macos-deps = Install macOS dependencies (cargo-zigbuild + zig + targets)
command-name-install-llvm = Install LLVM (lipo — Universal Binary)

menu-build-fedora = Fedora
menu-build-debian = Debian / Ubuntu
menu-build-freebsd = FreeBSD
menu-build-freebsd-pkg = Build .pkg (FreeBSD)
menu-build-flatpak-sub = Flatpak
menu-build-snap-sub = Snap
menu-build-appimage-sub = AppImage
menu-build-deb = Build .deb
menu-build-rpm = Build .rpm
menu-build-flatpak = Build Flatpak bundle
menu-build-snap = Build Snap package
menu-build-appimage = Build .AppImage
menu-build-exe = Build .exe (Windows)
menu-build = Build


menu-build-all = All Packages
build-all-status-running = Building…
build-all-status-ok = ✔ All packages built successfully
build-all-status-error = ✘ Build finished with errors (code { $code })
build-all-status-waiting = Waiting to start…
build-all-waiting-output = Starting scripts/build-all.sh…
build-all-btn-close = Close
build-all-btn-run = Run
build-all-btn-rerun = Run again
build-all-not-started = Select packages and click Run
build-all-hint-start = Select packages above and click Run…
