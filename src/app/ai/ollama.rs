use std::io::BufRead;
use std::sync::mpsc;
use std::time::Duration;

use serde_json::Value;

use super::provider::{AiProvider, ProviderCapabilities, ProviderConfig, StreamEvent};
use super::types::AiMessage;

/// Status returned by the Ollama availability check.
#[derive(Clone, Debug)]
pub enum OllamaStatus {
    Available(Vec<String>),
    Unavailable,
}

/// Native Ollama provider implementing `AiProvider`.
pub struct OllamaProvider {
    config: ProviderConfig,
    agent: ureq::Agent,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String, api_key: Option<String>) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(300))
            .timeout_write(Duration::from_secs(30))
            .build();
        Self {
            config: ProviderConfig {
                base_url,
                model,
                temperature: 0.7,
                num_ctx: 4096,
                api_key,
            },
            agent,
        }
    }

    /// Apply Bearer authorization header if api_key is set.
    fn apply_auth<'a>(&self, req: ureq::Request) -> ureq::Request {
        if let Some(ref key) = self.config.api_key {
            req.set("Authorization", &format!("Bearer {key}"))
        } else {
            req
        }
    }
}

impl AiProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.config.base_url);
        let req = self.apply_auth(self.agent.get(&url));
        req.call().is_ok()
    }

    fn available_models(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/tags", self.config.base_url);
        let req = self.apply_auth(self.agent.get(&url));
        let resp = req
            .call()
            .map_err(|e| format!("Failed to reach Ollama: {e}"))?;
        let body = resp
            .into_string()
            .map_err(|e| format!("Failed to read response: {e}"))?;
        parse_tags_response(&body)
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_streaming: true,
            supports_tools: false,
        }
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }

    fn send_chat(
        &self,
        messages: &[AiMessage],
        config: &ProviderConfig,
    ) -> Result<AiMessage, String> {
        let url = format!("{}/api/chat", config.base_url);
        let msgs: Vec<Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": config.model,
            "messages": msgs,
            "stream": false,
            "options": {
                "temperature": config.temperature,
                "num_ctx": config.num_ctx,
            }
        });

        let req = self.apply_auth(self.agent.post(&url));
        let resp = req
            .send_json(&body)
            .map_err(|e| format!("Ollama request failed: {e}"))?;
        let text = resp
            .into_string()
            .map_err(|e| format!("Failed to read response: {e}"))?;
        let parsed: Value =
            serde_json::from_str(&text).map_err(|e| format!("Invalid JSON: {e}"))?;

        let content = parsed["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(AiMessage {
            role: "assistant".to_string(),
            content,
            monologue: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            tool_call_name: None,
            tool_call_id: None,
            tool_result_for_id: None,
            tool_is_error: false,
            tool_call_arguments: None,
        })
    }

    fn stream_chat(
        &self,
        messages: Vec<AiMessage>,
        config: ProviderConfig,
    ) -> mpsc::Receiver<StreamEvent> {
        let (tx, rx) = mpsc::channel();
        let agent = self.agent.clone();

        std::thread::spawn(move || {
            let url = format!("{}/api/chat", config.base_url);
            let msgs: Vec<Value> = messages
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "role": m.role,
                        "content": m.content,
                    })
                })
                .collect();

            let body = serde_json::json!({
                "model": config.model,
                "messages": msgs,
                "stream": true,
                "options": {
                    "temperature": config.temperature,
                    "num_ctx": config.num_ctx,
                }
            });

            let req = if let Some(ref key) = config.api_key {
                agent.post(&url).set("Authorization", &format!("Bearer {key}"))
            } else {
                agent.post(&url)
            };
            let resp = match req.send_json(&body) {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(StreamEvent::Error(format!("Ollama request failed: {e}")));
                    return;
                }
            };

            let reader = std::io::BufReader::new(resp.into_reader());
            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error(format!("Read error: {e}")));
                        return;
                    }
                };

                if let Some(event) = parse_ndjson_line(&line) {
                    if tx.send(event).is_err() {
                        return; // receiver dropped
                    }
                }
            }
        });

        rx
    }
}

