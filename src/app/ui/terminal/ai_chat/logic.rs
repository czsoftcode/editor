use crate::app::ai_core::AiMessage;
use crate::app::ai_core::provider::{AiProvider, ProviderConfig};
use crate::app::ai_core::runtime_provider::OllamaProvider;
use crate::app::ai_core::tools::get_standard_tools;
use crate::app::types::{AppShared, Toast};
use crate::app::ui::workspace::state::WorkspaceState;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub fn send_query_to_agent(
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    _i18n: &crate::i18n::I18n,
) {
    if ws.ai.chat.prompt.trim().is_empty() {
        return;
    }

    // Slash command intercept — works even when provider is disconnected
    if ws.ai.chat.prompt.starts_with('/') {
        super::slash::dispatch(ws, shared);
        return;
    }

    if !ws.ai_provider_is_connected() {
        ws.toasts
            .push(Toast::error("AI provider is not connected."));
        return;
    }

    // Initialize tool executor lazily on first AI chat
    if ws.tool_executor.is_none() {
        let root = ws.root_path.clone();
        // Load blacklist patterns from settings
        let settings = crate::settings::Settings::load();
        let blacklist = if settings.ai_file_blacklist_patterns.is_empty() {
            None
        } else {
            Some(settings.ai_file_blacklist_patterns)
        };
        ws.tool_executor = Some(crate::app::ai_core::executor::ToolExecutor::new(
            root, blacklist, None,
        ));
    }

    // Build messages from conversation history
    let mut messages: Vec<AiMessage> = Vec::new();

    // System prompt + reasoning depth mandate
    let reasoning_mandate = ws.ai.settings.reasoning_depth.get_reasoning_mandate();
    let expertise_mandate = ws.ai.settings.expertise.get_persona_mandate();

    // Build composite system message: base prompt + expertise + reasoning depth
    let mut system_parts: Vec<&str> = Vec::new();
    if !ws.ai.chat.system_prompt.is_empty() {
        system_parts.push(&ws.ai.chat.system_prompt);
    }
    if !expertise_mandate.is_empty() {
        system_parts.push(expertise_mandate);
    }
    system_parts.push(reasoning_mandate);

    let system_content = system_parts.join("\n\n");

    // Append editor context (open files, build errors, git, etc.)
    let context_str = build_editor_context(ws);
    let full_system = if context_str.is_empty() {
        system_content
    } else {
        format!("{}\n\n{}", system_content, context_str)
    };

    messages.push(AiMessage {
        role: "system".to_string(),
        content: full_system,
        monologue: Vec::new(),
        timestamp: 0,
        tool_call_name: None,
        tool_call_id: None,
        tool_result_for_id: None,
        tool_is_error: false,
        tool_call_arguments: None,
    });

    // Conversation history (multi-turn)
    for (q, a) in &ws.ai.chat.conversation {
        // Skip system messages — they're slash command output, not AI conversation
        if a.starts_with(super::slash::SYSTEM_MSG_MARKER) {
            continue;
        }
        if !q.is_empty() {
            messages.push(AiMessage {
                role: "user".to_string(),
                content: q.clone(),
                monologue: Vec::new(),
                timestamp: 0,
                tool_call_name: None,
                tool_call_id: None,
                tool_result_for_id: None,
                tool_is_error: false,
                tool_call_arguments: None,
            });
        }
        if !a.is_empty() {
            messages.push(AiMessage {
                role: "assistant".to_string(),
                content: a.clone(),
                monologue: Vec::new(),
                timestamp: 0,
                tool_call_name: None,
                tool_call_id: None,
                tool_result_for_id: None,
                tool_is_error: false,
                tool_call_arguments: None,
            });
        }
    }

    // Current prompt
    let prompt = ws.ai.chat.prompt.clone();
    ws.ai.chat.retry_prompt = Some(prompt.clone());
    ws.ai.chat.retry_available = false;
    messages.push(AiMessage {
        role: "user".to_string(),
        content: prompt.clone(),
        monologue: Vec::new(),
        timestamp: 0,
        tool_call_name: None,
        tool_call_id: None,
        tool_result_for_id: None,
        tool_is_error: false,
        tool_call_arguments: None,
    });

    // Push empty response slot
    ws.ai
        .chat
        .conversation
        .push((prompt.clone(), String::new()));

    // Reset state
    ws.ai.chat.prompt.clear();
    ws.ai.chat.loading = true;
    ws.ai.chat.auto_scroll = true;
    ws.ai.chat.streaming_buffer.clear();
    ws.ai.cancellation_token = Arc::new(AtomicBool::new(false));

    // Update prompt history
    if ws.ai.chat.history.last() != Some(&prompt) {
        ws.ai.chat.history.push(prompt);
    }
    ws.ai.chat.history_index = None;

    // Create provider and start streaming
    let (base_url, model, api_key) = ws.ai_provider_connection_parts();
    let provider = OllamaProvider::new(base_url.clone(), model.clone(), api_key.clone());

    let config = ProviderConfig {
        base_url,
        model,
        temperature: ws.ai.settings.temperature,
        num_ctx: ws.ai.settings.num_ctx,
        api_key,
        top_p: ws.ai.settings.top_p,
        top_k: ws.ai.settings.top_k,
        repeat_penalty: ws.ai.settings.repeat_penalty,
        seed: ws.ai.settings.seed,
    };

    // stream_chat() spawns its own thread and returns Receiver immediately
    let tools = get_standard_tools();
    ws.ai.chat.stream_rx = Some(provider.stream_chat(messages, config, tools));
}

