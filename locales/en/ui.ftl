# General user interface elements

## Panels
panel-files = Files
btn-tree-project = Project
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
btn-run = ▶ Run
btn-run-new = ▶ Run+
btn-test = ▶ Test
btn-clean = ✖ Clean
btn-create-deb = Create .deb
hover-create-deb = Build and create a development .deb package with build number
btn-run-profile = ▶ Run Profile...
btn-git-profile =  Git...
btn-edit-profiles = ⚙ Edit
runner-none = No profiles defined.

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

## Status bar
statusbar-line-col = Line { $line }, Column { $col }
statusbar-encoding = UTF-8
statusbar-unsaved = Unsaved
statusbar-saving = Saving…
statusbar-saved = Saved
statusbar-lsp-initializing = LSP initializing...
statusbar-filetype-plain = Plain Text
statusbar-save-mode-automatic = Auto Save
statusbar-save-mode-manual = Manual Save

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
btn-cancel = Discard
btn-close = Close
cancel-confirm-title = Discard changes?
cancel-confirm-msg = Are you sure you want to discard all unsaved changes and close this window?
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
ai-diff-heading = Review AI Proposed Changes
ai-diff-new-file = New file proposed
ai-float-dock = Dock to panel
ai-float-undock = Undock to floating window
ai-viewport-open = Open in separate window
ai-tab-close-hover = Close tab
ai-tab-new-hover = New terminal tab
ai-staged-bar-msg = AI suggested changes to the project
ai-staged-bar-review = Review Changes
ai-staged-bar-promote-all = Promote All
ai-staged-modal-hint = Click a file to review differences and accept changes:
ai-staged-files = Suggested Changes
ai-staged-new = [NEW]
ai-staged-mod = [MOD]
ai-staged-del = [DELETED]
ai-promotion-success-title = Changes Applied
ai-promotion-success-body = The following file has been successfully updated in your project:
ai-promotion-success = Changes successfully applied to the project.
ai-promotion-all-success = Successfully promoted { $count } files to project.
ai-promotion-failed = Failed to apply changes: { $error }

## Plugin Permissions

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
settings-light-variant = Light variant
settings-light-variant-warm-ivory = Warm Ivory
settings-light-variant-cool-gray = Cool Gray
settings-light-variant-sepia = Sepia
settings-light-variant-warm-tan = Warm Tan
settings-dark-variant = Dark variant
settings-dark-variant-default = Default
settings-dark-variant-midnight = Midnight
settings-save-mode-title = Save mode
settings-save-mode-automatic = Automatic Save
settings-save-mode-manual = Manual Save
settings-save-mode-toast-automatic = Automatic Save enabled
settings-save-mode-toast-manual = Manual Save enabled
settings-auto-show-diff = Automatically open AI change preview
settings-conflict-title = Settings Changed
settings-conflict-message = Settings were updated in another window. Reload to see the latest values, or keep editing your current draft.
settings-conflict-reload = Reload
settings-conflict-keep = Keep editing
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

## Command Palette – AI plugins

## Semantic Indexing (RAG)
semantic-indexing-title = Semantic Project Indexing
semantic-indexing-init = Initializing ML model (downloading)...
semantic-indexing-processing = Processing: { $processed } / { $total }
semantic-indexing-btn-bg = Run in Background
semantic-indexing-status-bar = Indexing project...

## Settings
settings-suggested-patterns = Suggested patterns:

## Plugin Error

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
file-tree-delete-move-failed-reason = Move to trash failed: { $reason }
file-tree-delete-move-failed-guidance = Check permissions and file locks, then try again.
file-tree-delete-move-failed-reason-permission = insufficient permissions
file-tree-delete-move-failed-reason-locked = file or folder is currently in use
file-tree-delete-move-failed-reason-missing = item no longer exists in the project
file-tree-delete-move-failed-reason-internal-trash = internal trash directory cannot be deleted
file-tree-delete-move-failed-reason-generic = unexpected I/O error

## External conflict dialog
conflict-title = File Changed Externally
conflict-message = File "{ $name }" was changed outside the editor, but has unsaved changes in the editor.
conflict-choose = Choose which version you want to keep:
conflict-load-disk = Load from Disk
conflict-keep-editor = Keep Editor Version
conflict-dismiss = Cancel
conflict-hover-disk = Discard unsaved editor changes and load the version changed on disk
conflict-hover-keep = Keep your work-in-progress changes in the editor; the version on disk will be overwritten when you save (Ctrl+S)
conflict-hover-dismiss = Close notification without making changes

md-open-external = ↗ Open in External Viewer
md-layout-pod-sebou = Stacked
md-layout-vedle-sebe = Side by side
md-layout-jenom-kod = Code only
md-layout-jenom-nahled = Preview only

svg-open-external = ↗ Open preview in viewer

svg-modal-title = SVG File
svg-modal-body = This file is an SVG image. Do you want to open it in the system viewer, or edit it as XML text?
svg-modal-edit = Edit as text

## Support Modal
support-modal-title = Support PolyCredo Development
support-modal-body = PolyCredo Editor is developed with a vision of privacy, speed, and secure AI assistant integration. If you like the project, we would be grateful for any support. Your contributions help us dedicate more time to developing new features and maintenance.
support-modal-github = Follow on GitHub
support-modal-donate = Donate to Development
semantic-indexing-btn-stop = Stop indexing

## Unsaved close guard
unsaved-close-guard-title = Unsaved changes
unsaved-close-guard-message = This file has unsaved changes. What would you like to do before closing?
unsaved-close-guard-save = Save and close
unsaved-close-guard-discard = Discard changes
unsaved-close-guard-cancel = Cancel

## Settings updates
settings-blacklist = Plugin Blacklist (blocked files)
settings-blacklist-add = Add pattern
settings-blacklist-hint = Patterns like *.env, secret/*. Auto-blocks .gitignore.
semantic-indexing-btn-stop = Stop indexing

# Dependency Wizard
dep-wizard-title = Install Missing Tools
dep-wizard-install-question = Do you want to download it from the official source and install it to { $path }?
dep-wizard-btn-install = Download and Install
dep-wizard-status-downloading = Downloading...
dep-wizard-status-success = Tool installed successfully.
dep-wizard-status-error = Installation error: { $error }

dep-wizard-install-cmd-question = This tool can be installed using your system package manager. Do you want to start the installation?
dep-wizard-btn-run-cmd = Start Installation (requires sudo)
dep-wizard-status-running = Installing...

dep-wizard-appimagetool-desc = The appimagetool is required for the final packaging of the .AppImage bundle.
