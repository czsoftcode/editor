#[test]
fn app_root_no_longer_exports_ai_core_or_public_cli_alias() {
    let app_mod = include_str!("../src/app/mod.rs");

    assert!(
        !app_mod.contains("pub mod ai_core;"),
        "src/app/mod.rs must not export removed `pub mod ai_core;`"
    );
    assert!(
        !app_mod.contains("pub mod cli;"),
        "src/app/mod.rs must not contain `pub mod cli;`"
    );
    assert!(
        app_mod.contains("pub mod ai_prefs;"),
        "src/app/mod.rs must export replacement `pub mod ai_prefs;`"
    );
}
