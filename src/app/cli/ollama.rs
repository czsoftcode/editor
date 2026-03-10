use std::io::BufRead;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use serde_json::Value;

use super::provider::{AiProvider, ProviderCapabilities, ProviderConfig, StreamEvent};
use super::types::{AiMessage, AiToolDeclaration};

/// Global counter for generating unique tool call IDs.
static TOOL_CALL_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Builds the Ollama "options" JSON object from a ProviderConfig.
fn build_options(config: &ProviderConfig) -> Value {
    let mut opts = serde_json::json!({
        "temperature": config.temperature,
        "num_ctx": config.num_ctx,
        "top_p": config.top_p,
        "top_k": config.top_k,
        "repeat_penalty": config.repeat_penalty,
    });
    if config.seed != 0 {
        opts["seed"] = serde_json::json!(config.seed);
    }
    opts
}

/// Model details returned by `/api/show`.
#[derive(Clone, Debug, Default)]
pub struct ModelInfo {
    pub family: String,
    pub parameter_size: String,
    pub quantization_level: String,
    pub parameters: String,
    pub context_length: Option<u64>,
}

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
                top_p: 0.9,
                top_k: 40,
                repeat_penalty: 1.1,
                seed: 0,
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
            supports_tools: true,
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
        let msgs: Vec<Value> = messages.iter().map(|m| serialize_message(m)).collect();

        let body = serde_json::json!({
            "model": config.model,
            "messages": msgs,
            "stream": false,
            "options": build_options(&config)
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
        tools: Vec<AiToolDeclaration>,
    ) -> mpsc::Receiver<StreamEvent> {
        let (tx, rx) = mpsc::channel();
        let agent = self.agent.clone();

        std::thread::spawn(move || {
            let url = format!("{}/api/chat", config.base_url);
            let msgs: Vec<Value> = messages.iter().map(|m| serialize_message(m)).collect();

            let has_tools = !tools.is_empty();

            let mut body = serde_json::json!({
                "model": config.model,
                "messages": msgs,
                "stream": !has_tools,
                "options": {
                    "temperature": config.temperature,
                    "num_ctx": config.num_ctx,
                }
            });

            // Add tools array when tools are present
            if has_tools {
                let tools_json: Vec<Value> = tools
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "type": "function",
                            "function": {
                                "name": t.name,
                                "description": t.description,
                                "parameters": t.parameters,
                            }
                        })
                    })
                    .collect();
                body["tools"] = Value::Array(tools_json);
            }

            let make_req = |agent: &ureq::Agent, url: &str, key: &Option<String>| {
                if let Some(k) = key {
                    agent.post(url).set("Authorization", &format!("Bearer {k}"))
                } else {
                    agent.post(url)
                }
            };

            let req = make_req(&agent, &url, &config.api_key);
            let (resp, tools_active) = match req.send_json(&body) {
                Ok(r) => (r, has_tools),
                Err(e) if has_tools => {
                    // Model may not support tools API — fallback to streaming
                    // and inject tool descriptions into a system message
                    let err_detail = match e {
                        ureq::Error::Status(code, resp) => {
                            let body = resp.into_string().unwrap_or_default();
                            format!("status={code} body={body}")
                        }
                        other => format!("{other}"),
                    };
                    eprintln!(
                        "[Ollama] Tools request failed for model '{}': {}",
                        config.model, err_detail
                    );
                    eprintln!("[Ollama] Falling back to streaming with text-based tools");

                    // Build text description of tools for the system message
                    let tools_text = tools_to_system_prompt_text(&tools);
                    let mut fallback_msgs = msgs.clone();
                    // Prepend tools description as system message
                    fallback_msgs.insert(
                        0,
                        serde_json::json!({
                            "role": "system",
                            "content": tools_text,
                        }),
                    );

                    let mut fallback_body = serde_json::json!({
                        "model": config.model,
                        "messages": fallback_msgs,
                        "stream": true,
                        "options": {
                            "temperature": config.temperature,
                            "num_ctx": config.num_ctx,
                        }
                    });
                    fallback_body.as_object_mut().map(|o| o.remove("tools"));
                    let fallback_req = make_req(&agent, &url, &config.api_key);
                    match fallback_req.send_json(&fallback_body) {
                        Ok(r) => {
                            eprintln!(
                                "[Ollama] Fallback streaming request succeeded for model '{}'",
                                config.model
                            );
                            (r, false)
                        }
                        Err(e2) => {
                            let _ =
                                tx.send(StreamEvent::Error(format!("Ollama request failed: {e2}")));
                            return;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(StreamEvent::Error(format!("Ollama request failed: {e}")));
                    return;
                }
            };

            if tools_active {
                // Non-streaming: read full response, parse tool_calls or content
                let text = match resp.into_string() {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error(format!("Read error: {e}")));
                        return;
                    }
                };
                let parsed: Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error(format!("Invalid JSON: {e}")));
                        return;
                    }
                };

                if let Some(err) = parsed["error"].as_str() {
                    let _ = tx.send(StreamEvent::Error(err.to_string()));
                    return;
                }

                // Check for structured tool_calls in JSON response
                let has_structured_calls = parsed["message"]["tool_calls"]
                    .as_array()
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);

                if has_structured_calls {
                    for tc in parsed["message"]["tool_calls"].as_array().unwrap() {
                        let name = tc["function"]["name"]
                            .as_str()
                            .unwrap_or("unknown")
                            .to_string();
                        let arguments = tc["function"]["arguments"].clone();
                        let id = format!(
                            "tc_{}_{}",
                            name,
                            TOOL_CALL_COUNTER.fetch_add(1, Ordering::Relaxed)
                        );
                        let _ = tx.send(StreamEvent::ToolCall {
                            id,
                            name,
                            arguments,
                        });
                    }
                }

                // Check content for raw tool-call markers
                let content = parsed["message"]["content"].as_str().unwrap_or("");
                let (thinking, after_think) = strip_thinking_tags(content);
                let (raw_calls, clean) = parse_raw_tool_calls_from_content(&after_think);
                let has_raw_calls = !raw_calls.is_empty();

                if let Some(thought) = thinking {
                    let _ = tx.send(StreamEvent::Thinking(thought));
                }

                // Emit content text even when tool_calls are present
                // (model may explain what it's doing before calling a tool)
                if !clean.is_empty() && (has_structured_calls || has_raw_calls) {
                    let _ = tx.send(StreamEvent::Token(clean.clone()));
                }

                for rc in &raw_calls {
                    let id = format!(
                        "tc_{}_{}",
                        rc.name,
                        TOOL_CALL_COUNTER.fetch_add(1, Ordering::Relaxed)
                    );
                    let _ = tx.send(StreamEvent::ToolCall {
                        id,
                        name: rc.name.clone(),
                        arguments: rc.arguments.clone(),
                    });
                }

                // If model returned content but NO tool calls at all, the model
                // silently ignores the tools API. Retry with text-based tools.
                if !has_structured_calls && !has_raw_calls && !content.is_empty() {
                    eprintln!(
                        "[Ollama] Model '{}' ignored tools API — retrying with text-based tools in system prompt",
                        config.model
                    );

                    let tools_text = tools_to_system_prompt_text(&tools);
                    let mut retry_msgs = msgs.clone();
                    retry_msgs.insert(
                        0,
                        serde_json::json!({
                            "role": "system",
                            "content": tools_text,
                        }),
                    );

                    let retry_body = serde_json::json!({
                        "model": config.model,
                        "messages": retry_msgs,
                        "stream": false,
                        "options": {
                            "temperature": config.temperature,
                            "num_ctx": config.num_ctx,
                        }
                    });

                    let retry_req = make_req(&agent, &url, &config.api_key);
                    match retry_req.send_json(&retry_body) {
                        Ok(retry_resp) => {
                            if let Ok(retry_text) = retry_resp.into_string() {
                                if let Ok(retry_parsed) = serde_json::from_str::<Value>(&retry_text)
                                {
                                    let retry_content =
                                        retry_parsed["message"]["content"].as_str().unwrap_or("");
                                    if !retry_content.is_empty() {
                                        let (rt, ra) = strip_thinking_tags(retry_content);
                                        let (retry_calls, retry_clean) =
                                            parse_raw_tool_calls_from_content(&ra);

                                        if let Some(thought) = rt {
                                            let _ = tx.send(StreamEvent::Thinking(thought));
                                        }
                                        for rc in &retry_calls {
                                            let id = format!(
                                                "tc_{}_{}",
                                                rc.name,
                                                TOOL_CALL_COUNTER.fetch_add(1, Ordering::Relaxed)
                                            );
                                            let _ = tx.send(StreamEvent::ToolCall {
                                                id,
                                                name: rc.name.clone(),
                                                arguments: rc.arguments.clone(),
                                            });
                                        }
                                        if !retry_clean.is_empty() && !retry_calls.is_empty() {
                                            let _ = tx.send(StreamEvent::Token(retry_clean));
                                        } else if retry_calls.is_empty() {
                                            // Still no tool calls — emit original content
                                            if !clean.is_empty() {
                                                let _ = tx.send(StreamEvent::Token(clean));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // Retry failed — emit original content
                            if !clean.is_empty() {
                                let _ = tx.send(StreamEvent::Token(clean));
                            }
                        }
                    }
                } else if !clean.is_empty() {
                    let _ = tx.send(StreamEvent::Token(clean));
                }

                // Emit Done
                let model = parsed["model"].as_str().unwrap_or("").to_string();
                let prompt_tokens = parsed["prompt_eval_count"].as_u64().unwrap_or(0);
                let completion_tokens = parsed["eval_count"].as_u64().unwrap_or(0);
                let _ = tx.send(StreamEvent::Done {
                    model,
                    prompt_tokens,
                    completion_tokens,
                });
            } else {
                // Streaming: accumulate all content, then post-process for
                // <thinking>, raw tool-call markers, and <function> tags.
                let reader = std::io::BufReader::new(resp.into_reader());
                let mut full_content = String::new();
                let mut done_event: Option<StreamEvent> = None;

                for line in reader.lines() {
                    let line = match line {
                        Ok(l) => l,
                        Err(e) => {
                            let _ = tx.send(StreamEvent::Error(format!("Read error: {e}")));
                            return;
                        }
                    };

                    if let Some(event) = parse_ndjson_line(&line) {
                        match event {
                            StreamEvent::Token(text) => {
                                // Stream token to UI immediately for responsiveness
                                let _ = tx.send(StreamEvent::Token(text.clone()));
                                full_content.push_str(&text);
                            }
                            StreamEvent::Done { .. } => {
                                done_event = Some(event);
                            }
                            other => {
                                if tx.send(other).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                }

                // Post-process accumulated content for special tokens
                if !full_content.is_empty() {
                    let has_thinking = full_content.contains("<thinking>");
                    let has_raw_tools = full_content.contains(RAW_CALLS_BEGIN)
                        || full_content.contains("<tool_call>")
                        || full_content.contains("<function>")
                        || full_content.contains("<action>")
                        || full_content.contains("[json ")
                        || full_content.contains("<function_calls>");

                    if has_thinking || has_raw_tools {
                        let (thinking, after_think) = strip_thinking_tags(&full_content);
                        let (raw_calls, clean) = parse_raw_tool_calls_from_content(&after_think);

                        // Send a "replace" signal: clear streamed tokens, re-emit clean
                        // We use a special empty token followed by clean content
                        // to signal the UI to replace the buffer
                        let _ = tx.send(StreamEvent::ContentReplace(clean));

                        if let Some(thought) = thinking {
                            let _ = tx.send(StreamEvent::Thinking(thought));
                        }
                        for rc in &raw_calls {
                            let id = format!(
                                "tc_{}_{}",
                                rc.name,
                                TOOL_CALL_COUNTER.fetch_add(1, Ordering::Relaxed)
                            );
                            let _ = tx.send(StreamEvent::ToolCall {
                                id,
                                name: rc.name.clone(),
                                arguments: rc.arguments.clone(),
                            });
                        }
                    }
                }

                if let Some(done) = done_event {
                    let _ = tx.send(done);
                }
            }
        });

        rx
    }
}

/// Parse Ollama `/api/tags` JSON response into a list of model names.
/// Preserves full tag (e.g. `:latest`, `:cloud`) so users can pick variants.
pub fn parse_tags_response(json: &str) -> Result<Vec<String>, String> {
    let parsed: Value = serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;
    let models = parsed["models"]
        .as_array()
        .ok_or_else(|| "Missing 'models' array".to_string())?;

    Ok(models
        .iter()
        .filter_map(|m| m["name"].as_str().map(|name| name.to_string()))
        .collect())
}

/// Fetch model info from Ollama `/api/show` endpoint.
pub fn fetch_model_info(
    base_url: &str,
    model: &str,
    api_key: &Option<String>,
) -> Result<ModelInfo, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(10))
        .timeout_write(Duration::from_secs(5))
        .build();
    let url = format!("{base_url}/api/show");
    let body = serde_json::json!({
        "model": model,
        "verbose": true,
    });
    let mut req = agent.post(&url);
    if let Some(key) = api_key {
        req = req.set("Authorization", &format!("Bearer {key}"));
    }
    let resp = req
        .send_json(&body)
        .map_err(|e| format!("Failed to fetch model info: {e}"))?;
    let text = resp
        .into_string()
        .map_err(|e| format!("Failed to read response: {e}"))?;
    parse_show_response(&text)
}

/// Parse `/api/show` JSON response into `ModelInfo`.
pub fn parse_show_response(json: &str) -> Result<ModelInfo, String> {
    let parsed: Value = serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;

    let details = &parsed["details"];
    let family = details["family"].as_str().unwrap_or("").to_string();
    let parameter_size = details["parameter_size"].as_str().unwrap_or("").to_string();
    let quantization_level = details["quantization_level"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let parameters = parsed["parameters"].as_str().unwrap_or("").to_string();

    // Try to find context_length from model_info keys (e.g. "llama.context_length")
    let context_length = parsed["model_info"].as_object().and_then(|obj| {
        obj.iter()
            .find(|(k, _)| k.ends_with(".context_length"))
            .and_then(|(_, v)| v.as_u64())
    });

    Ok(ModelInfo {
        family,
        parameter_size,
        quantization_level,
        parameters,
        context_length,
    })
}

/// Spawn a background thread to fetch model info via `/api/show`.
pub fn spawn_model_info_fetch(
    base_url: String,
    model: String,
    api_key: Option<String>,
) -> mpsc::Receiver<Result<ModelInfo, String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = fetch_model_info(&base_url, &model, &api_key);
        let _ = tx.send(result);
    });
    rx
}

