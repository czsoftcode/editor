use std::fs;
use std::path::Path;

const EXISTING_FILES: &[&str] = &["src/app/ui/workspace/modal_dialogs/settings.rs"];
const REMOVED_FILES: &[&str] = &[
    "src/app/ui/ai_panel.rs",
    "src/app/ai_core/mod.rs",
    "src/app/ai_core/types.rs",
];

#[test]
fn readiness_gate_has_no_legacy_cli_namespace_in_critical_callsites() {
    for rel in EXISTING_FILES {
        let path = Path::new(rel);
        let content = fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        assert!(
            !content.contains("crate::app::cli"),
            "legacy cli namespace found in {}",
            path.display()
        );
    }

    for rel in REMOVED_FILES {
        let path = Path::new(rel);
        assert!(
            !path.exists(),
            "removed file must stay deleted: {}",
            path.display()
        );
    }
}
