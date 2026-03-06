use crate::app::ai::ollama::OllamaProvider;
use crate::app::ai::provider::{AiProvider, ProviderConfig};
use crate::app::ai::state::OllamaConnectionStatus;
use crate::app::ai::tools::get_standard_tools;
use crate::app::ai::types::AiMessage;
use crate::app::types::Toast;
use crate::app::ui::workspace::state::WorkspaceState;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub fn send_query_to_agent(ws: &mut WorkspaceState) {
    if ws.ai.chat.prompt.trim().is_empty() {
        return;
    }

    if ws.ai.ollama.status != OllamaConnectionStatus::Connected {
        ws.toasts
            .push(Toast::error("Ollama is not connected.".to_string()));
        return;
    }

    // Initialize tool executor lazily on first AI chat
    if ws.tool_executor.is_none() {
        let root = ws.root_path.clone();
        ws.tool_executor = Some(crate::app::ai::executor::ToolExecutor::new(root, None, None));
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
    ws.ai.chat.conversation.push((prompt.clone(), String::new()));

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
    let provider = OllamaProvider::new(
        ws.ai.ollama.base_url.clone(),
        ws.ai.ollama.selected_model.clone(),
        ws.ai.ollama.api_key.clone(),
    );

    let config = ProviderConfig {
        base_url: ws.ai.ollama.base_url.clone(),
        model: ws.ai.ollama.selected_model.clone(),
        temperature: 0.7,
        num_ctx: 4096,
        api_key: ws.ai.ollama.api_key.clone(),
    };

    // stream_chat() spawns its own thread and returns Receiver immediately
    let tools = get_standard_tools();
    ws.ai.chat.stream_rx = Some(provider.stream_chat(messages, config, tools));
}

/// Builds an editor context string from workspace state for injection into system message.
fn build_editor_context(ws: &WorkspaceState) -> String {
    use crate::app::ai::types::{AiContextPayload, AiFileContext, AiBuildErrorContext, AiGitFileStatus};

    let mut payload = AiContextPayload::default();

    // Project name and root
    payload.project_name = ws.root_path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    payload.project_root = ".".to_string();

    // Open files
    for (i, tab) in ws.editor.tabs.iter().enumerate() {
        let rel_path = tab.path.strip_prefix(&ws.root_path)
            .unwrap_or(&tab.path)
            .to_string_lossy().into_owned();
        let is_active = Some(i) == ws.editor.active_tab;
        let file_ctx = AiFileContext {
            path: rel_path,
            content: if is_active { Some(tab.content.clone()) } else { None },
            is_active,
        };
        payload.open_files.push(file_ctx.clone());
        if is_active {
            payload.active_file = Some(file_ctx);
        }
    }

    // Build errors
    for err in &ws.build_errors {
        let rel_path = err.file.strip_prefix(&ws.root_path)
            .unwrap_or(&err.file)
            .to_string_lossy().into_owned();
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
        let rel = abs_path.strip_prefix(&ws.root_path)
            .unwrap_or(abs_path)
            .to_string_lossy().into_owned();
        let code = match status {
            crate::app::ui::git_status::GitVisualStatus::Modified => "M",
            crate::app::ui::git_status::GitVisualStatus::Added => "A",
            crate::app::ui::git_status::GitVisualStatus::Deleted => "D",
            crate::app::ui::git_status::GitVisualStatus::Untracked => "??",
        };
        payload.git_status.push(AiGitFileStatus { path: rel, status: code.to_string() });
    }

    payload.to_system_message()
}