/// Parse a single NDJSON line from Ollama streaming response.
/// Detects tool_calls in addition to token/done/error events.
pub fn parse_ndjson_line(line: &str) -> Option<StreamEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parsed: Value = serde_json::from_str(trimmed).ok()?;

    if let Some(err) = parsed["error"].as_str() {
        return Some(StreamEvent::Error(err.to_string()));
    }

    // Check for tool_calls before content (streaming tool support)
    if let Some(tool_calls) = parsed["message"]["tool_calls"].as_array() {
        if let Some(tc) = tool_calls.first() {
            let name = tc["function"]["name"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            let arguments = tc["function"]["arguments"].clone();
            let id = format!(
                "tc_{}_{}",
                name,
                TOOL_CALL_COUNTER.fetch_add(1, Ordering::Relaxed)
            );
            return Some(StreamEvent::ToolCall {
                id,
                name,
                arguments,
            });
        }
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

/// Serialize an AiMessage to Ollama JSON format, including tool call metadata.
fn serialize_message(m: &AiMessage) -> Value {
    // Assistant messages with tool_calls
    if m.role == "assistant" && m.tool_call_name.is_some() {
        let mut msg = serde_json::json!({
            "role": "assistant",
            "content": m.content,
        });
        let name = m.tool_call_name.as_deref().unwrap_or("unknown");
        let arguments = m
            .tool_call_arguments
            .clone()
            .unwrap_or(Value::Object(Default::default()));
        msg["tool_calls"] = serde_json::json!([{
            "function": {
                "name": name,
                "arguments": arguments,
            }
        }]);
        return msg;
    }

    // Tool result messages
    if m.role == "tool" {
        return serde_json::json!({
            "role": "tool",
            "content": m.content,
        });
    }

    // Regular messages (user, system, assistant without tool_calls)
    serde_json::json!({
        "role": m.role,
        "content": m.content,
    })
}

// ---------------------------------------------------------------------------
// Raw tool-call parsing (models like cogito / Qwen emit tool calls as special
// tokens in content instead of structured JSON `tool_calls` field)
// ---------------------------------------------------------------------------

const RAW_CALLS_BEGIN: &str = "<|tool\u{2581}calls\u{2581}begin|>";
const RAW_CALLS_END: &str = "<|tool\u{2581}calls\u{2581}end|>";
const RAW_CALL_BEGIN: &str = "<|tool\u{2581}call\u{2581}begin|>";
const RAW_CALL_END: &str = "<|tool\u{2581}call\u{2581}end|>";
const RAW_TOOL_SEP: &str = "<|tool\u{2581}sep|>";

/// A tool call parsed from raw text tokens.
struct RawToolCall {
    name: String,
    arguments: Value,
}

/// Strip `<thinking>…</thinking>` blocks from content, returning
/// `(thinking_text, cleaned_content)`.
pub fn strip_thinking_tags(content: &str) -> (Option<String>, String) {
    let mut clean = content.to_string();
    let mut thinking = String::new();

    while let Some(start) = clean.find("<thinking>") {
        let end_tag = "</thinking>";
        if let Some(end) = clean[start..].find(end_tag) {
            let block_end = start + end + end_tag.len();
            let inner = &clean[start + "<thinking>".len()..start + end];
            if !thinking.is_empty() {
                thinking.push('\n');
            }
            thinking.push_str(inner.trim());
            clean = format!(
                "{}{}",
                clean[..start].trim_end(),
                clean[block_end..].trim_start()
            );
        } else {
            // Unclosed tag — strip from start to end
            let inner = &clean[start + "<thinking>".len()..];
            thinking.push_str(inner.trim());
            clean = clean[..start].trim_end().to_string();
            break;
        }
    }

    let thought = if thinking.is_empty() {
        None
    } else {
        Some(thinking)
    };
    (thought, clean)
}

/// Parse raw tool-call markers from model content text.
///
/// Supports two formats:
/// 1. `<|tool▁calls▁begin|>…<|tool▁calls▁end|>` (Qwen / cogito native tokens)
/// 2. `<function> name [json {…}](code)` or `<function>name\n{…}\n</function>`
///
/// Returns `(parsed_calls, cleaned_content)` — the cleaned content has the
/// tool-call section removed.
fn parse_raw_tool_calls_from_content(content: &str) -> (Vec<RawToolCall>, String) {
    // --- Strategy 1: <|tool▁calls▁begin|> markers ---
    if let Some(begin_idx) = content.find(RAW_CALLS_BEGIN) {
        let end_idx = content
            .find(RAW_CALLS_END)
            .map(|i| i + RAW_CALLS_END.len())
            .unwrap_or(content.len());

        let tool_section = &content[begin_idx..end_idx];

        let before = content[..begin_idx].trim_end();
        let after = content[end_idx..].trim_start();
        let clean = if before.is_empty() {
            after.to_string()
        } else if after.is_empty() {
            before.to_string()
        } else {
            format!("{before}\n{after}")
        };

        let mut calls = Vec::new();
        let mut pos = 0;
        while let Some(rel) = tool_section[pos..].find(RAW_CALL_BEGIN) {
            let call_start = pos + rel + RAW_CALL_BEGIN.len();
            let call_end = tool_section[call_start..]
                .find(RAW_CALL_END)
                .map(|i| call_start + i)
                .unwrap_or(tool_section.len());

            let call_content = &tool_section[call_start..call_end];

            if let Some(sep_idx) = call_content.find(RAW_TOOL_SEP) {
                let after_sep = call_content[sep_idx + RAW_TOOL_SEP.len()..].trim();
                if let Some(parsed) = parse_single_raw_tool_call(after_sep) {
                    calls.push(parsed);
                }
            }

            pos = call_end + RAW_CALL_END.len();
        }

        return (calls, clean);
    }

    // --- Strategy 2: <function> / <action> tag formats ---
    // Matches: `<function> name [json {…}](code)` (single line)
    //      or: `<function>name\n{…}\n</function>` (multi-line)
    //      or: `<action> [json {"function":"name","params":{…}}](code) </action>`
    let mut calls = Vec::new();
    let mut clean = content.to_string();

    // Parse <function> tags
    while let Some(start) = clean.find("<function>") {
        let tag_end = start + "<function>".len();

        let (block_end, inner) = if let Some(close_rel) = clean[tag_end..].find("</function>") {
            let close_pos = tag_end + close_rel;
            (close_pos + "</function>".len(), &clean[tag_end..close_pos])
        } else {
            let line_end = clean[tag_end..]
                .find('\n')
                .map(|i| tag_end + i)
                .unwrap_or(clean.len());
            (line_end, &clean[tag_end..line_end])
        };

        let inner = inner.trim();
        if let Some(parsed) = parse_function_tag_call(inner) {
            calls.push(parsed);
        }

        let before = clean[..start].trim_end();
        let after = clean[block_end..].trim_start();
        clean = if before.is_empty() {
            after.to_string()
        } else if after.is_empty() {
            before.to_string()
        } else {
            format!("{before}\n{after}")
        };
    }

    // Parse <tool_call> tags — official cogito format:
    // <tool_call>{"name":"func","arguments":{…}}</tool_call>
    while let Some(start) = clean.find("<tool_call>") {
        let tag_end = start + "<tool_call>".len();

        let (block_end, inner) = if let Some(close_rel) = clean[tag_end..].find("</tool_call>") {
            let close_pos = tag_end + close_rel;
            let mut end = close_pos + "</tool_call>".len();
            // Skip optional <|eot_id|> after closing tag
            if clean[end..].starts_with("<|eot_id|>") {
                end += "<|eot_id|>".len();
            }
            (end, &clean[tag_end..close_pos])
        } else {
            // No closing tag — take to end of content
            (clean.len(), &clean[tag_end..])
        };

        let inner = inner.trim();
        if let Some(parsed) = parse_tool_call_tag(inner) {
            calls.push(parsed);
        }

        let before = clean[..start].trim_end();
        let after = clean[block_end..].trim_start();
        clean = if before.is_empty() {
            after.to_string()
        } else if after.is_empty() {
            before.to_string()
        } else {
            format!("{before}\n{after}")
        };
    }

    // Parse <action> tags — some models wrap calls in <action>…</action>
    while let Some(start) = clean.find("<action>") {
        let tag_end = start + "<action>".len();

        let (block_end, inner) = if let Some(close_rel) = clean[tag_end..].find("</action>") {
            let close_pos = tag_end + close_rel;
            (close_pos + "</action>".len(), &clean[tag_end..close_pos])
        } else {
            let line_end = clean[tag_end..]
                .find('\n')
                .map(|i| tag_end + i)
                .unwrap_or(clean.len());
            (line_end, &clean[tag_end..line_end])
        };

        let inner = inner.trim();
        if let Some(parsed) = parse_action_tag_call(inner) {
            calls.push(parsed);
        }

        let before = clean[..start].trim_end();
        let after = clean[block_end..].trim_start();
        clean = if before.is_empty() {
            after.to_string()
        } else if after.is_empty() {
            before.to_string()
        } else {
            format!("{before}\n{after}")
        };
    }

    // Parse bare [json {…}](code) format — cogito v2.1 markdown-link style
    // Pattern: [json {"name":"func","arguments":{…}}](code)
    //      or: [json {"name":"func","arguments":{…}} ](code)
    {
        let mut search_from = 0;
        while let Some(start) = clean[search_from..].find("[json ") {
            let abs_start = search_from + start;
            let bracket_start = abs_start + "[json ".len();

            // Find the closing ](code)
            if let Some(close_rel) = clean[bracket_start..].find("](code)") {
                let json_end = bracket_start + close_rel;
                let block_end = json_end + "](code)".len();
                let inner = clean[bracket_start..json_end].trim();

                if let Some(json_start) = inner.find('{') {
                    let json_val = extract_balanced_json(&inner[json_start..]);
                    // Try {"name":"…","arguments":{…}} format
                    if let Some(func_name) = json_val["name"].as_str() {
                        let arguments = json_val
                            .get("arguments")
                            .cloned()
                            .unwrap_or(Value::Object(Default::default()));
                        calls.push(RawToolCall {
                            name: func_name.to_string(),
                            arguments,
                        });
                    }
                    // Try {"function":"…","params":{…}} format
                    else if let Some(func_name) = json_val["function"].as_str() {
                        let arguments = json_val
                            .get("params")
                            .cloned()
                            .unwrap_or(Value::Object(Default::default()));
                        calls.push(RawToolCall {
                            name: func_name.to_string(),
                            arguments,
                        });
                    }
                }

                let before = clean[..abs_start].trim_end();
                let after = clean[block_end..].trim_start();
                clean = if before.is_empty() {
                    after.to_string()
                } else if after.is_empty() {
                    before.to_string()
                } else {
                    format!("{before}\n{after}")
                };
                // Don't advance search_from — string shifted
            } else {
                search_from = bracket_start;
            }
        }
    }

    // Parse XML <function_calls><invoke name="…"><arg>val</arg></invoke></function_calls>
    // (Claude-style XML that some models mimic)
    while let Some(start) = clean.find("<function_calls>") {
        let tag_end = start + "<function_calls>".len();

        let (block_end, inner) = if let Some(close_rel) = clean[tag_end..].find("</function_calls>")
        {
            let close_pos = tag_end + close_rel;
            (
                close_pos + "</function_calls>".len(),
                &clean[tag_end..close_pos],
            )
        } else {
            (clean.len(), &clean[tag_end..])
        };

        // Parse <invoke name="tool_name"> ... </invoke> blocks
        let inner_owned = inner.to_string();
        let mut invoke_pos = 0;
        while let Some(inv_start) = inner_owned[invoke_pos..].find("<invoke") {
            let abs_inv = invoke_pos + inv_start;
            let inv_end = inner_owned[abs_inv..]
                .find("</invoke>")
                .map(|i| abs_inv + i + "</invoke>".len())
                .unwrap_or(inner_owned.len());

            let invoke_block = &inner_owned[abs_inv..inv_end];
            if let Some(parsed) = parse_xml_invoke_call(invoke_block) {
                calls.push(parsed);
            }

            invoke_pos = inv_end;
        }

        let before = clean[..start].trim_end();
        let after = clean[block_end..].trim_start();
        clean = if before.is_empty() {
            after.to_string()
        } else if after.is_empty() {
            before.to_string()
        } else {
            format!("{before}\n{after}")
        };
    }

    if !calls.is_empty() {
        return (calls, clean);
    }

    (Vec::new(), content.to_string())
}

/// Parse an XML `<invoke name="tool">` block into a tool call.
///
/// Format: `<invoke name="read_project_file"><path>src/main.rs</path></invoke>`
/// Extracts tag name as function name, child XML tags as arguments.
fn parse_xml_invoke_call(block: &str) -> Option<RawToolCall> {
    // Extract function name from name="…" attribute
    let name_start = block.find("name=\"")? + "name=\"".len();
    let name_end = block[name_start..].find('"')? + name_start;
    let func_name = &block[name_start..name_end];

    if func_name.is_empty() {
        return None;
    }

    // Find the content after the opening tag's >
    let content_start = block[name_end..].find('>')? + name_end + 1;
    let content_end = block.rfind("</invoke").unwrap_or(block.len());
    let content = &block[content_start..content_end];

    // Parse XML child tags as arguments: <key>value</key>
    let mut args = serde_json::Map::new();
    let mut pos = 0;
    while let Some(tag_start) = content[pos..].find('<') {
        let abs_start = pos + tag_start;
        if content[abs_start..].starts_with("</") {
            break; // closing tag
        }
        let tag_name_end = content[abs_start + 1..]
            .find('>')
            .map(|i| abs_start + 1 + i)?;
        let tag_name = content[abs_start + 1..tag_name_end].trim();

        let close_tag = format!("</{tag_name}>");
        if let Some(val_end) = content[tag_name_end + 1..].find(&close_tag) {
            let val = content[tag_name_end + 1..tag_name_end + 1 + val_end].trim();
            args.insert(tag_name.to_string(), Value::String(val.to_string()));
            pos = tag_name_end + 1 + val_end + close_tag.len();
        } else {
            pos = tag_name_end + 1;
        }
    }

    Some(RawToolCall {
        name: func_name.to_string(),
        arguments: Value::Object(args),
    })
}

/// Parse inner content of a `<function>` tag.
///
/// Handles:
/// - `name [json {…}](code)` (bracket format)
/// - `name {json}` (inline)
/// - `name\n{json}` (multiline)
fn parse_function_tag_call(inner: &str) -> Option<RawToolCall> {
    let inner = inner.trim();
    if inner.is_empty() {
        return None;
    }

    // Split on first whitespace or newline to get function name
    let (name, rest) = match inner.find(|c: char| c.is_whitespace()) {
        Some(idx) => (inner[..idx].trim(), inner[idx..].trim()),
        None => (inner, ""),
    };

    if name.is_empty() {
        return None;
    }

    let arguments = if rest.is_empty() {
        Value::Object(Default::default())
    } else if rest.starts_with('{') {
        extract_balanced_json(rest)
    } else if let Some(json_start) = rest.find('{') {
        extract_balanced_json(&rest[json_start..])
    } else {
        Value::Object(Default::default())
    };

    Some(RawToolCall {
        name: name.to_string(),
        arguments,
    })
}

/// Parse `"function_name {json}"` or `"function_name [json {…}](code)"` after
/// the `<|tool▁sep|>` marker.
fn parse_single_raw_tool_call(text: &str) -> Option<RawToolCall> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    // Split on first whitespace to get function name
    let (name, rest) = match text.find(|c: char| c.is_whitespace()) {
        Some(idx) => (&text[..idx], text[idx..].trim()),
        None => (text, ""),
    };

    if name.is_empty() || name == "function" {
        // "function" is a type prefix some models emit before the separator —
        // shouldn't appear after the separator, but guard anyway.
        return None;
    }

    let arguments = if rest.is_empty() {
        Value::Object(Default::default())
    } else if rest.starts_with('{') {
        extract_balanced_json(rest)
    } else if let Some(json_start) = rest.find('{') {
        // e.g. "[json {…}](code)" — find the JSON inside
        extract_balanced_json(&rest[json_start..])
    } else {
        Value::Object(Default::default())
    };

    Some(RawToolCall {
        name: name.to_string(),
        arguments,
    })
}

/// Parse inner content of a `<tool_call>` tag (cogito official format).
///
/// Format: `{"name":"function_name","arguments":{"arg":"value"}}`
fn parse_tool_call_tag(inner: &str) -> Option<RawToolCall> {
    let inner = inner.trim();
    if inner.is_empty() {
        return None;
    }

    let json_start = inner.find('{')?;
    let json_val = extract_balanced_json(&inner[json_start..]);

    let func_name = json_val["name"].as_str()?;
    let arguments = json_val
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));

    Some(RawToolCall {
        name: func_name.to_string(),
        arguments,
    })
}

