use std::fs;
use std::path::Path;

const FILES: &[&str] = &[
    "src/app/ui/ai_panel.rs",
    "src/app/ui/workspace/modal_dialogs/settings.rs",
    "src/app/ai_core/mod.rs",
    "src/app/ai_core/types.rs",
];

#[test]
fn readiness_gate_has_no_legacy_cli_namespace_in_critical_callsites() {
    for rel in FILES {
        let path = Path::new(rel);
        let content = fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        assert!(
            !content.contains("crate::app::cli"),
            "legacy cli namespace found in {}",
            path.display()
        );
    }
}
