#[test]
fn app_root_exports_ai_core_without_public_cli_alias() {
    let app_mod = include_str!("../src/app/mod.rs");

    assert!(
        app_mod.contains("pub mod ai_core;"),
        "src/app/mod.rs must export `pub mod ai_core;`"
    );
    assert!(
        !app_mod.contains("pub mod cli;"),
        "src/app/mod.rs must not contain `pub mod cli;`"
    );
}