/// Parse inner content of an `<action>` tag.
///
/// Format: `[json {"function":"name","params":{…}}](code)`
/// or just: `{"function":"name","params":{…}}`
fn parse_action_tag_call(inner: &str) -> Option<RawToolCall> {
    let inner = inner.trim();
    if inner.is_empty() {
        return None;
    }

    // Find JSON object in the content
    let json_start = inner.find('{')?;
    let json_val = extract_balanced_json(&inner[json_start..]);

    // Extract function name and params from the JSON wrapper
    let func_name = json_val["function"].as_str()?;
    let params = json_val
        .get("params")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));

    Some(RawToolCall {
        name: func_name.to_string(),
        arguments: params,
    })
}

/// Extract a balanced JSON object starting at position 0 of `text`.
fn extract_balanced_json(text: &str) -> Value {
    let mut depth = 0i32;
    let mut end = 0;
    for (i, c) in text.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    if end > 0 {
        serde_json::from_str(&text[..end]).unwrap_or(Value::Object(Default::default()))
    } else {
        Value::Object(Default::default())
    }
}

/// Build a text description of available tools for injection into system message
/// when the Ollama tools API is not supported by the model.
fn tools_to_system_prompt_text(tools: &[AiToolDeclaration]) -> String {
    let mut text = String::from(
        "# Tool Use\n\n\
         You have access to tools. To call a tool you MUST output EXACTLY this format — no other format will work:\n\n\
         ```\n<tool_call>\n{\"name\": \"TOOL_NAME\", \"arguments\": {\"param\": \"value\"}}\n</tool_call>\n```\n\n\
         IMPORTANT: Do NOT describe what you want to do — just output the <tool_call> block. Do NOT use any other format.\n\n\
         ## Available tools:\n\n",
    );
    for t in tools {
        text.push_str(&format!("- **{}**: {}\n", t.name, t.description));
        if let Some(props) = t.parameters["properties"].as_object() {
            for (k, v) in props {
                let desc = v["description"].as_str().unwrap_or("");
                let typ = v["type"].as_str().unwrap_or("string");
                text.push_str(&format!("  - `{k}` ({typ}): {desc}\n"));
            }
        }
    }
    text
}