pub fn can_retry_last_prompt(ws: &WorkspaceState) -> bool {
    !ws.ai.chat.loading && ws.ai.chat.retry_available && ws.ai.chat.retry_prompt.is_some()
}

pub fn retry_last_prompt(
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) {
    if !can_retry_last_prompt(ws) {
        return;
    }
    if let Some(prompt) = ws.ai.chat.retry_prompt.clone() {
        ws.ai.chat.prompt = prompt;
        send_query_to_agent(ws, shared, i18n);
    }
}

/// Builds an editor context string from workspace state for injection into system message.
fn build_editor_context(ws: &WorkspaceState) -> String {
    use crate::app::ai_core::{
        AiBuildErrorContext, AiContextPayload, AiFileContext, AiGitFileStatus,
    };

    let mut payload = AiContextPayload {
        project_name: ws
            .root_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        project_root: ".".to_string(),
        ..AiContextPayload::default()
    };

    // Open files
    for (i, tab) in ws.editor.tabs.iter().enumerate() {
        let rel_path = tab
            .path
            .strip_prefix(&ws.root_path)
            .unwrap_or(&tab.path)
            .to_string_lossy()
            .into_owned();
        let is_active = Some(i) == ws.editor.active_tab;
        let file_ctx = AiFileContext {
            path: rel_path,
            content: if is_active {
                Some(tab.content.clone())
            } else {
                None
            },
            is_active,
        };
        payload.open_files.push(file_ctx.clone());
        if is_active {
            payload.active_file = Some(file_ctx);
        }
    }

    // Build errors
    for err in &ws.build_errors {
        let rel_path = err
            .file
            .strip_prefix(&ws.root_path)
            .unwrap_or(&err.file)
            .to_string_lossy()
            .into_owned();
        payload.build_errors.push(AiBuildErrorContext {
            file: rel_path,
            line: err.line,
            message: err.message.clone(),
            is_warning: err.is_warning,
        });
    }

    // Cursor position
    if let Some(tab) = ws.editor.active()
        && let Some(cr) = tab.last_cursor_range
    {
        payload.cursor_line = Some(cr.primary.rcursor.row + 1);
        payload.cursor_col = Some(cr.primary.rcursor.column + 1);
    }

    // Git context
    payload.git_branch = ws.git_branch.clone();
    for (abs_path, status) in &ws.file_tree.git_statuses {
        let rel = abs_path
            .strip_prefix(&ws.root_path)
            .unwrap_or(abs_path)
            .to_string_lossy()
            .into_owned();
        let code = match status {
            crate::app::ui::git_status::GitVisualStatus::Modified => "M",
            crate::app::ui::git_status::GitVisualStatus::Added => "A",
            crate::app::ui::git_status::GitVisualStatus::Deleted => "D",
            crate::app::ui::git_status::GitVisualStatus::Untracked => "??",
        };
        payload.git_status.push(AiGitFileStatus {
            path: rel,
            status: code.to_string(),
        });
    }

    payload.to_system_message()
}

#[cfg(test)]
mod tests {
    use super::{normalize_prompt_input, validate_provider_config};

    #[test]
    fn normalize_prompt_input_trims_and_keeps_slash_commands() {
        assert_eq!(normalize_prompt_input("   /help   "), Some("/help".to_string()));
        assert_eq!(normalize_prompt_input("   hello world  "), Some("hello world".to_string()));
    }

    #[test]
    fn normalize_prompt_input_rejects_whitespace_only() {
        assert_eq!(normalize_prompt_input("   \n\t "), None);
    }

    #[test]
    fn provider_config_requires_non_empty_model() {
        assert!(validate_provider_config("llama3.1").is_ok());
        assert!(validate_provider_config("   ").is_err());
    }
}
