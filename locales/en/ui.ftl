# General user interface elements

## Panels
panel-files = Files
panel-build = Build
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

## Status bar
statusbar-line-col = Line { $line }, Column { $col }
statusbar-encoding = UTF-8
statusbar-unsaved = Unsaved
statusbar-saving = Saving…
statusbar-saved = Saved
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

## Terminal
terminal-unavailable = Terminal is not available.
terminal-retry = Try again
terminal-exited = [Process exited — press R to restart]

## Go to Line dialog (Ctrl+G)
goto-line-prompt = Go to line:
goto-line-placeholder = line number

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
ai-float-dock = Dock to panel
ai-float-undock = Undock to floating window
ai-tab-close-hover = Close tab
ai-tab-new-hover = New terminal tab

## Settings
settings-title = Settings
settings-language = Language
settings-language-restart = Language changes take effect immediately.
settings-theme = Theme
settings-theme-dark = Dark
settings-theme-light = Light
settings-editor-font = Editor — font size
settings-ai-font = AI Terminal — font size
settings-default-path = Default project path
settings-creates-in = Will be created at:

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
conflict-message = File "{ $name }" was changed by another program, but has unsaved changes in the editor.
conflict-choose = Choose which version should win:
conflict-load-disk = Load from disk
conflict-keep-editor = Keep mine
conflict-dismiss = Dismiss
conflict-hover-disk = Discard editor changes and load the version saved on disk
conflict-hover-keep = Keep editor changes; the disk file will be overwritten on save
conflict-hover-dismiss = Close notification without changes

md-open-external = ⧉ Open in External Viewer

svg-open-external = ⧉ Open preview in viewer

svg-modal-title = SVG File
svg-modal-body = This file is an SVG image. Do you want to open it in the system viewer, or edit it as XML text?
svg-modal-edit = Edit as text
