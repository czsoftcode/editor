use std::fs;

#[test]
fn phase36_block_trash_delete() {
    let trash_mod = fs::read_to_string("src/app/trash.rs").expect("failed to read src/app/trash.rs");
    assert!(
        trash_mod.contains("is_inside_trash_dir"),
        "trash engine must explicitly guard deletes inside `.polycredo/trash`"
    );
    assert!(
        trash_mod.contains("nelze smazat interni `.polycredo/trash`"),
        "trash delete guard must expose user-facing reason for blocked operation"
    );
    assert!(
        trash_mod.contains("zkuste smazat polozku mimo trash"),
        "blocked delete message must include next-step hint for toast surfacing"
    );
}

#[test]
fn phase36_move_file_to_trash() {
    let project_config =
        fs::read_to_string("src/app/project_config.rs").expect("failed to read src/app/project_config.rs");
    assert!(
        project_config.contains("fn project_trash_dir"),
        "project config must expose stable project_trash_dir helper"
    );

    let trash_mod = fs::read_to_string("src/app/trash.rs").expect("failed to read src/app/trash.rs");
    assert!(
        trash_mod.contains("resolve_trash_destination"),
        "trash move contract must expose explicit destination resolver"
    );
    assert!(
        trash_mod.contains("std::fs::rename(&source_abs, &destination_abs)"),
        "file delete must be routed through move-to-trash rename"
    );
    assert!(
        !trash_mod.contains("std::fs::remove_file"),
        "file delete contract must not contain hard-delete fallback"
    );
}

#[test]
fn phase36_move_dir_to_trash() {
    let trash_mod = fs::read_to_string("src/app/trash.rs").expect("failed to read src/app/trash.rs");
    assert!(
        trash_mod.contains("TrashEntryKind::Directory"),
        "directory delete must be represented in trash metadata contract"
    );
    assert!(
        trash_mod.contains("relative_path"),
        "directory move must preserve relative tree structure in trash target"
    );
    assert!(
        !trash_mod.contains("std::fs::remove_dir_all"),
        "directory delete contract must not contain hard-delete fallback"
    );
}

#[test]
fn phase36_collision_suffix() {
    let trash_mod = fs::read_to_string("src/app/trash.rs").expect("failed to read src/app/trash.rs");
    assert!(
        trash_mod.contains("trash-{deleted_at}-{attempt}"),
        "collision policy must use timestamp + counter naming"
    );
    assert!(
        trash_mod.contains("for attempt in 0..1000_u32"),
        "collision policy must retry deterministic counter window"
    );
}

#[test]
fn phase36_fail_closed() {
    let trash_mod = fs::read_to_string("src/app/trash.rs").expect("failed to read src/app/trash.rs");
    assert!(
        trash_mod.contains("format_fail_closed_move_error"),
        "fail-closed path must use dedicated move-error formatter"
    );
    assert!(
        trash_mod.contains("puvodni polozka zustava beze zmeny"),
        "move failure must explicitly preserve source item contract"
    );
    assert!(
        trash_mod.contains("zkontrolujte prava a zkuste akci znovu"),
        "fail-closed error must include actionable guidance for user"
    );
    assert!(
        !trash_mod.contains("std::fs::remove_file") && !trash_mod.contains("std::fs::remove_dir_all"),
        "failure path must never fallback to hard delete"
    );
}
