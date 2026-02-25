use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Debug)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<Content>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<Tool>,
}

#[derive(Serialize, Clone, Debug)]
struct Tool {
    function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FunctionDeclaration {
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct Content {
    #[serde(default)]
    role: String,
    #[serde(default)]
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FunctionCall {
    name: String,
    #[serde(default)]
    args: serde_json::Value,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Deserialize, Debug)]
struct UsageMetadata {
    #[serde(rename = "totalTokenCount", default)]
    total_token_count: u32,
}

#[derive(Deserialize, Clone, Debug)]
struct Candidate {
    #[serde(default)]
    content: Content,
}

#[derive(Deserialize, Debug)]
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
}

#[derive(Deserialize, Debug)]
pub struct AiFileContext {
    pub path: String,
    pub content: Option<String>,
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
    fn list_project_files() -> String;
    fn search_project(query: String) -> String;
    fn semantic_search(query: String) -> String;
    fn exec_in_sandbox(command: String) -> String;
    fn log_monologue(message: String);
    fn log_usage(tokens: u64);
    fn log_payload(payload: String);
}

#[plugin_fn]
pub fn ask_gemini(input_json: String) -> FnResult<String> {
    let input: PluginInput = serde_json::from_str(&input_json)?;
    let api_key = config::get("API_KEY")?.ok_or(anyhow::anyhow!("Missing API_KEY"))?;
    let model = config::get("MODEL")?.unwrap_or_else(|| "gemini-1.5-flash".to_string());
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model);

    let tools = if let Some(decls) = input.tools { vec![Tool { function_declarations: decls }] } else { vec![] };

    let mut messages = Vec::new();
    for (q, a) in &input.history {
        if q.is_empty() && (a.contains("____") || a.contains("Version:")) { continue; }
        if !q.is_empty() {
            messages.push(Content { role: "user".to_string(), parts: vec![Part { text: Some(q.clone()), function_call: None, function_response: None, extra: HashMap::new() }] });
        }
        if !a.is_empty() {
            messages.push(Content { role: "model".to_string(), parts: vec![Part { text: Some(a.clone()), function_call: None, function_response: None, extra: HashMap::new() }] });
        }
    }

    let language = config::get("LANGUAGE")?.unwrap_or_else(|| "en".to_string());
    let mut system_instruction = config::get("SYSTEM_PROMPT")?.unwrap_or_else(|| "Expert Rust Developer.".to_string());

    system_instruction.push_str("\n\nCORE MANDATE: Use 'semantic_search' for code discovery. If truncated, use 'line_start' to read more. YOU MUST USE 'write_file' TO SAVE REPORTS.");
    system_instruction.push_str(&format!("\n\nLanguage: {}. Text thoughts must be separate from calls.", language));

    let user_prompt = format!("Context: {:?}\n\nQuestion: {}", input.context, input.prompt);
    messages.push(Content { role: "user".to_string(), parts: vec![Part { text: Some(user_prompt), function_call: None, function_response: None, extra: HashMap::new() }] });

    let mut current_iteration = 0;
    let mut last_total_tokens = 0u64;
    const MAX_ITERATIONS: i32 = 100;
    let mut trace_log = format!("--- TRACE ---\nPrompt: {}\n\n", input.prompt);

