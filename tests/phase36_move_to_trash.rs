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
