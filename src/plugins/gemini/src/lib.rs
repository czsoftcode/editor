use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<Content>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<Tool>,
}

#[derive(Serialize, Clone)]
struct Tool {
    function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Serialize, Deserialize, Clone)]
struct FunctionDeclaration {
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct Content {
    #[serde(default)]
    role: String,
    #[serde(default)]
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "functionCall")]
    function_call: Option<FunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "functionResponse")]
    function_response: Option<FunctionResponse>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone)]
struct FunctionCall {
    name: String,
    #[serde(default)]
    args: serde_json::Value,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone)]
struct FunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Deserialize)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount", default)]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount", default)]
    candidates_token_count: u32,
    #[serde(rename = "totalTokenCount", default)]
    total_token_count: u32,
}

#[derive(Deserialize, Clone)]
struct Candidate {
    #[serde(default)]
    content: Content,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
    #[serde(rename = "finishMessage")]
    finish_message: Option<String>,
}

// --- Data structures for Unified Context & Tools ---

#[derive(Deserialize)]
struct PluginInput {
    prompt: String,
    history: Vec<(String, String)>,
    context: Option<AiContextPayload>,
    tools: Option<Vec<FunctionDeclaration>>,
}

#[derive(Deserialize)]
pub struct AiContextPayload {
    pub open_files: Vec<AiFileContext>,
    pub build_errors: Vec<AiBuildErrorContext>,
    pub active_file: Option<AiFileContext>,
}

#[derive(Deserialize)]
pub struct AiFileContext {
    pub path: String,
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub struct AiBuildErrorContext {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub is_warning: bool,
}

// --- Host Functions ---
#[host_fn]
extern "ExtismHost" {
    fn read_project_file(path: String) -> String;
    fn list_project_files() -> String;
    fn search_project(query: String) -> String;
    fn semantic_search(query: String) -> String;
    fn exec_in_sandbox(command: String) -> String;
    fn log_monologue(message: String);
    fn log_usage(tokens: u64);
}

#[plugin_fn]
pub fn ask_gemini(input_json: String) -> FnResult<String> {
    let input: PluginInput = serde_json::from_str(&input_json).map_err(|e| {
        anyhow::anyhow!("Failed to parse plugin input JSON: {}. Input was: {}", e, input_json)
    })?;
    
    let api_key = config::get("API_KEY")?.ok_or(anyhow::anyhow!("Missing API_KEY in plugin settings"))?;
    let model = config::get("MODEL")?.unwrap_or_else(|| "gemini-1.5-flash".to_string());
    
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );

    // 2. Prepare Tools
    let tools = if let Some(decls) = input.tools {
        vec![Tool { function_declarations: decls }]
    } else {
        vec![]
    };

    // 3. Build Chat History
    let mut messages = Vec::new();
    
    for (q, a) in &input.history {
        if q.is_empty() && a.is_empty() { continue; }
        if q.is_empty() && (a.contains("____") || a.contains("Version:")) {
            continue;
        }
        if !q.is_empty() {
            messages.push(Content {
                role: "user".to_string(),
                parts: vec![Part { text: Some(q.clone()), function_call: None, function_response: None, extra: HashMap::new() }],
            });
        }
        if !a.is_empty() {
            messages.push(Content {
                role: "model".to_string(),
                parts: vec![Part { text: Some(a.clone()), function_call: None, function_response: None, extra: HashMap::new() }],
            });
        }
    }

    let default_system_instruction = "You are an expert developer assistant with full access to a secure project sandbox.
    IMPORTANT: You must use the provided TOOLS for all project-related actions. 
    Use 'list_project_files' to see what files exist, 'read_project_file' to see code, 'search_project' for keywords, 
    or 'semantic_search' to find code related to concepts/meanings.";

    let mut system_instruction = config::get("SYSTEM_PROMPT")?.unwrap_or_else(|| default_system_instruction.to_string());
    let language = config::get("LANGUAGE")?.unwrap_or_else(|| "en".to_string());

    system_instruction.push_str("\n\nCORE SECURITY MANDATE: You are strictly confined to the project sandbox directory.");
    system_instruction.push_str(&format!("\nCRITICAL: You must communicate ONLY in this language: {}.", language));

    // Assemble Lean Context
    let mut context_info = String::new();
    
    if let Some(ctx) = &input.context {
        if !ctx.open_files.is_empty() {
            context_info.push_str("Currently open files in editor tabs:\n");
            for file in &ctx.open_files {
                let active_mark = if let Some(active) = &ctx.active_file {
                    if active.path == file.path { " (ACTIVE)" } else { "" }
                } else { "" };
                context_info.push_str(&format!("- {}{}\n", file.path, active_mark));
            }
            context_info.push('\n');
        }

        if !ctx.build_errors.is_empty() {
            context_info.push_str("Current Build Errors/Warnings:\n");
            for err in &ctx.build_errors {
                let level = if err.is_warning { "Warning" } else { "Error" };
                context_info.push_str(&format!("[{}] {}:{}: {}\n", level, err.file, err.line, err.message));
            }
            context_info.push('\n');
        }
        
        context_info.push_str("Note: Use 'list_project_files' if you need to know about files not listed above.\n");
    }

    let user_prompt = format!("Context:\n{}\n\nUser Question: {}", context_info, input.prompt);
    
    messages.push(Content {
        role: "user".to_string(),
        parts: vec![Part {
            text: Some(user_prompt),
            function_call: None,
            function_response: None,
            extra: HashMap::new(),
        }],
    });