/// Convert tool declarations to Ollama tools JSON format.
pub fn tools_to_ollama_json(tools: &[AiToolDeclaration]) -> Vec<Value> {
    tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                }
            })
        })
        .collect()
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
pub fn spawn_ollama_check(
    base_url: String,
    api_key: Option<String>,
) -> mpsc::Receiver<OllamaStatus> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();
        let url = format!("{base_url}/api/tags");
        let req = if let Some(ref key) = api_key {
            agent
                .get(&url)
                .set("Authorization", &format!("Bearer {key}"))
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
        assert_eq!(result, vec!["llama3:latest", "codellama:7b"]);
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
    fn parse_show_response_full() {
        let json = r#"{
            "details": {
                "family": "qwen3",
                "parameter_size": "32B",
                "quantization_level": "Q4_K_M"
            },
            "parameters": "stop \"<|im_end|>\"\nnum_ctx 40960",
            "model_info": {
                "qwen3.context_length": 40960
            }
        }"#;
        let info = parse_show_response(json).unwrap();
        assert_eq!(info.family, "qwen3");
        assert_eq!(info.parameter_size, "32B");
        assert_eq!(info.quantization_level, "Q4_K_M");
        assert_eq!(info.context_length, Some(40960));
        assert!(info.parameters.contains("num_ctx 40960"));
    }

    #[test]
    fn parse_show_response_minimal() {
        let json = r#"{"details": {}, "model_info": {}}"#;
        let info = parse_show_response(json).unwrap();
        assert_eq!(info.family, "");
        assert_eq!(info.parameter_size, "");
        assert_eq!(info.context_length, None);
    }

    #[test]
    fn parse_ndjson_token() {
        let line =
            r#"{"model":"llama3","message":{"role":"assistant","content":"Hello"},"done":false}"#;
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
        let provider = OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "llama3".to_string(),
            None,
        );
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn ollama_provider_capabilities() {
        let provider = OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "llama3".to_string(),
            None,
        );
        let caps = provider.capabilities();
        assert!(caps.supports_streaming);
        assert!(caps.supports_tools);
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

    #[test]
    fn parse_ndjson_tool_call() {
        let line = r#"{"model":"llama3","message":{"role":"assistant","content":"","tool_calls":[{"function":{"name":"read_project_file","arguments":{"path":"src/main.rs"}}}]},"done":false}"#;
        let event = parse_ndjson_line(line).unwrap();
        if let StreamEvent::ToolCall {
            id,
            name,
            arguments,
        } = event
        {
            assert!(id.starts_with("tc_read_project_file_"));
            assert_eq!(name, "read_project_file");
            assert_eq!(arguments["path"], "src/main.rs");
        } else {
            panic!("Expected ToolCall, got {:?}", event);
        }
    }

    #[test]
    fn parse_ndjson_tool_call_id_format() {
        // Two consecutive tool_calls should have incrementing IDs
        let line = r#"{"model":"llama3","message":{"role":"assistant","content":"","tool_calls":[{"function":{"name":"exec","arguments":{"command":"cargo check"}}}]},"done":false}"#;
        let event1 = parse_ndjson_line(line).unwrap();
        let event2 = parse_ndjson_line(line).unwrap();
        if let (StreamEvent::ToolCall { id: id1, .. }, StreamEvent::ToolCall { id: id2, .. }) =
            (event1, event2)
        {
            assert!(id1.starts_with("tc_exec_"));
            assert!(id2.starts_with("tc_exec_"));
            assert_ne!(id1, id2); // IDs must be unique
        } else {
            panic!("Expected ToolCall variants");
        }
    }

    #[test]
    fn tools_to_ollama_json_format() {
        let tools = vec![AiToolDeclaration {
            name: "exec".to_string(),
            description: "Run a command".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" }
                },
                "required": ["command"]
            }),
        }];
        let json = tools_to_ollama_json(&tools);
        assert_eq!(json.len(), 1);
        assert_eq!(json[0]["type"], "function");
        assert_eq!(json[0]["function"]["name"], "exec");
        assert_eq!(json[0]["function"]["description"], "Run a command");
        assert!(json[0]["function"]["parameters"]["properties"]["command"].is_object());
    }

    #[test]
    fn serialize_message_regular() {
        let msg = AiMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            monologue: Vec::new(),
            timestamp: 0,
            tool_call_name: None,
            tool_call_id: None,
            tool_result_for_id: None,
            tool_is_error: false,
            tool_call_arguments: None,
        };
        let json = serialize_message(&msg);
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"], "Hello");
        assert!(json.get("tool_calls").is_none());
    }

    #[test]
    fn serialize_message_assistant_with_tool_call() {
        let msg = AiMessage {
            role: "assistant".to_string(),
            content: String::new(),
            monologue: Vec::new(),
            timestamp: 0,
            tool_call_name: Some("read_project_file".to_string()),
            tool_call_id: Some("tc_read_1".to_string()),
            tool_result_for_id: None,
            tool_is_error: false,
            tool_call_arguments: Some(serde_json::json!({"path": "src/main.rs"})),
        };
        let json = serialize_message(&msg);
        assert_eq!(json["role"], "assistant");
        let tc = &json["tool_calls"][0];
        assert_eq!(tc["function"]["name"], "read_project_file");
        assert_eq!(tc["function"]["arguments"]["path"], "src/main.rs");
    }

    #[test]
    fn serialize_message_tool_result() {
        let msg = AiMessage {
            role: "tool".to_string(),
            content: "file content here".to_string(),
            monologue: Vec::new(),
            timestamp: 0,
            tool_call_name: None,
            tool_call_id: None,
            tool_result_for_id: Some("tc_read_1".to_string()),
            tool_is_error: false,
            tool_call_arguments: None,
        };
        let json = serialize_message(&msg);
        assert_eq!(json["role"], "tool");
        assert_eq!(json["content"], "file content here");
    }

    // --- Raw tool-call parser tests ---

    #[test]
    fn parse_raw_tool_calls_cogito_format() {
        let content = "<|tool\u{2581}calls\u{2581}begin|><|tool\u{2581}call\u{2581}begin|>function<|tool\u{2581}sep|>read_project_file\n{\"path\": \"src/main.rs\"}\n<|tool\u{2581}call\u{2581}end|><|tool\u{2581}calls\u{2581}end|>";
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_project_file");
        assert_eq!(calls[0].arguments["path"], "src/main.rs");
        assert!(clean.is_empty());
    }

    #[test]
    fn parse_raw_tool_calls_bracket_format() {
        // cogito sometimes uses: name [json {…}](code)
        let content = "<|tool\u{2581}calls\u{2581}begin|><|tool\u{2581}call\u{2581}begin|>function<|tool\u{2581}sep|>read_project_file [json {\"path\": \"src/main.rs\"} ](code)<|tool\u{2581}call\u{2581}end|><|tool\u{2581}calls\u{2581}end|>";
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_project_file");
        assert_eq!(calls[0].arguments["path"], "src/main.rs");
        assert!(clean.is_empty());
    }

    #[test]
    fn parse_raw_tool_calls_with_surrounding_text() {
        let content = "Let me read that file for you.\n<|tool\u{2581}calls\u{2581}begin|><|tool\u{2581}call\u{2581}begin|>function<|tool\u{2581}sep|>read_project_file\n{\"path\": \"src/main.rs\"}\n<|tool\u{2581}call\u{2581}end|><|tool\u{2581}calls\u{2581}end|>\nDone.";
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_project_file");
        assert!(clean.contains("Let me read that file"));
        assert!(clean.contains("Done."));
    }

    #[test]
    fn parse_raw_tool_calls_no_markers() {
        let content = "Just regular text, no tool calls here.";
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert!(calls.is_empty());
        assert_eq!(clean, content);
    }

    #[test]
    fn parse_raw_tool_calls_multiple_calls() {
        let content = "<|tool\u{2581}calls\u{2581}begin|><|tool\u{2581}call\u{2581}begin|>function<|tool\u{2581}sep|>read_project_file\n{\"path\": \"src/main.rs\"}\n<|tool\u{2581}call\u{2581}end|><|tool\u{2581}call\u{2581}begin|>function<|tool\u{2581}sep|>exec\n{\"command\": \"cargo check\"}\n<|tool\u{2581}call\u{2581}end|><|tool\u{2581}calls\u{2581}end|>";
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "read_project_file");
        assert_eq!(calls[1].name, "exec");
        assert_eq!(calls[1].arguments["command"], "cargo check");
        assert!(clean.is_empty());
    }

    #[test]
    fn parse_raw_tool_calls_inline_json() {
        // Some models put the JSON on the same line
        let content = "<|tool\u{2581}calls\u{2581}begin|><|tool\u{2581}call\u{2581}begin|>function<|tool\u{2581}sep|>exec {\"command\": \"ls -la\"}<|tool\u{2581}call\u{2581}end|><|tool\u{2581}calls\u{2581}end|>";
        let (calls, _) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "exec");
        assert_eq!(calls[0].arguments["command"], "ls -la");
    }

    #[test]
    fn parse_raw_tool_calls_cogito_tool_call_tag() {
        // Official cogito format: <tool_call>{"name":"func","arguments":{…}}</tool_call>
        let content = r#"<tool_call>
{"name": "read_project_file", "arguments": {"path": "src/main.rs"}}
</tool_call>"#;
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_project_file");
        assert_eq!(calls[0].arguments["path"], "src/main.rs");
        assert!(clean.is_empty());
    }

    #[test]
    fn parse_raw_tool_calls_cogito_tool_call_with_eot() {
        let content = r#"<tool_call>
{"name": "exec", "arguments": {"command": "cargo check"}}
</tool_call><|eot_id|>"#;
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "exec");
        assert_eq!(calls[0].arguments["command"], "cargo check");
        assert!(clean.is_empty());
    }

    #[test]
    fn parse_raw_tool_calls_cogito_tool_call_with_text() {
        let content = "Let me read that file.\n<tool_call>\n{\"name\": \"read_project_file\", \"arguments\": {\"path\": \"src/main.rs\"}}\n</tool_call>";
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_project_file");
        assert!(clean.contains("Let me read that file."));
    }

    #[test]
    fn parse_raw_tool_calls_bare_json_code_link() {
        // cogito v2.1 markdown-link format
        let content =
            r#"[json {"name": "semantic_search", "arguments": {"query": "main entry"}} ](code)"#;
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "semantic_search");
        assert_eq!(calls[0].arguments["query"], "main entry");
        assert!(clean.is_empty());
    }

    #[test]
    fn parse_raw_tool_calls_bare_json_code_link_with_text() {
        let content = "Let me search.\n\n[json {\"name\": \"read_project_file\", \"arguments\": {\"path\": \"src/main.rs\"}} ](code)\n\nDone.";
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_project_file");
        assert!(clean.contains("Let me search."));
        assert!(clean.contains("Done."));
    }

    #[test]
    fn parse_raw_tool_calls_bare_json_function_params_format() {
        let content =
            r#"[json {"function": "read_project_file", "params": {"path": "src/main.rs"}} ](code)"#;
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_project_file");
        assert_eq!(calls[0].arguments["path"], "src/main.rs");
        assert!(clean.is_empty());
    }

    #[test]
    fn parse_raw_tool_calls_cogito_parallel_tool_calls() {
        let content = "<tool_call>\n{\"name\": \"read_project_file\", \"arguments\": {\"path\": \"src/main.rs\"}}\n</tool_call>\n<tool_call>\n{\"name\": \"read_project_file\", \"arguments\": {\"path\": \"Cargo.toml\"}}\n</tool_call>";
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].arguments["path"], "src/main.rs");
        assert_eq!(calls[1].arguments["path"], "Cargo.toml");
        assert!(clean.is_empty());
    }

    #[test]
    fn parse_raw_tool_calls_function_tag_inline() {
        let content = r#"<function> read_project_file [json {"path": "src/main.rs"} ](code)"#;
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_project_file");
        assert_eq!(calls[0].arguments["path"], "src/main.rs");
        assert!(clean.is_empty());
    }

    #[test]
    fn parse_raw_tool_calls_function_tag_multiline() {
        let content = "<function>exec\n{\"command\": \"cargo check\"}\n</function>";
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "exec");
        assert_eq!(calls[0].arguments["command"], "cargo check");
        assert!(clean.is_empty());
    }

    #[test]
    fn parse_raw_tool_calls_function_tag_with_text() {
        let content = "Let me check.\n<function> read_project_file [json {\"path\": \"src/main.rs\"} ](code)\nSure.";
        let (calls, clean) = parse_raw_tool_calls_from_content(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_project_file");
        assert!(clean.contains("Let me check."));
        assert!(clean.contains("Sure."));
    }

    #[test]
    fn strip_thinking_tags_basic() {
        let content = "<thinking>\nI need to read the file.\n</thinking>\nHere is the result.";
        let (thinking, clean) = strip_thinking_tags(content);
        assert!(thinking.is_some());
        assert!(thinking.unwrap().contains("I need to read the file."));
        assert!(clean.contains("Here is the result."));
        assert!(!clean.contains("<thinking>"));
    }

    #[test]
    fn strip_thinking_tags_none() {
        let content = "Just regular text.";
        let (thinking, clean) = strip_thinking_tags(content);
        assert!(thinking.is_none());
        assert_eq!(clean, content);
    }

    #[test]
    fn strip_thinking_tags_unclosed() {
        let content = "<thinking>\nThinking forever...";
        let (thinking, clean) = strip_thinking_tags(content);
        assert!(thinking.is_some());
        assert!(thinking.unwrap().contains("Thinking forever..."));
        assert!(clean.is_empty());
    }

    #[test]
    fn extract_balanced_json_simple() {
        let val = extract_balanced_json(r#"{"a": 1, "b": "hello"} trailing"#);
        assert_eq!(val["a"], 1);
        assert_eq!(val["b"], "hello");
    }

    #[test]
    fn extract_balanced_json_nested() {
        let val = extract_balanced_json(r#"{"a": {"b": {"c": 1}}} extra"#);
        assert_eq!(val["a"]["b"]["c"], 1);
    }
}
