use std::fs;

#[test]
fn phase35_delete_path_uses_background_task() {
    let dialogs = fs::read_to_string("src/app/ui/file_tree/dialogs.rs")
        .expect("failed to read src/app/ui/file_tree/dialogs.rs");
    assert!(
        dialogs.contains("spawn_task(move ||"),
        "delete flow must run in background task"
    );
    assert!(
        dialogs.contains("move_path_to_trash(&root, &path)"),
        "delete flow must route through trash move"
    );
    assert!(
        !dialogs.contains("remove_dir_all(&path)") && !dialogs.contains("remove_file(&path)"),
        "delete dialog must not hard-delete directly"
    );
}
