use std::fs;

#[test]
fn phase36_error_toast() {
    let dialogs = fs::read_to_string("src/app/ui/file_tree/dialogs.rs")
        .expect("failed to read src/app/ui/file_tree/dialogs.rs");
    assert!(
        dialogs.contains("format_delete_toast_error"),
        "delete workflow must map trash errors through a dedicated toast formatter"
    );
    assert!(
        dialogs.contains("file-tree-delete-move-failed-reason")
            && dialogs.contains("file-tree-delete-move-failed-guidance"),
        "delete toast formatter must compose reason + actionable guidance from i18n keys"
    );
    assert!(
        !dialogs.contains("DeleteJobResult::Error(format!(\"trash move failed: {err}\"))"),
        "raw engine error text must not be forwarded directly to toasts"
    );

    for locale in [
        "locales/cs/ui.ftl",
        "locales/en/ui.ftl",
        "locales/de/ui.ftl",
        "locales/ru/ui.ftl",
        "locales/sk/ui.ftl",
    ] {
        let ftl = fs::read_to_string(locale).expect("failed to read locale file");
        assert!(
            ftl.contains("file-tree-delete-move-failed-reason")
                && ftl.contains("file-tree-delete-move-failed-guidance"),
            "locale {locale} must provide delete toast wording parity for phase 36"
        );
    }
}
