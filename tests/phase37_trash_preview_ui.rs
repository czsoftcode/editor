use std::fs;

#[test]
fn phase37_open_trash_preview_from_menu() {
    let menubar = fs::read_to_string("src/app/ui/workspace/menubar/mod.rs")
        .expect("failed to read menubar module");
    let project_menu = fs::read_to_string("src/app/ui/workspace/menubar/project.rs")
        .expect("failed to read project menu");

    assert!(
        menubar.contains("pub trash_preview: bool"),
        "menu actions must expose trash_preview flag"
    );
    assert!(
        menubar.contains("ws.file_tree.request_open_trash_preview();"),
        "menu action handler must route trash preview open request"
    );
    assert!(
        project_menu.contains("menu-project-trash-preview"),
        "project menu must expose trash preview entrypoint"
    );
}

#[test]
fn phase37_open_trash_preview_from_command() {
    let registry =
        fs::read_to_string("src/app/registry/mod.rs").expect("failed to read command registry");
    let palette = fs::read_to_string("src/app/ui/widgets/command_palette.rs")
        .expect("failed to read command palette");

    assert!(
        registry.contains("project.trash_preview"),
        "registry must expose command id for trash preview"
    );
    assert!(
        registry.contains("command-name-trash-preview"),
        "registry command must use i18n key for trash preview"
    );
    assert!(
        palette.contains("TrashPreview"),
        "command palette must define TrashPreview command id"
    );
    assert!(
        palette.contains("actions.trash_preview = true"),
        "command execution must route trash preview to menu actions"
    );
}

#[test]
fn phase37_preview_filtering() {
    let preview = fs::read_to_string("src/app/ui/file_tree/preview.rs")
        .expect("failed to read file tree preview module");
    let file_tree_mod =
        fs::read_to_string("src/app/ui/file_tree/mod.rs").expect("failed to read file tree module");

    assert!(
        preview.contains("show_trash_preview_dialog"),
        "trash preview modal function must exist"
    );
    assert!(
        file_tree_mod.contains("show_trash_preview_dialog"),
        "phase 37 verification hook must retain show_trash_preview_dialog symbol"
    );
    assert!(
        preview.contains("self.trash_preview_filter.to_ascii_lowercase()"),
        "preview modal must filter list by normalized text query"
    );
    assert!(
        preview.contains("file-tree-trash-preview-filter")
            && preview.contains("file-tree-trash-preview-no-results"),
        "preview modal must surface filter input and empty-state copy"
    );
}

#[test]
fn phase37_restore_job_result_to_pending() {
    let file_tree_mod =
        fs::read_to_string("src/app/ui/file_tree/mod.rs").expect("failed to read file tree module");

    assert!(
        file_tree_mod.contains("restore_rx: Option<mpsc::Receiver<RestoreJobResult>>"),
        "file tree must track restore background receiver"
    );
    assert!(
        file_tree_mod.contains("pending_restored: Option<PathBuf>"),
        "file tree must expose pending restored path handoff"
    );
    assert!(
        file_tree_mod.contains("RestoreJobResult::Restored(path)")
            && file_tree_mod.contains("self.pending_restored = Some(path.clone())"),
        "restore polling must route successful result into pending_restored"
    );
}

#[test]
fn phase37_conflict_routes_to_modal() {
    let preview = fs::read_to_string("src/app/ui/file_tree/preview.rs")
        .expect("failed to read file tree preview module");
    let dialogs = fs::read_to_string("src/app/ui/file_tree/dialogs.rs")
        .expect("failed to read file tree dialogs");

    assert!(
        preview.contains("RestoreJobResult::Conflict(selected_path)"),
        "restore conflict must route into explicit conflict result"
    );
    assert!(
        dialogs.contains("show_restore_conflict_dialog"),
        "file tree must render dedicated restore conflict modal"
    );
    assert!(
        dialogs.contains("restore_from_trash_as_copy"),
        "conflict dialog must offer non-destructive restore-as-copy branch"
    );
}

#[test]
fn phase37_conflict_has_no_overwrite_action() {
    let dialogs = fs::read_to_string("src/app/ui/file_tree/dialogs.rs")
        .expect("failed to read file tree dialogs");

    assert!(
        dialogs.contains("file-tree-restore-as-copy"),
        "conflict modal must expose restore-as-copy action"
    );
    assert!(
        !dialogs.contains("replace existing") && !dialogs.contains("force restore"),
        "conflict modal must not offer overwrite path"
    );
}

#[test]
fn phase37_trash_preview_ui() {
    let file_tree_mod =
        fs::read_to_string("src/app/ui/file_tree/mod.rs").expect("failed to read file tree module");
    let preview = fs::read_to_string("src/app/ui/file_tree/preview.rs")
        .expect("failed to read file tree preview module");
    let dialogs = fs::read_to_string("src/app/ui/file_tree/dialogs.rs")
        .expect("failed to read file tree dialogs module");

    assert!(
        file_tree_mod.contains("request_open_trash_preview")
            && preview.contains("show_trash_preview_dialog")
            && dialogs.contains("show_restore_conflict_dialog"),
        "phase37 ui flow must keep preview open trigger, preview modal, and conflict modal hooks"
    );
}
