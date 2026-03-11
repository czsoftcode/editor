use std::fs;
use std::path::Path;

const FILES: &[&str] = &[
    "src/app/ui/terminal/ai_chat/mod.rs",
    "src/app/ui/terminal/ai_chat/logic.rs",
    "src/app/ui/terminal/ai_chat/render.rs",
    "src/app/ui/terminal/right/ai_bar.rs",
    "src/app/ui/widgets/ai/chat/mod.rs",
    "src/app/ui/widgets/ai/chat/settings.rs",
    "src/app/ui/background.rs",
];

#[test]
fn ai_terminal_subset_uses_ai_core_instead_of_cli_namespace() {
    for rel in FILES {
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
}
