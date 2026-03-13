use std::fs;

#[test]
fn phase35_trash_path_contract_exists() {
    let project_config = fs::read_to_string("src/app/project_config.rs")
        .expect("failed to read src/app/project_config.rs");
    assert!(
        project_config.contains("fn trash_dir_path"),
        "project_config must expose trash_dir_path helper"
    );
    assert!(
        project_config.contains(".join(TRASH_DIR)"),
        "trash_dir_path must point to .polycredo/trash"
    );

    let trash_mod =
        fs::read_to_string("src/app/trash.rs").expect("failed to read src/app/trash.rs");
    for field in [
        "trash_name",
        "original_relative_path",
        "deleted_at",
        "entry_kind",
    ] {
        assert!(
            trash_mod.contains(field),
            "trash metadata contract must contain `{field}`"
        );
    }
}
