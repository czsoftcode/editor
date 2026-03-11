use std::fs;
use std::path::Path;

const CRITICAL_RUNTIME_FILES: &[&str] = &[
    "src/app/ui/terminal/ai_chat/logic.rs",
    "src/app/ui/terminal/ai_chat/slash.rs",
    "src/app/ui/terminal/ai_chat/approval.rs",
    "src/app/ui/background_events.rs",
    "src/app/ai_core/executor.rs",
];

#[test]
fn phase32_namespace_guard_blocks_legacy_cli_paths() {
    for rel in CRITICAL_RUNTIME_FILES {
        let path = Path::new(rel);
        let content = fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        assert!(
            !content.contains("crate::app::cli"),
            "legacy namespace crate::app::cli found in {}",
            path.display()
        );
        assert!(
            !content.contains("app::cli"),
            "legacy namespace app::cli found in {}",
            path.display()
        );
    }
}