/// Parse Ollama `/api/tags` JSON response into a list of model names.
/// Strips `:latest` suffix for cleaner display.
pub fn parse_tags_response(json: &str) -> Result<Vec<String>, String> {
    let parsed: Value = serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;
    let models = parsed["models"]
        .as_array()
        .ok_or_else(|| "Missing 'models' array".to_string())?;

    Ok(models
        .iter()
        .filter_map(|m| {
            m["name"].as_str().map(|name| {
                name.strip_suffix(":latest")
                    .unwrap_or(name)
                    .to_string()
            })
        })
        .collect())
}

/// Parse a single NDJSON line from Ollama streaming response.
pub fn parse_ndjson_line(line: &str) -> Option<StreamEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parsed: Value = serde_json::from_str(trimmed).ok()?;

    if let Some(err) = parsed["error"].as_str() {
        return Some(StreamEvent::Error(err.to_string()));
    }

    let done = parsed["done"].as_bool().unwrap_or(false);

    if done {
        let model = parsed["model"].as_str().unwrap_or("").to_string();
        let prompt_tokens = parsed["prompt_eval_count"].as_u64().unwrap_or(0);
        let completion_tokens = parsed["eval_count"].as_u64().unwrap_or(0);
        return Some(StreamEvent::Done {
            model,
            prompt_tokens,
            completion_tokens,
        });
    }

    let content = parsed["message"]["content"].as_str().unwrap_or("");
    if !content.is_empty() {
        Some(StreamEvent::Token(content.to_string()))
    } else {
        None
    }
}

/// Validate an Ollama API URL.
///
/// Returns `Some(cleaned_url)` if the URL is a valid Ollama endpoint,
/// `None` otherwise. Accepts both local URLs with explicit port
/// (e.g. `http://localhost:11434`) and cloud URLs without port
/// (e.g. `https://ollama.example.com`). Strips trailing slash.
pub fn validate_ollama_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parsed = url::Url::parse(trimmed).ok()?;

    // Must be http or https
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return None,
    }

    // Must have a host
    let host = parsed.host_str()?;

    // Build cleaned URL — include port only when explicitly specified
    let mut clean = if let Some(port) = parsed.port() {
        format!("{}://{}:{}", parsed.scheme(), host, port)
    } else {
        format!("{}://{}", parsed.scheme(), host)
    };

    // Preserve path if present (but strip trailing slash)
    let path = parsed.path().trim_end_matches('/');
    if !path.is_empty() && path != "/" {
        clean.push_str(path);
    }

    Some(clean)
}

