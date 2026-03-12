use std::fs;

#[test]
fn phase37_i18n_restore_preview_parity() {
    let required_keys = [
        "file-tree-trash-preview-title",
        "file-tree-trash-preview-filter",
        "file-tree-trash-preview-loading",
        "file-tree-trash-preview-no-results",
        "file-tree-trash-preview-kind-file",
        "file-tree-trash-preview-kind-dir",
        "file-tree-trash-preview-restore",
        "file-tree-restore-conflict-title",
        "file-tree-restore-conflict-message",
        "file-tree-restore-as-copy",
        "file-tree-restore-success",
        "file-tree-restore-error",
    ];

    for locale in [
        "locales/cs/ui.ftl",
        "locales/en/ui.ftl",
        "locales/de/ui.ftl",
        "locales/ru/ui.ftl",
        "locales/sk/ui.ftl",
    ] {
        let ftl = fs::read_to_string(locale).expect("failed to read locale file");
        for key in required_keys {
            assert!(
                ftl.contains(key),
                "locale {locale} must contain key `{key}` for phase37 restore/preview parity"
            );
        }
    }
}
