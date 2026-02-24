# Error and informational messages

## Files
error-file-read = Error reading file: { $path }
error-file-write = Error writing file: { $path }
error-file-save = Error saving "{ $name }": { $reason }
error-file-deleted = File was deleted: { $path }
error-file-delete = Error deleting { $name }: { $reason }
error-file-rename = Rename error: { $reason }
error-file-create = Error creating file { $name }: { $reason }
error-file-read-only-error = Cannot save "{ $name }" because it could not be read correctly. This tab is now read-only to prevent data loss.
error-safe-mode-blocked = Project is in Safe Mode (read-only). You can only make changes in the Sandbox or disable Safe Mode in Settings.
error-file-watch = File watching error

## Directories
error-folder-create = Error creating folder { $name }: { $reason }
error-folder-delete = Error deleting folder { $name }: { $reason }

## Projects
error-project-create = Error creating project: { $reason }
error-project-open = Error opening project: { $path }
error-project-not-found = Project not found: { $path }
error-project-dir-create = Cannot create project directory: { $reason }
error-cmd-failed = Command failed with code: { $code }
error-cmd-start = Could not start command: { $reason }
error-projects-dir-prepare = Cannot prepare projects directory: { $reason }

## Session
error-session-restore = Project from previous session not found: { $path }
error-session-load = Error loading session.
error-session-save = Error saving session.

## Build
error-build-parse = Error parsing build output.

## Clipboard
error-clipboard = Clipboard error: { $reason }

## IPC
error-ipc-connect = Error connecting to running instance.

## General
error-unknown = An unknown error occurred.

## Informational messages (toast info)
info-file-saved = File saved.
info-project-created = Project { $name } created successfully.
info-session-restored =
    { $count ->
        [one] Restored 1 window from the previous session.
       *[other] Restored { $count } windows from the previous session.
    }