    // 4. API Request Loop
    let mut current_iteration = 0;
    let mut last_total_tokens = 0u64;
    const MAX_ITERATIONS: i32 = 15;

    loop {
        current_iteration += 1;
        if current_iteration > MAX_ITERATIONS {
            let _ = unsafe { log_usage(last_total_tokens) };
            return Ok("Error: AI exceeded maximum tool call depth.".to_string());
        }

        let req = GeminiRequest {
            system_instruction: Some(Content {
                role: "system".to_string(),
                parts: vec![Part {
                    text: Some(system_instruction.to_string()),
                    function_call: None,
                    function_response: None,
                    extra: HashMap::new(),
                }],
            }),
            contents: messages.clone(),
            tools: if current_iteration == 1 {
                tools.clone()
            } else {
                vec![]
            },
        };

        let body = serde_json::to_string(&req)?;
        let http_req = HttpRequest::new(&url)
            .with_method("POST")
            .with_header("Content-Type", "application/json")
            .with_header("x-goog-api-key", &api_key);

        let resp = http::request(&http_req, Some(body))?;
        if resp.status() != 200 && resp.status() != 0 {
            let _ = unsafe { log_usage(last_total_tokens) };
            return Err(anyhow::anyhow!(
                "Gemini API error ({}): {}",
                resp.status(),
                String::from_utf8_lossy(&resp.body())
            )
            .into());
        }

        let body_bytes = resp.body();
        let gemini_resp: GeminiResponse = serde_json::from_slice(&body_bytes).map_err(|e| {
            let raw_body = String::from_utf8_lossy(&body_bytes);
            anyhow::anyhow!("Failed to parse Gemini response: {}. Body: {}", e, raw_body)
        })?;

        if let Some(usage) = &gemini_resp.usage_metadata {
            last_total_tokens = usage.total_token_count as u64;
            let usage_msg = format!(
                "Token usage: {} (In: {}, Out: {})",
                usage.total_token_count, usage.prompt_token_count, usage.candidates_token_count
            );
            let _ = unsafe { log_monologue(usage_msg) };
        }

        let candidates = gemini_resp.candidates.unwrap_or_default();
        if candidates.is_empty() {
            let _ = unsafe { log_usage(last_total_tokens) };
            return Ok("Error: Gemini API returned no candidates. This might be a safety block or API error.".to_string());
        }
        
        let candidate = candidates[0].clone();

        // Handle error states in candidates
        if let Some(reason) = &candidate.finish_reason {
            if reason == "MALFORMED_FUNCTION_CALL" {
                let _ = unsafe { log_usage(last_total_tokens) };
                return Ok(format!("Error: AI generated a malformed function call. Message: {}", candidate.finish_message.as_deref().unwrap_or("none")));
            }
        }

        let mut has_function_call = false;
        let mut response_parts = Vec::new();

        for part in &candidate.content.parts {
            if let Some(function_call) = &part.function_call {
                has_function_call = true;

                let func_name = function_call
                    .name
                    .strip_prefix("default_api:")
                    .unwrap_or(&function_call.name);

                let result = if func_name == "read_project_file" {
                    let path = function_call.args["path"].as_str().unwrap_or("");
                    let _ = unsafe { log_monologue(format!("Reading file: {}", path)) };
                    let content = unsafe { read_project_file(path.to_string())? };
                    serde_json::json!({ "content": content })
                } else if func_name == "search_project" {
                    let query = function_call.args["query"].as_str().unwrap_or("");
                    let _ = unsafe { log_monologue(format!("Searching project for: '{}'", query)) };
                    let results_json = unsafe { search_project(query.to_string())? };
                    let mut results: Vec<serde_json::Value> = serde_json::from_str(&results_json).unwrap_or(vec![]);
                    let total_found = results.len();
                    results.truncate(20);
                    serde_json::json!({ "results": results, "total_found": total_found })
                } else if func_name == "semantic_search" {
                    let query = function_call.args["query"].as_str().unwrap_or("");
                    let _ = unsafe { log_monologue(format!("Semantic search for: '{}'", query)) };
                    let results_json = unsafe { semantic_search(query.to_string())? };
                    let results: serde_json::Value = serde_json::from_str(&results_json).unwrap_or(serde_json::json!([]));
                    serde_json::json!({ "results": results })
                } else if func_name == "list_project_files" {
                    let _ = unsafe { log_monologue("Listing project files...".to_string()) };
                    let files = unsafe { list_project_files()? };
                    serde_json::json!({ "files": files })
                } else if func_name == "exec_in_sandbox" {
                    let cmd = function_call.args["command"].as_str().unwrap_or("");
                    let _ = unsafe { log_monologue(format!("Executing: {}", cmd)) };
                    let output = unsafe { exec_in_sandbox(cmd.to_string())? };
                    serde_json::json!({ "output": output })
                } else {
                    serde_json::json!({ "error": format!("Unknown function: {}", function_call.name) })
                };

                response_parts.push(Part {
                    text: None,
                    function_call: None,
                    function_response: Some(FunctionResponse {
                        name: function_call.name.clone(),
                        response: result,
                    }),
                    extra: HashMap::new(),
                });
            }
        }

        if has_function_call {
            messages.push(Content {
                role: "model".to_string(),
                parts: candidate.content.parts.clone(),
            });
            messages.push(Content {
                role: "user".to_string(),
                parts: response_parts,
            });
        } else {
            let _ = unsafe { log_usage(last_total_tokens) };
            let answer = candidate
                .content
                .parts
                .iter()
                .find_map(|p| p.text.clone())
                .unwrap_or_else(|| "No text response".to_string());
            return Ok(answer);
        }
    }
}
