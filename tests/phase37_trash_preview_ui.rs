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
