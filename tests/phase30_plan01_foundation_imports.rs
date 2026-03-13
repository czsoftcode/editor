const TARGET_FILES: &[&str] = &[
    "src/settings.rs",
    "src/app/types.rs",
    "src/app/ui/workspace/state/mod.rs",
    "src/app/ui/workspace/state/init.rs",
];

#[test]
fn foundation_subset_uses_ai_core_namespace() {
    for file in TARGET_FILES {
        let content = std::fs::read_to_string(file)
            .unwrap_or_else(|err| panic!("failed to read {file}: {err}"));
        assert!(
            !content.contains("crate::app::cli"),
            "{file} must not contain crate::app::cli import path"
        );
        assert!(
            !content.contains("app::cli"),
            "{file} must not contain app::cli path"
        );
    }
}