    loop {
        current_iteration += 1;
        if current_iteration > MAX_ITERATIONS { break; }

        let history_window = if messages.len() > 30 {
            let mut window = vec![messages[0].clone()];
            let mut start = messages.len() - 29;
            while start > 1 && messages[start].role == "user" && messages[start].parts.iter().any(|p| p.function_response.is_some()) { start -= 1; }
            window.extend(messages[start..].to_vec());
            window
        } else {
            messages.clone()
        };

        let req = GeminiRequest {
            system_instruction: Some(Content { role: "system".to_string(), parts: vec![Part { text: Some(system_instruction.clone()), function_call: None, function_response: None, extra: HashMap::new() }] }),
            contents: history_window,
            tools: tools.clone(),
        };

        let body = serde_json::to_string(&req)?;
        if let Ok(pretty) = serde_json::to_string_pretty(&req) { let _ = unsafe { log_payload(pretty) }; }

        let http_req = HttpRequest::new(&url).with_method("POST").with_header("Content-Type", "application/json").with_header("x-goog-api-key", &api_key);
        let resp = http::request(&http_req, Some(body))?;
        if resp.status() != 200 && resp.status() != 0 { return Err(anyhow::anyhow!("API Error {}: {}", resp.status(), String::from_utf8_lossy(&resp.body())).into()); }

        let gemini_resp: GeminiResponse = serde_json::from_slice(&resp.body())?;
        if let Some(usage) = &gemini_resp.usage_metadata {
            last_total_tokens = usage.total_token_count as u64;
            let _ = unsafe { log_monologue(format!("Step {}: {} tokens", current_iteration, usage.total_token_count)) };
        }

        let candidate = gemini_resp.candidates.unwrap_or_default().get(0).cloned().ok_or(anyhow::anyhow!("No candidate"))?;
        let mut model_text = Vec::new();
        let mut model_calls = Vec::new();

        for part in &candidate.content.parts {
            if let Some(t) = &part.text { if !t.trim().is_empty() { 
                model_text.push(part.clone()); 
                let _ = unsafe { log_monologue(format!("> {}", t)) }; 
                trace_log.push_str(t); 
            } }
            if part.function_call.is_some() { model_calls.push(part.clone()); }
        }

        if !model_calls.is_empty() {
            if !model_text.is_empty() { messages.push(Content { role: "model".to_string(), parts: model_text }); }
            messages.push(Content { role: "model".to_string(), parts: model_calls.clone() });

            let mut response_parts = Vec::new();
            for part in &model_calls {
                let call = part.function_call.as_ref().unwrap();
                let name = call.name.strip_prefix("default_api:").unwrap_or(&call.name);
                trace_log.push_str(&format!("\nCall: {} {:?}\n", name, call.args));

                // MANDATORY: Function response MUST be an object {}, NOT an array []
                let result = if name == "read_project_file" {
                    match unsafe { read_project_file(serde_json::to_string(&call.args)?) } {
                        Ok(res) => serde_json::json!({ "content": res }),
                        Err(e) => serde_json::json!({ "error": format!("Read failed: {}", e) }),
                    }
                } else if name == "write_file" {
                    match unsafe { write_project_file(serde_json::to_string(&call.args)?) } {
                        Ok(_) => serde_json::json!({ "status": "success" }),
                        Err(e) => serde_json::json!({ "error": format!("Write failed: {}", e) }),
                    }
                } else if name == "semantic_search" || name == "search_project" {
                    let q = call.args["query"].as_str().unwrap_or("");
                    let res_result = if name == "semantic_search" { unsafe { semantic_search(q.to_string()) } } else { unsafe { search_project(q.to_string()) } };
                    match res_result {
                        Ok(res_str) => {
                            let res_json: serde_json::Value = serde_json::from_str(&res_str).unwrap_or(serde_json::json!([]));
                            serde_json::json!({ "results": res_json }) // Wrap array in object!
                        },
                        Err(e) => serde_json::json!({ "error": format!("Search failed: {}", e) }),
                    }
                } else if name == "list_project_files" {
                    match unsafe { list_project_files() } {
                        Ok(res_str) => serde_json::from_str(&res_str).unwrap_or(serde_json::json!({"error": "invalid host response"})),
                        Err(e) => serde_json::json!({ "error": format!("Listing failed: {}", e) }),
                    }
                } else if name == "exec_in_sandbox" {
                    match unsafe { exec_in_sandbox(call.args["command"].as_str().unwrap_or("").to_string()) } {
                        Ok(res) => serde_json::json!({ "output": res }),
                        Err(e) => serde_json::json!({ "error": format!("Execution failed: {}", e) }),
                    }
                } else { serde_json::json!({"error": "unknown function"}) };

                response_parts.push(Part { text: None, function_call: None, function_response: Some(FunctionResponse { name: call.name.clone(), response: result }), extra: HashMap::new() });
            }
            messages.push(Content { role: "user".to_string(), parts: response_parts });
        } else {
            let _ = unsafe { log_usage(last_total_tokens) };
            let ans = candidate.content.parts.iter().find_map(|p| p.text.clone()).unwrap_or_default();
            // Trace log is best-effort, don't fail the whole request if it fails
            let _ = unsafe { 
                let log_input = serde_json::json!({"path": ".gemini_trace.log", "content": trace_log});
                if let Ok(json_str) = serde_json::to_string(&log_input) {
                    let _ = write_project_file(json_str);
                }
            };
            return Ok(ans);
        }
    }
    Ok("Exceeded depth".to_string())
}
