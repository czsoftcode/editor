use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

// --- Host Functions ---
#[host_fn]
extern "ExtismHost" {
    fn read_project_file(path: String) -> String;
    fn list_project_files() -> String;
    fn get_active_file_path() -> String;
    fn get_active_file_content() -> String;
}

#[plugin_fn]
pub fn ask_gemini(input: String) -> FnResult<String> {
    let api_key = config::get("API_KEY")?.ok_or(anyhow::anyhow!("Missing API_KEY in plugin settings"))?;
    let model = config::get("MODEL")?.unwrap_or_else(|| "gemini-1.5-flash".to_string());
    
    // 1. Gather Context from Host
    let active_path = unsafe { get_active_file_path()? };
    let active_content = unsafe { get_active_file_content()? };
    let file_list = unsafe { list_project_files()? };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );

    // 2. Build Rich Prompt
    let system_instruction = "You are a senior developer assistant integrated into the PolyCredo Editor. 
    You have access to the user's files. Provide concise, expert advice. 
    If the user asks about the project, refer to the provided context.";

    let mut context_info = format!("Project structure:\n{}\n\n", file_list);
    if !active_path.is_empty() {
        context_info.push_str(&format!("Current active file ({}):\n```\n{}\n```\n", active_path, active_content));
    }

    let full_prompt = format!("Context:\n{}\n\nUser Question: {}", context_info, input);

    let req = GeminiRequest {
        system_instruction: Some(Content {
            parts: vec![Part { text: system_instruction.to_string() }],
        }),
        contents: vec![Content {
            parts: vec![Part { text: full_prompt }],
        }],
    };

    let body = serde_json::to_string(&req)?;
    
    let http_req = HttpRequest::new(url)
        .with_method("POST")
        .with_header("Content-Type", "application/json")
        .with_header("x-goog-api-key", api_key);

    let resp = http::request(&http_req, Some(body))?;
    
    if resp.status() != 200 && resp.status() != 0 {
        let body_bytes = resp.body();
        let error_body = String::from_utf8_lossy(&body_bytes);
        return Err(anyhow::anyhow!("Gemini API error ({}): {}", resp.status(), error_body).into());
    }

    let body_bytes = resp.body();
    let gemini_resp: GeminiResponse = serde_json::from_slice(&body_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to parse Gemini response: {}. Body: {}", e, String::from_utf8_lossy(&body_bytes)))?;
    
    let answer = gemini_resp.candidates.first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.clone())
        .unwrap_or_else(|| "No response from Gemini".to_string());

    Ok(answer)
}