/// Spawn a background thread to check Ollama availability and return status.
pub fn spawn_ollama_check(base_url: String, api_key: Option<String>) -> mpsc::Receiver<OllamaStatus> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();
        let url = format!("{base_url}/api/tags");
        let req = if let Some(ref key) = api_key {
            agent.get(&url).set("Authorization", &format!("Bearer {key}"))
        } else {
            agent.get(&url)
        };
        let status = match req.call() {
            Ok(resp) => {
                if let Ok(body) = resp.into_string() {
                    match parse_tags_response(&body) {
                        Ok(models) => OllamaStatus::Available(models),
                        Err(_) => OllamaStatus::Unavailable,
                    }
                } else {
                    OllamaStatus::Unavailable
                }
            }
            Err(_) => OllamaStatus::Unavailable,
        };
        let _ = tx.send(status);
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_event_token() {
        let event = StreamEvent::Token("hello".to_string());
        if let StreamEvent::Token(t) = event {
            assert_eq!(t, "hello");
        } else {
            panic!("Expected Token variant");
        }
    }

    #[test]
    fn stream_event_done() {
        let event = StreamEvent::Done {
            model: "llama3".to_string(),
            prompt_tokens: 10,
            completion_tokens: 20,
        };
        if let StreamEvent::Done {
            model,
            prompt_tokens,
            completion_tokens,
        } = event
        {
            assert_eq!(model, "llama3");
            assert_eq!(prompt_tokens, 10);
            assert_eq!(completion_tokens, 20);
        } else {
            panic!("Expected Done variant");
        }
    }

    #[test]
    fn stream_event_error() {
        let event = StreamEvent::Error("something failed".to_string());
        if let StreamEvent::Error(msg) = event {
            assert_eq!(msg, "something failed");
        } else {
            panic!("Expected Error variant");
        }
    }

    #[test]
    fn parse_tags_valid() {
        let json = r#"{"models":[{"name":"llama3:latest","size":123},{"name":"codellama:7b","size":456}]}"#;
        let result = parse_tags_response(json).unwrap();
        assert_eq!(result, vec!["llama3", "codellama:7b"]);
    }

    #[test]
    fn parse_tags_empty_models() {
        let json = r#"{"models":[]}"#;
        let result = parse_tags_response(json).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_tags_invalid_json() {
        let result = parse_tags_response("not json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_ndjson_token() {
        let line = r#"{"model":"llama3","message":{"role":"assistant","content":"Hello"},"done":false}"#;
        let event = parse_ndjson_line(line).unwrap();
        if let StreamEvent::Token(t) = event {
            assert_eq!(t, "Hello");
        } else {
            panic!("Expected Token, got {:?}", event);
        }
    }

    #[test]
    fn parse_ndjson_done() {
        let line = r#"{"model":"llama3","message":{"role":"assistant","content":""},"done":true,"prompt_eval_count":50,"eval_count":100}"#;
        let event = parse_ndjson_line(line).unwrap();
        if let StreamEvent::Done {
            model,
            prompt_tokens,
            completion_tokens,
        } = event
        {
            assert_eq!(model, "llama3");
            assert_eq!(prompt_tokens, 50);
            assert_eq!(completion_tokens, 100);
        } else {
            panic!("Expected Done, got {:?}", event);
        }
    }

    #[test]
    fn parse_ndjson_empty_line() {
        assert!(parse_ndjson_line("").is_none());
        assert!(parse_ndjson_line("   ").is_none());
    }

    #[test]
    fn ollama_provider_new_and_name() {
        let provider = OllamaProvider::new("http://localhost:11434".to_string(), "llama3".to_string(), None);
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn ollama_provider_capabilities() {
        let provider = OllamaProvider::new("http://localhost:11434".to_string(), "llama3".to_string(), None);
        let caps = provider.capabilities();
        assert!(caps.supports_streaming);
        assert!(!caps.supports_tools);
    }

    #[test]
    fn ollama_provider_api_key_stored() {
        let provider = OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "llama3".to_string(),
            Some("sk-test-key".to_string()),
        );
        assert_eq!(provider.config().api_key, Some("sk-test-key".to_string()));
    }

    #[test]
    fn validate_ollama_url_localhost() {
        assert_eq!(
            validate_ollama_url("http://localhost:11434"),
            Some("http://localhost:11434".to_string())
        );
    }

    #[test]
    fn validate_ollama_url_ip_with_port() {
        assert_eq!(
            validate_ollama_url("http://192.168.1.100:11434"),
            Some("http://192.168.1.100:11434".to_string())
        );
    }

    #[test]
    fn validate_ollama_url_accepts_https_no_port() {
        assert_eq!(
            validate_ollama_url("https://ollama.com"),
            Some("https://ollama.com".to_string())
        );
    }

    #[test]
    fn validate_ollama_url_https_with_path() {
        assert_eq!(
            validate_ollama_url("https://my-proxy.com/ollama/"),
            Some("https://my-proxy.com/ollama".to_string())
        );
    }

    #[test]
    fn validate_ollama_url_http_no_port() {
        assert_eq!(
            validate_ollama_url("http://ollama.local"),
            Some("http://ollama.local".to_string())
        );
    }

    #[test]
    fn validate_ollama_url_rejects_empty() {
        assert_eq!(validate_ollama_url(""), None);
    }

    #[test]
    fn validate_ollama_url_rejects_garbage() {
        assert_eq!(validate_ollama_url("not-a-url"), None);
    }

    #[test]
    fn validate_ollama_url_strips_trailing_slash() {
        assert_eq!(
            validate_ollama_url("http://localhost:11434/"),
            Some("http://localhost:11434".to_string())
        );
    }

    #[test]
    fn validate_ollama_url_custom_host_with_port() {
        assert_eq!(
            validate_ollama_url("http://my-server:11434"),
            Some("http://my-server:11434".to_string())
        );
    }

    #[test]
    fn validate_ollama_url_rejects_ftp_scheme() {
        assert_eq!(validate_ollama_url("ftp://server.com:11434"), None);
    }
}
