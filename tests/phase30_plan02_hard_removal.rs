use std::path::Path;

#[test]
fn cli_tree_is_physically_removed_and_not_registered_in_app_root() {
    assert!(
        !Path::new("src/app/cli").exists(),
        "src/app/cli directory must be removed"
    );

    let app_mod = std::fs::read_to_string("src/app/mod.rs")
        .expect("failed to read src/app/mod.rs");

    assert!(
        !app_mod.contains("mod cli;"),
        "src/app/mod.rs must not reference legacy cli module"
    );
}
