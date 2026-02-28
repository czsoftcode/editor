use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Debug)]
struct OllamaRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<Tool>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    #[serde(default)]
    role: String,
    #[serde(default)]
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_content: Option<String>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            role: "assistant".to_string(),
            content: String::new(),
            tool_calls: None,
            tool_call_id: None,
            tool_name: None,
            reasoning_content: None,
        }
    }
}

#[derive(Serialize, Clone, Debug)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: FunctionDeclaration,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FunctionDeclaration {
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ToolCall {
    #[serde(default)]
    id: String,
    #[serde(rename = "type", default = "default_tool_type")]
    tool_type: String,
    function: FunctionCall,
}

fn default_tool_type() -> String {
    "function".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FunctionCall {
    name: String,
    #[serde(default)]
    arguments: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct OllamaResponse {
    #[serde(default)]
    model: Option<String>,
    #[serde(alias = "messages", default)]
    message: Message,
    #[serde(default)]
    response: Option<String>,
    #[serde(default)]
    done: bool,
    prompt_eval_count: Option<u64>,
    eval_count: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct OllamaErrorResponse {
    #[serde(alias = "message")]
    error: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct PluginInput {
    prompt: String,
    history: Vec<(String, String)>,
    context: Option<AiContextPayload>,
    tools: Option<Vec<FunctionDeclaration>>,
}

#[derive(Deserialize, Debug)]
pub struct AiContextPayload {
    pub open_files: Vec<AiFileContext>,
    pub build_errors: Vec<AiBuildErrorContext>,
    pub active_file: Option<AiFileContext>,
    #[serde(default)]
    pub memory_keys: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct AiFileContext {
    pub path: String,
    pub content: Option<String>,
    #[serde(default)]
    pub is_active: bool,
}

#[derive(Deserialize, Debug)]
pub struct AiBuildErrorContext {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub is_warning: bool,
}

#[host_fn]
extern "ExtismHost" {
    fn read_project_file(path: String) -> String;
    fn write_project_file(input: String);
    fn replace_project_file(input: String);
    fn list_project_files() -> String;
    fn search_project(query: String) -> String;
    fn semantic_search(query: String) -> String;
    fn exec_in_sandbox(command: String) -> String;
    fn store_scratch(input: String);
    fn retrieve_scratch(key: String) -> String;
    fn store_fact(input: String);
    fn retrieve_fact(key: String) -> String;
    fn list_facts() -> String;
    fn delete_fact(key: String);
    fn ask_user(input: String) -> String;
    fn announce_completion(input: String);
    fn log_monologue(message: String);
    fn log_usage(in_tokens: u64, out_tokens: u64);
    fn log_payload(payload: String);
}

fn save_trace(trace: &str) {
    let log_input = serde_json::json!({"path": "ollama_trace.log", "content": trace});
    if let Ok(json_str) = serde_json::to_string(&log_input) {
        let _ = unsafe { write_project_file(json_str) };
    }
}

#[plugin_fn]
pub fn ask_ollama(input_json: String) -> FnResult<String> {
    let input: PluginInput = serde_json::from_str(&input_json)?;
    let base_url = config::get("API_URL")?.unwrap_or_else(|| "http://localhost:11434".to_string());
    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let model = config::get("MODEL")?.unwrap_or_else(|| "llama3.1".to_string());

    let tools = if let Some(decls) = input.tools {
        decls.into_iter().map(|f| Tool { tool_type: "function".to_string(), function: f }).collect()
    } else {
        vec![]
    };

    let mut options = HashMap::new();
    
    // Set stable Temperature (default 0.2 if not set)
    let temp_val = config::get("TEMPERATURE")?
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.2);
    options.insert("temperature".to_string(), serde_json::json!(temp_val));

    // Set stable Context Window (default 8192 if not set)
    let ctx_val = config::get("NUM_CTX")?
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(8192);
    options.insert("num_ctx".to_string(), serde_json::json!(ctx_val));

    let mut messages = Vec::new();

    // System instruction
    let language_code = config::get("LANGUAGE")?.unwrap_or_else(|| "en".to_string());
    let language_name = match language_code.to_lowercase().as_str() {
        "cs" | "czech" => "Czech",
        "sk" | "slovak" => "Slovak",
        "de" | "german" => "German",
        "ru" | "russian" => "Russian",
        _ => "English",
    };

    let mut system_prompt = config::get("SYSTEM_PROMPT")?.unwrap_or_else(|| "Expert Rust Developer.".to_string());
    if language_name != "English" {
        system_prompt.push_str(&format!(
            "\n\nSTRICT LANGUAGE RULE: You MUST speak ONLY in the following language: '{}'. This applies to your final response, your inner monologue, and your thoughts. NEVER switch to English.", 
            language_name
        ));
    }
    system_prompt.push_str("\nMANDATE: LONG-TERM MEMORY: The context payload contains `memory_keys`, which is a LIST of fact names you have stored. It is NOT a key itself. At the START of a new task, you MUST review this list. If you see relevant keys, use `retrieve_fact` on EACH of them to recall your memory before proceeding.");
    
    system_prompt.push_str("\nUse 'rg' (ripgrep) via 'search_project' for fast searching. For complex architectural questions or when you need to find relevant logic across the entire codebase, use 'semantic_search'.");
    system_prompt.push_str("\nIf you feel the context is getting full or you're missing information, proactively use 'semantic_search' to find the most relevant snippets.");

    messages.push(Message {
        role: "system".to_string(),
        content: system_prompt,
        ..Default::default()
    });

    // History (Cleaned with Sliding Window)
    let mut history_messages = Vec::new();
    for (q, a) in &input.history {
        if q.is_empty() || a.contains("____") { continue; }
        history_messages.push(Message { role: "user".to_string(), content: q.clone(), ..Default::default() });
        history_messages.push(Message { role: "assistant".to_string(), content: a.clone(), ..Default::default() });
    }

    // Keep only last 20 messages of history to avoid context overflow
    if history_messages.len() > 20 {
        let start = history_messages.len() - 20;
        messages.extend(history_messages[start..].to_vec());
    } else {
        messages.extend(history_messages);
    }

    // Context formatting
    let mut context_str = String::new();
    if let Some(ctx) = input.context {
        if !ctx.memory_keys.is_empty() {
            context_str.push_str(&format!(
                "Long-term memory keys available: {}\nIMPORTANT: Before you begin, use 'retrieve_fact' on ANY relevant key to recall your memory.\n",
                ctx.memory_keys.join(", ")
            ));
        }
        if let Some(active) = ctx.active_file {
            context_str.push_str(&format!("Active file: {}\n", active.path));
        }
        if !ctx.open_files.is_empty() {
            context_str.push_str("Open files: ");
            let paths: Vec<String> = ctx.open_files.into_iter().map(|f| f.path).collect();
            context_str.push_str(&paths.join(", "));
            context_str.push_str("\n");
        }
        if !ctx.build_errors.is_empty() {
            context_str.push_str(&format!("Build errors found: {}\n", ctx.build_errors.len()));
        }
    }

    let user_prompt = if context_str.is_empty() {
        input.prompt.clone()
    } else {
        format!("Context:\n{}\nQuestion: {}", context_str, input.prompt)
    };

    messages.push(Message { role: "user".to_string(), content: user_prompt, ..Default::default() });

                    let mut current_iteration = 0;
                    let max_iterations: i32 = config::get("MAX_ITERATIONS")?
                        .and_then(|v| v.parse::<i32>().ok())
                        .unwrap_or(30);
                    let mut trace_log = format!("--- TRACE ---\nPrompt: {}\n\n", input.prompt);
                
                    loop {
                        current_iteration += 1;
                        if current_iteration > max_iterations {
                break;
            }

        let req = OllamaRequest {
            model: model.clone(),
            messages: messages.clone(),
            tools: tools.clone(),
            stream: false,
            options: Some(options.clone()),
        };

        let body = serde_json::to_string(&req)?;
        if let Ok(pretty) = serde_json::to_string_pretty(&req) { 
            let _ = unsafe { log_payload(pretty) }; 
        }

        let mut http_req = HttpRequest::new(&url).with_method("POST").with_header("Content-Type", "application/json");
        
        if let Ok(Some(api_key)) = config::get("API_KEY") {
            if !api_key.trim().is_empty() {
                http_req = http_req.with_header("Authorization", format!("Bearer {}", api_key.trim()));
            }
        }

        let resp = match http::request(&http_req, Some(body)) {
            Ok(r) => r,
            Err(e) => {
                let err_msg = format!("HTTP Request failed: {}", e);
                save_trace(&trace_log);
                return Err(anyhow::anyhow!(err_msg).into());
            }
        };
        
        let raw_resp = String::from_utf8_lossy(&resp.body()).into_owned();
        trace_log.push_str(&format!("\n--- API RESPONSE (Iteration {}) ---\n{}\n------------------\n", current_iteration, raw_resp));

        if resp.status() != 200 && resp.status() != 0 {
            let error_msg = if let Ok(err_json) = serde_json::from_slice::<OllamaErrorResponse>(&resp.body()) {
                err_json.error
            } else {
                raw_resp.clone()
            };
            save_trace(&trace_log);
            return Err(anyhow::anyhow!("Ollama API Error {}: {}", resp.status(), error_msg).into());
        }

        let ollama_resp: OllamaResponse = match serde_json::from_slice(&resp.body()) {
            Ok(r) => r,
            Err(e) => {
                if let Ok(err_json) = serde_json::from_slice::<OllamaErrorResponse>(&resp.body()) {
                    save_trace(&trace_log);
                    return Err(anyhow::anyhow!("Ollama Error: {}", err_json.error).into());
                }
                save_trace(&trace_log);
                return Err(anyhow::anyhow!("JSON Deserialization Error: {}. Response body: {}", e, raw_resp).into());
            }
        };
        
        let in_tokens = ollama_resp.prompt_eval_count.unwrap_or(0);
        let out_tokens = ollama_resp.eval_count.unwrap_or(0);
        let _ = unsafe { log_usage(in_tokens, out_tokens) };

        let msg = ollama_resp.message;
        
        // Handle Reasoning
        if let Some(reasoning) = &msg.reasoning_content {
            if !reasoning.trim().is_empty() {
                let _ = unsafe { log_monologue(format!("🤔 THOUGHTS:\n{}", reasoning)) };
            }
        }

        let mut content = if !msg.content.is_empty() {
            msg.content.clone()
        } else {
            ollama_resp.response.unwrap_or_default()
        };
        
        // --- NEW: Detect and extract thought/reasoning tags from content ---
        let thought_tags = [("<thought>", "</thought>"), ("<reasoning>", "</reasoning>")];
        for (start_tag, end_tag) in thought_tags {
            if let Some(start_pos) = content.find(start_tag) {
                if let Some(end_pos) = content.find(end_tag) {
                    let thought_start = start_pos + start_tag.len();
                    let reasoning = &content[thought_start..end_pos];
                    if !reasoning.trim().is_empty() {
                        let _ = unsafe { log_monologue(format!("🤔 THOUGHTS:\n{}", reasoning.trim())) };
                    }
                    // Remove the thought block from final content
                    let mut new_content = content[..start_pos].to_string();
                    new_content.push_str(&content[end_pos + end_tag.len()..]);
                    content = new_content.trim().to_string();
                }
            }
        }
        // ------------------------------------------------------------------

        if let Some(tool_calls) = &msg.tool_calls {
            if !tool_calls.is_empty() {
                // Mezikrok — obsah modelu je myšlenka/analýza, logujeme jako monolog
                if !content.is_empty() {
                    let _ = unsafe { log_monologue(content.clone()) };
                    trace_log.push_str(&content);
                    trace_log.push_str("\n");
                }
                messages.push(msg.clone());

                for call in tool_calls {
                    let name = call.function.name.strip_prefix("default_api:").unwrap_or(&call.function.name);
                    let _ = unsafe { log_monologue(format!("Calling tool: {}...", name)) };
                    trace_log.push_str(&format!("\nCall: {} {:?}\n", name, call.function.arguments));

                    let result = if name == "read_project_file" {
                        let mut args = call.function.arguments.clone();
                        if let Ok(Some(limit_str)) = config::get("MAX_READ_CHARS") {
                            if let Ok(limit) = limit_str.parse::<u64>() {
                                if let Some(obj) = args.as_object_mut() {
                                    obj.insert("max_chars_limit".to_string(), serde_json::json!(limit));
                                }
                            }
                        }
                        match unsafe { read_project_file(serde_json::to_string(&args)?) } {
                            Ok(res) => serde_json::json!({ "content": res }),
                            Err(e) => serde_json::json!({ "error": format!("Read failed: {}", e) }),
                        }
                    } else if name == "write_file" {
                        unsafe { let _ = write_project_file(serde_json::to_string(&call.function.arguments)?); }
                        serde_json::json!({ "status": "success" })
                    } else if name == "replace" {
                        unsafe { let _ = replace_project_file(serde_json::to_string(&call.function.arguments)?); }
                        serde_json::json!({ "status": "success" })
                    } else if name == "semantic_search" || name == "search_project" {
                        let q = call.function.arguments["query"].as_str().unwrap_or("");
                        let res_result = if name == "semantic_search" { unsafe { semantic_search(q.to_string()) } } else { unsafe { search_project(q.to_string()) } };
                        match res_result {
                            Ok(res_str) => {
                                let res_json: serde_json::Value = serde_json::from_str(&res_str).unwrap_or(serde_json::json!([]));
                                serde_json::json!({ "results": res_json })
                            },
                            Err(e) => serde_json::json!({ "error": format!("Search failed: {}", e) }),
                        }
                    } else if name == "list_project_files" {
                        match unsafe { list_project_files() } {
                            Ok(res_str) => serde_json::from_str(&res_str).unwrap_or(serde_json::json!({"error": "invalid host response"})),
                            Err(e) => serde_json::json!({ "error": format!("Listing failed: {}", e) }),
                        }
                    } else if name == "exec_in_sandbox" {
                        match unsafe { exec_in_sandbox(call.function.arguments["command"].as_str().unwrap_or("").to_string()) } {
                            Ok(res) => serde_json::json!({ "output": res }),
                            Err(e) => serde_json::json!({ "error": format!("Execution failed: {}", e) }),
                        }
                    } else if name == "store_scratch" {
                        unsafe { let _ = store_scratch(serde_json::to_string(&call.function.arguments)?); }
                        serde_json::json!({ "status": "success" })
                    } else if name == "retrieve_scratch" {
                        let key = call.function.arguments["key"].as_str().unwrap_or("").to_string();
                        match unsafe { retrieve_scratch(key) } {
                            Ok(res) => serde_json::json!({ "value": res }),
                            Err(e) => serde_json::json!({ "error": format!("Scratch retrieval failed: {}", e) }),
                        }
                    } else if name == "store_fact" {
                        unsafe { let _ = store_fact(serde_json::to_string(&call.function.arguments)?); }
                        serde_json::json!({ "status": "success" })
                    } else if name == "retrieve_fact" {
                        let key = call.function.arguments["key"].as_str().unwrap_or("").to_string();
                        match unsafe { retrieve_fact(key) } {
                            Ok(res) => serde_json::json!({ "value": res }),
                            Err(e) => serde_json::json!({ "error": format!("Retrieval failed: {}", e) }),
                        }
                    } else if name == "list_facts" {
                        match unsafe { list_facts() } {
                            Ok(res_str) => serde_json::from_str(&res_str).unwrap_or(serde_json::json!({"keys": []})),
                            Err(e) => serde_json::json!({ "error": format!("list_facts failed: {}", e) }),
                        }
                    } else if name == "delete_fact" {
                        let key = call.function.arguments["key"].as_str().unwrap_or("").to_string();
                        unsafe { let _ = delete_fact(key); }
                        serde_json::json!({ "status": "deleted" })
                    } else if name == "ask_user" {
                        match unsafe { ask_user(serde_json::to_string(&call.function.arguments)?) } {
                            Ok(answer) => serde_json::json!({ "answer": answer }),
                            Err(e) => serde_json::json!({ "error": format!("ask_user failed: {}", e) }),
                        }
                    } else if name == "announce_completion" {
                        unsafe { let _ = announce_completion(serde_json::to_string(&call.function.arguments)?); }
                        serde_json::json!({ "status": "completed" })
                    } else { serde_json::json!({"error": "unknown function"}) };

                    let mut content = serde_json::to_string(&result)?;
                    // SAFETY: Truncate tool output if it's too long to prevent Ollama server crash/OOM
                    if content.len() > 16384 {
                        content = format!("{}... [TRUNCATED due to length]", &content[..16384]);
                    }

                    messages.push(Message {
                        role: "tool".to_string(),
                        content,
                        tool_calls: None,
                        tool_call_id: if call.id.is_empty() { None } else { Some(call.id.clone()) },
                        tool_name: Some(call.function.name.clone()),
                        reasoning_content: None,
                    });
                }
                
                save_trace(&trace_log);
                continue;
            }
        }

        save_trace(&trace_log);
        return Ok(content);
    }

    Ok("Exceeded depth".to_string())
}
