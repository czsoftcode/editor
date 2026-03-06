use serde::{Deserialize, Serialize};
use std::sync::mpsc;

use super::types::AiMessage;

/// Capabilities advertised by an AI provider.
#[derive(Clone, Debug)]
pub struct ProviderCapabilities {
    pub supports_streaming: bool,
    pub supports_tools: bool,
}

/// Configuration for an AI provider connection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub base_url: String,
    pub model: String,
    pub temperature: f64,
    pub num_ctx: u64,
}

/// Events emitted during a streaming chat response.
#[derive(Clone, Debug)]
pub enum StreamEvent {
    Token(String),
    Done {
        model: String,
        prompt_tokens: u64,
        completion_tokens: u64,
    },
    Error(String),
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
}

/// Trait abstraction for AI providers (Ollama, Claude, Gemini, …).
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn available_models(&self) -> Result<Vec<String>, String>;
    fn capabilities(&self) -> ProviderCapabilities;
    fn config(&self) -> &ProviderConfig;
    fn send_chat(&self, messages: &[AiMessage], config: &ProviderConfig) -> Result<AiMessage, String>;
    fn stream_chat(&self, messages: Vec<AiMessage>, config: ProviderConfig) -> mpsc::Receiver<StreamEvent>;
}
