# Application dialogs

## Startup dialog
startup-title = PolyCredo Editor
startup-subtitle = AI Polyglot Code Editor
startup-open-folder = Open Folder
startup-new-project = New Project
startup-recent-projects = Recent Projects
startup-no-recent = No recent projects
startup-quit = Quit
startup-missing-session =
    { $count ->
        [one] 1 project from the previous session was not found.
       *[other] { $count } projects from the previous session were not found.
    }
startup-missing-session-label = Projects from the previous session could not be restored:
startup-path-label = Path:

## Open project dialog
open-project-title = Open Project
open-project-question = A project is already open. Where would you like to open the new one?
open-project-in-window = In This Window
open-project-new-window = In New Window
open-project-cancel = Cancel

## New project wizard
wizard-title = New Project
wizard-project-type = Project Type
wizard-project-name = Project Name
wizard-project-path = Path
wizard-type-rust = Rust
wizard-type-symfony = Symfony
wizard-creating = Creating project…
wizard-name-hint = Only letters, digits, _ and - are allowed
wizard-error-empty-name = Project name cannot be empty.
wizard-error-invalid-name = Invalid name. Only letters, digits, _ and - are allowed.
wizard-error-starts-with-dash = Name must not start with a dash.
wizard-error-exists = A project with this name already exists at the given path.
wizard-error-create = Error creating project: { $reason }

## Close project dialog
close-project-title = Close Project
close-project-message = Are you sure you want to close this project?
close-project-confirm = Close
close-project-cancel = Cancel

## Quit dialog
quit-title = Quit Application
quit-message = Are you sure you want to quit PolyCredo Editor?
quit-confirm = Quit
quit-cancel = Cancel

## About dialog
about-title = About
about-version = Version { $version }
about-build = Build { $build }
about-description = AI Polyglot Code Editor
about-copyright = © 2024–2026 PolyCredo
about-close = Close

## Confirmation dialogs (generic)
confirm-delete-file = Are you sure you want to delete { $name }?
confirm-delete-folder = Are you sure you want to delete { $name } and all its contents?
confirm-delete-confirm = Delete
confirm-delete-cancel = Cancel

## Rename
rename-title = Rename
rename-label = New name:
rename-confirm = Rename
rename-cancel = Cancel
