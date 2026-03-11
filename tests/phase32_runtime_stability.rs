use std::fs;
use std::path::Path;

#[test]
fn phase32_runtime_stability_chat_runtime_is_removed_in_phase33() {
    let removed = [
        "src/app/ui/terminal/ai_chat/logic.rs",
        "src/app/ui/terminal/ai_chat/slash.rs",
        "src/app/ai_core/executor.rs",
    ];
    for rel in removed {
        assert!(
            !Path::new(rel).exists(),
            "phase 33 must keep removed runtime file absent: {rel}"
        );
    }

    let background = fs::read_to_string("src/app/ui/background.rs")
        .unwrap_or_else(|err| panic!("failed to read src/app/ui/background.rs: {err}"));
    assert!(
        !background.contains("ApprovalDecision::Approve"),
        "background runtime must not contain approval flow after chat removal",
    );
    assert!(
        !background.contains("should_apply_async_result("),
        "background runtime must not contain slash stale guards after chat removal",
    );
}
