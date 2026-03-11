use std::fs;
use std::path::Path;

fn read(rel: &str) -> String {
    fs::read_to_string(Path::new(rel)).unwrap_or_else(|err| panic!("failed to read {rel}: {err}"))
}

#[test]
fn prompt_and_retry_regression_markers_exist() {
    let logic = read("src/app/ui/terminal/ai_chat/logic.rs");
    assert!(logic.contains("normalize_prompt_input("));
    assert!(logic.contains("ws.ai.chat.retry_prompt = Some(prompt.clone());"));

    let background = read("src/app/ui/background.rs");
    assert!(background.contains("drain_stream_events(rx, &ws.ai.chat.streaming_buffer)"));
    assert!(background.contains("ws.ai.chat.retry_available = true;"));
    assert!(background.contains("Use Retry to send the last prompt again."));
}

#[test]
fn slash_stale_guard_and_approval_paths_are_explicit() {
    let slash = read("src/app/ui/terminal/ai_chat/slash.rs");
    assert!(slash.contains("pub fn should_apply_async_result("));

    let background = read("src/app/ui/background.rs");
    assert!(background.contains("ApprovalDecision::Approve"));
    assert!(background.contains("ApprovalDecision::Deny"));
    assert!(background.contains("should_apply_async_result("));
}

#[test]
fn approval_denial_must_emit_error_toast_for_visibility() {
    let background = read("src/app/ui/background.rs");
    assert!(
        background.contains("ws.toasts.push(Toast::error(format!(\"AI tool `{}`: {}\", pending.tool_name, output)));"),
        "approval denial branch must emit toast error with tool name and message",
    );

    let executor = read("src/app/ai_core/executor.rs");
    assert!(executor.contains("pub fn process_approval_response("));
}
