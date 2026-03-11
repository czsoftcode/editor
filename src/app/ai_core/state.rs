use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};

use super::ollama::ModelInfo;
use super::{AiExpertiseRole, AiReasoningDepth, OllamaStatus};

/// Connection status of the Ollama provider.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum OllamaConnectionStatus {
    #[default]
    Checking,
    Connected,
    Disconnected,
}

/// Chat-specific state: prompt, history, conversation, tokens.
pub struct ChatState {
    pub prompt: String,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub monologue: Vec<String>,
    pub conversation: Vec<(String, String)>,
    pub system_prompt: String,
    pub response: Option<String>,
    pub loading: bool,
    pub focus_requested: bool,
    pub last_payload: String,
    pub in_tokens: u32,
    pub out_tokens: u32,
    /// Receiver for streaming tokens from the provider.
    pub stream_rx: Option<mpsc::Receiver<super::provider::StreamEvent>>,
    /// Buffer accumulating streamed tokens for the current response.
    pub streaming_buffer: String,
    /// Whether the chat view should auto-scroll to the latest content.
    pub auto_scroll: bool,
    /// Thinking/reasoning text per conversation entry (parallel to `conversation` vec).
    pub thinking_history: Vec<Option<String>>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            history: Vec::new(),
            history_index: None,
            monologue: Vec::new(),
            conversation: Vec::new(),
            system_prompt: String::new(),
            response: None,
            loading: false,
            focus_requested: true,
            last_payload: String::new(),
            in_tokens: 0,
            out_tokens: 0,
            stream_rx: None,
            streaming_buffer: String::new(),
            auto_scroll: true,
            thinking_history: Vec::new(),
        }
    }
}

/// Ollama provider connection state.
pub struct OllamaState {
    pub status: OllamaConnectionStatus,
    pub models: Vec<String>,
    pub selected_model: String,
    pub check_rx: Option<mpsc::Receiver<OllamaStatus>>,
    pub last_check: std::time::Instant,
    pub base_url: String,
    pub api_key: Option<String>,
    /// Filter text for model picker combobox.
    pub model_filter: String,
    /// Cached info for the currently selected model.
    pub model_info: Option<ModelInfo>,
    /// Receiver for async model info fetch.
    pub model_info_rx: Option<mpsc::Receiver<Result<ModelInfo, String>>>,
    /// Model name for which model_info was fetched (to detect changes).
    pub model_info_for: String,
}

impl Default for OllamaState {
    fn default() -> Self {
        Self {
            status: OllamaConnectionStatus::default(),
            models: Vec::new(),
            selected_model: String::new(),
            check_rx: None,
            last_check: std::time::Instant::now(),
            base_url: String::new(),
            api_key: None,
            model_filter: String::new(),
            model_info: None,
            model_info_rx: None,
            model_info_for: String::new(),
        }
    }
}

/// AI settings: expertise, reasoning depth, language, provider selection.
pub struct AiSettings {
    pub expertise: AiExpertiseRole,
    pub reasoning_depth: AiReasoningDepth,
    pub font_scale: u32,
    pub language: String,
    pub selected_provider: String,
    pub show_settings: bool,
    pub temperature: f64,
    pub num_ctx: u64,
    pub top_p: f64,
    pub top_k: u64,
    pub repeat_penalty: f64,
    pub seed: i64,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            expertise: AiExpertiseRole::default(),
            reasoning_depth: AiReasoningDepth::default(),
            font_scale: 100,
            language: String::new(),
            selected_provider: "gemini".to_string(),
            show_settings: false,
            temperature: 0.7,
            num_ctx: 4096,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            seed: 0,
        }
    }
}

/// Top-level AI state aggregating all AI sub-states.
pub struct AiState {
    pub chat: ChatState,
    pub ollama: OllamaState,
    pub settings: AiSettings,
    pub inspector_open: bool,
    pub cancellation_token: Arc<AtomicBool>,
    pub markdown_cache: egui_commonmark::CommonMarkCache,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            chat: ChatState::default(),
            ollama: OllamaState::default(),
            settings: AiSettings::default(),
            inspector_open: false,
            cancellation_token: Arc::new(AtomicBool::new(false)),
            markdown_cache: egui_commonmark::CommonMarkCache::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_state_default_has_streaming_fields() {
        let cs = ChatState::default();
        assert!(
            cs.streaming_buffer.is_empty(),
            "streaming_buffer should be empty by default"
        );
        assert!(cs.auto_scroll, "auto_scroll should be true by default");
        assert!(
            cs.stream_rx.is_none(),
            "stream_rx should be None by default"
        );
    }

    #[test]
    fn chat_state_default_has_retry_state() {
        let cs = ChatState::default();
        assert!(cs.retry_prompt.is_none());
        assert!(!cs.retry_available);
    }
}
