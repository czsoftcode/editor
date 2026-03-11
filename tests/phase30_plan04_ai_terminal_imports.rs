use std::fs;
use std::path::Path;

const ACTIVE_FILES: &[&str] = &[
    "src/app/ui/terminal/right/ai_bar.rs",
    "src/app/ui/background.rs",
];
const REMOVED_FILES: &[&str] = &[
    "src/app/ui/terminal/ai_chat/mod.rs",
    "src/app/ui/terminal/ai_chat/logic.rs",
    "src/app/ui/terminal/ai_chat/render.rs",
    "src/app/ui/widgets/ai/chat/mod.rs",
    "src/app/ui/widgets/ai/chat/settings.rs",
];

#[test]
fn ai_terminal_subset_has_no_legacy_cli_namespace_and_removed_chat_runtime() {
    for rel in ACTIVE_FILES {
        let path = Path::new(rel);
        let content = fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        assert!(
            !content.contains("crate::app::cli"),
            "legacy cli namespace found in {}",
            path.display()
        );
        assert!(
            !content.contains("app::cli"),
            "legacy cli namespace found in {}",
            path.display()
        );
    }
    for rel in REMOVED_FILES {
        let path = Path::new(rel);
        assert!(
            !path.exists(),
            "phase 33 must keep removed file absent: {}",
            path.display()
        );
    }
}
