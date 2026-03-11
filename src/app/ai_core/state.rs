use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};

use super::ollama::{ModelInfo, spawn_model_info_fetch};
use super::{AiExpertiseRole, AiReasoningDepth, OllamaStatus};
use crate::app::ai_core::spawn_ollama_check;

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
    /// Last valid prompt that can be resent via explicit Retry action.
    pub retry_prompt: Option<String>,
    /// Whether Retry action should be shown after temporary runtime failure.
    pub retry_available: bool,
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
            retry_prompt: None,
            retry_available: false,
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

impl AiState {
    pub fn provider_is_connected(&self) -> bool {
        self.ollama.status == OllamaConnectionStatus::Connected
    }

    pub fn provider_connection_parts(&self) -> (String, String, Option<String>) {
        (
            self.ollama.base_url.clone(),
            self.ollama.selected_model.clone(),
            self.ollama.api_key.clone(),
        )
    }

    pub fn provider_display_name(&self) -> &str {
        if self.ollama.selected_model.is_empty() {
            "provider"
        } else {
            &self.ollama.selected_model
        }
    }

    pub fn provider_model(&self) -> &str {
        &self.ollama.selected_model
    }

    pub fn provider_models(&self) -> &[String] {
        &self.ollama.models
    }

    pub fn set_provider_model(&mut self, model: String) {
        self.ollama.selected_model = model;
    }

    pub fn sync_provider_settings(&mut self, settings: &crate::settings::Settings) {
        let url_changed = self.ollama.base_url != settings.ollama_base_url;
        if url_changed {
            self.ollama.base_url = settings.ollama_base_url.clone();
            self.ollama.last_check =
                std::time::Instant::now() - std::time::Duration::from_secs(999);
            self.ollama.status = OllamaConnectionStatus::Checking;
        }

        self.ollama.api_key = if settings.ollama_api_key.is_empty() {
            None
        } else {
            Some(settings.ollama_api_key.clone())
        };

        if !settings.ai_default_model.is_empty() && self.ollama.selected_model.is_empty() {
            self.ollama.selected_model = settings.ai_default_model.clone();
        }

        self.settings.expertise = settings.ai_expertise;
        self.settings.reasoning_depth = settings.ai_reasoning_depth;
        self.settings.top_p = settings.ollama_top_p;
        self.settings.top_k = settings.ollama_top_k;
        self.settings.repeat_penalty = settings.ollama_repeat_penalty;
        self.settings.seed = settings.ollama_seed;
    }

    pub fn poll_provider_connection(&mut self) {
        if let Some(rx) = &self.ollama.check_rx
            && let Ok(status) = rx.try_recv()
        {
            match status {
                OllamaStatus::Available(models) => {
                    self.ollama.status = OllamaConnectionStatus::Connected;
                    if self.ollama.selected_model.is_empty()
                        && let Some(first) = models.first()
                    {
                        self.ollama.selected_model = first.clone();
                    }
                    self.ollama.models = models;
                }
                OllamaStatus::Unavailable => {
                    self.ollama.status = OllamaConnectionStatus::Disconnected;
                    self.ollama.models.clear();
                }
            }
            self.ollama.check_rx = None;
            self.ollama.last_check = std::time::Instant::now();
        }

        if self.ollama.last_check.elapsed().as_secs() >= crate::config::OLLAMA_CHECK_INTERVAL_SECS
            && self.ollama.check_rx.is_none()
            && !self.chat.loading
        {
            self.ollama.check_rx = Some(spawn_ollama_check(
                self.ollama.base_url.clone(),
                self.ollama.api_key.clone(),
            ));
        }
    }

    pub fn poll_provider_model_info(&mut self) {
        if !self.ollama.selected_model.is_empty()
            && self.ollama.selected_model != self.ollama.model_info_for
            && self.ollama.model_info_rx.is_none()
        {
            self.ollama.model_info_for = self.ollama.selected_model.clone();
            self.ollama.model_info = None;
            self.ollama.model_info_rx = Some(spawn_model_info_fetch(
                self.ollama.base_url.clone(),
                self.ollama.selected_model.clone(),
                self.ollama.api_key.clone(),
            ));
        }

        if let Some(rx) = &self.ollama.model_info_rx
            && let Ok(result) = rx.try_recv()
        {
            self.ollama.model_info = result.ok();
            self.ollama.model_info_rx = None;
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
