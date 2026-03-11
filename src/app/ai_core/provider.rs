use serde::{Deserialize, Serialize};
use std::sync::mpsc;

use super::types::{AiMessage, AiToolDeclaration};

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
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default = "default_top_p")]
    pub top_p: f64,
    #[serde(default = "default_top_k")]
    pub top_k: u64,
    #[serde(default = "default_repeat_penalty")]
    pub repeat_penalty: f64,
    #[serde(default)]
    pub seed: i64,
}

fn default_top_p() -> f64 {
    0.9
}
fn default_top_k() -> u64 {
    40
}
fn default_repeat_penalty() -> f64 {
    1.1
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
    /// Model thinking/reasoning block (e.g. cogito `<thinking>` tags).
    Thinking(String),
    /// Replace the entire streaming buffer with this cleaned content.
    /// Used when post-processing strips special tokens from accumulated output.
    ContentReplace(String),
}

/// Trait abstraction for AI providers (Ollama, Claude, Gemini, …).
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn available_models(&self) -> Result<Vec<String>, String>;
    fn capabilities(&self) -> ProviderCapabilities;
    fn config(&self) -> &ProviderConfig;
    fn send_chat(
        &self,
        messages: &[AiMessage],
        config: &ProviderConfig,
    ) -> Result<AiMessage, String>;
    fn stream_chat(
        &self,
        messages: Vec<AiMessage>,
        config: ProviderConfig,
        tools: Vec<AiToolDeclaration>,
    ) -> mpsc::Receiver<StreamEvent>;
}
