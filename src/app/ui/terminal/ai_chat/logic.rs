use crate::app::ai::ollama::OllamaProvider;
use crate::app::ai::provider::{AiProvider, ProviderConfig};
use crate::app::ai::state::OllamaConnectionStatus;
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

    // Build messages from conversation history
    let mut messages: Vec<AiMessage> = Vec::new();

    // System prompt
    if !ws.ai.chat.system_prompt.is_empty() {
        messages.push(AiMessage {
            role: "system".to_string(),
            content: ws.ai.chat.system_prompt.clone(),
            monologue: Vec::new(),
            timestamp: 0,
            tool_call_name: None,
            tool_call_id: None,
            tool_result_for_id: None,
            tool_is_error: false,
        });
    }

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
    ws.ai.chat.stream_rx = Some(provider.stream_chat(messages, config));
}
