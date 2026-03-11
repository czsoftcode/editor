use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::PathBuf;

use serde_json::Value;

use super::audit::AuditLogger;
use super::security::{
    CommandBlacklist, CommandClassification, FileBlacklist, PathSandbox, RateLimiter, SecretsFilter,
};

// ---------------------------------------------------------------------------
// ToolResult
// ---------------------------------------------------------------------------

/// Decision made by user on a tool approval dialog.
#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalDecision {
    /// Execute the tool call this time.
    Approve,
    /// Deny the tool call.
    Deny,
    /// Execute and auto-approve this tool in the future.
    Always,
}

/// Result of executing (or pre-evaluating) a tool call.
#[derive(Debug, Clone)]
pub enum ToolResult {
    /// Auto-approved, already executed, here's the output.
    Success(String),
    /// Needs user approval before execution -- contains preview info.
    NeedsApproval {
        tool_name: String,
        description: String,
        details: String,
        is_network: bool,
        is_new_file: bool,
    },
    /// Special: ask_user -- needs UI input, not a standard approval.
    AskUser {
        question: String,
        options: Vec<String>,
    },
    /// Special: announce_completion -- task done signal.
    Completion {
        summary: String,
        files_modified: Vec<String>,
        follow_up: Option<String>,
    },
    /// Error (blocked, rate limited, etc.)
    Error(String),
}

// ---------------------------------------------------------------------------
// ToolExecutor
// ---------------------------------------------------------------------------

/// Native tool executor that dispatches tool calls to handler functions
/// with security, approval, and audit logging.
pub struct ToolExecutor {
    project_root: PathBuf,
    sandbox: PathSandbox,
    file_blacklist: FileBlacklist,
    rate_limiter: RateLimiter,
    audit: AuditLogger,
    scratch: HashMap<String, String>,
    facts: HashMap<String, String>,
    facts_path: PathBuf,
    auto_approved: HashSet<String>,
}

impl ToolExecutor {
    /// Creates a new ToolExecutor with all security subsystems initialized.
    pub fn new(
        project_root: PathBuf,
        user_blacklist_patterns: Option<Vec<String>>,
        gitignore_patterns: Option<Vec<String>>,
    ) -> Self {
        let sandbox = PathSandbox::new(project_root.clone());
        let file_blacklist = FileBlacklist::new(user_blacklist_patterns, gitignore_patterns);
        let rate_limiter = RateLimiter::new();
        let audit = AuditLogger::new(project_root.clone());
        let facts_path = project_root.join(".polycredo").join("ai-facts.json");

        // Load existing facts from disk
        let facts = Self::load_facts(&facts_path);

        let auto_approved: HashSet<String> = [
            "read_project_file",
            "list_project_files",
            "search_project",
            "semantic_search",
            "store_scratch",
            "retrieve_scratch",
            "store_fact",
            "retrieve_fact",
            "list_facts",
            "delete_fact",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            project_root,
            sandbox,
            file_blacklist,
            rate_limiter,
            audit,
            scratch: HashMap::new(),
            facts,
            facts_path,
            auto_approved,
        }
    }

    /// Dispatches a tool call. Auto-approved tools execute immediately.
    /// Approval-required tools return NeedsApproval with preview.
    /// Special tools return AskUser/Completion variants.
    pub fn execute(&mut self, tool_name: &str, args: &Value) -> ToolResult {
        // Special tools first (no security checks needed)
        match tool_name {
            "ask_user" => return self.handle_ask_user(args),
            "announce_completion" => return self.handle_announce_completion(args),
            _ => {}
        }

        let is_auto = self.auto_approved.contains(tool_name);

        let result = match tool_name {
            // Auto-approved file tools
            "read_project_file" => self.handle_read_file(args),
            "list_project_files" => self.handle_list_files(),
            "search_project" => self.handle_search(args),
            "semantic_search" => self.handle_semantic_search(),

            // Auto-approved scratch/fact tools
            "store_scratch" => self.handle_store_scratch(args),
            "retrieve_scratch" => self.handle_retrieve_scratch(args),
            "store_fact" => self.handle_store_fact(args),
            "retrieve_fact" => self.handle_retrieve_fact(args),
            "list_facts" => self.handle_list_facts(),
            "delete_fact" => self.handle_delete_fact(args),

            // Approval-required tools
            "write_file" => self.handle_write_file(args),
            "replace" => self.handle_replace(args),
            "exec" => self.handle_exec(args),

            _ => ToolResult::Error(format!("Unknown tool: {}", tool_name)),
        };

        // Audit log
        let status = match &result {
            ToolResult::Success(_) => "auto_approved",
            ToolResult::NeedsApproval { .. } => "needs_approval",
            ToolResult::Error(_) => "denied",
            ToolResult::AskUser { .. } => "ask_user",
            ToolResult::Completion { .. } => "completion",
        };
        self.audit
            .log_tool_call(tool_name, status, &format!("args={}", args));

        // Scrub secrets from success output
        if is_auto && let ToolResult::Success(ref output) = result {
            return ToolResult::Success(SecretsFilter::scrub(output));
        }

        result
    }

    /// Called after user approves -- actually performs write_file, replace, or exec.
    pub fn execute_approved(&mut self, tool_name: &str, args: &Value) -> ToolResult {
        let result = match tool_name {
            "write_file" => self.execute_write_approved(args),
            "replace" => self.execute_replace_approved(args),
            "exec" => self.execute_exec_approved(args),
            _ => ToolResult::Error(format!(
                "Tool '{}' does not need approval or is unknown",
                tool_name
            )),
        };

        let status = match &result {
            ToolResult::Success(_) => "approved",
            ToolResult::Error(_) => "approved_error",
            _ => "approved",
        };
        self.audit
            .log_tool_call(tool_name, status, &format!("args={}", args));

        // Scrub secrets from output
        if let ToolResult::Success(ref output) = result {
            return ToolResult::Success(SecretsFilter::scrub(output));
        }

        result
    }

    /// Processes an approval decision for a tool call that required approval.
    /// Centralizes approve/deny/always logic for testability.
    pub fn process_approval_response(
        &mut self,
        tool_name: &str,
        args: &Value,
        decision: ApprovalDecision,
    ) -> ToolResult {
        match decision {
            ApprovalDecision::Approve => {
                self.audit
                    .log_tool_call(tool_name, "user_approved", &format!("args={}", args));
                self.execute_approved(tool_name, args)
            }
            ApprovalDecision::Deny => {
                self.audit
                    .log_tool_call(tool_name, "user_denied", &format!("args={}", args));
                ToolResult::Error(format!("Tool '{}' call denied by user", tool_name))
            }
            ApprovalDecision::Always => {
                self.auto_approved.insert(tool_name.to_string());
                self.audit.log_tool_call(
                    tool_name,
                    "user_always_approved",
                    &format!("args={}", args),
                );
                self.execute_approved(tool_name, args)
            }
        }
    }

    /// Returns true if a tool is in the always-approved set (runtime, set via "Always" button).
    pub fn check_always_approved(&self, tool_name: &str) -> bool {
        self.auto_approved.contains(tool_name)
    }

    /// Resets scratch memory (called on new conversation start).
    pub fn reset_scratch(&mut self) {
        self.scratch.clear();
    }

    /// Builds the pair of conversation messages needed to resume the AI after tool execution.
    /// Returns (assistant tool_call message, tool result message).
    pub fn build_approval_messages(
        tool_call_id: &str,
        tool_name: &str,
        args: &Value,
        result_content: &str,
        is_error: bool,
    ) -> (super::types::AiMessage, super::types::AiMessage) {
        let assistant_msg = super::types::AiMessage {
            role: "assistant".to_string(),
            content: String::new(),
            monologue: Vec::new(),
            timestamp: 0,
            tool_call_name: Some(tool_name.to_string()),
            tool_call_id: Some(tool_call_id.to_string()),
            tool_result_for_id: None,
            tool_is_error: false,
            tool_call_arguments: Some(args.clone()),
        };

        let tool_result_msg = super::types::AiMessage {
            role: "tool".to_string(),
            content: result_content.to_string(),
            monologue: Vec::new(),
            timestamp: 0,
            tool_call_name: None,
            tool_call_id: None,
            tool_result_for_id: Some(tool_call_id.to_string()),
            tool_is_error: is_error,
            tool_call_arguments: None,
        };

        (assistant_msg, tool_result_msg)
    }

    // -----------------------------------------------------------------------
    // File tool handlers
    // -----------------------------------------------------------------------

    fn handle_read_file(&self, args: &Value) -> ToolResult {
        let path_str = match args["path"].as_str() {
            Some(p) => p,
            None => return ToolResult::Error("Missing 'path' argument".to_string()),
        };

        // Blacklist check
        if self.file_blacklist.is_blocked(path_str) {
            self.audit
                .log_security_event("FILE_BLOCKED", &format!("path={}", path_str));
            return ToolResult::Error(format!("Access to '{}' is blocked (blacklisted)", path_str));
        }

        // Sandbox check
        let full_path = match self.sandbox.validate_path(path_str) {
            Ok(p) => p,
            Err(e) => {
                self.audit
                    .log_security_event("PATH_TRAVERSAL", &format!("path={}: {}", path_str, e));
                return ToolResult::Error(e);
            }
        };

        // Read file
        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(e) => return ToolResult::Error(format!("Cannot read '{}': {}", path_str, e)),
        };

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let line_start = args["line_start"].as_u64().unwrap_or(1) as usize;
        let line_end = args["line_end"].as_u64().map(|v| v as usize);

        let output = if line_start > 1 || line_end.is_some() {
            let start = if line_start > 1 {
                (line_start - 1).min(total_lines)
            } else {
                0
            };
            let end = line_end.map(|e| e.min(total_lines)).unwrap_or(total_lines);
            lines[start..end].join("\n")
        } else {
            content
        };

        // Truncate large output (first 500 lines)
        let output_lines: Vec<&str> = output.lines().collect();
        if output_lines.len() > 500 {
            let truncated = output_lines[..500].join("\n");
            ToolResult::Success(format!(
                "{}\n\n[Truncated: showing 500/{} lines. Use line_start/line_end to read more.]",
                truncated, total_lines
            ))
        } else {
            ToolResult::Success(output)
        }
    }

    fn handle_write_file(&mut self, args: &Value) -> ToolResult {
        let path_str = match args["path"].as_str() {
            Some(p) => p,
            None => return ToolResult::Error("Missing 'path' argument".to_string()),
        };
        let content = match args["content"].as_str() {
            Some(c) => c,
            None => return ToolResult::Error("Missing 'content' argument".to_string()),
        };

        // Blacklist check
        if self.file_blacklist.is_blocked(path_str) {
            self.audit
                .log_security_event("FILE_BLOCKED", &format!("write path={}", path_str));
            return ToolResult::Error(format!("Access to '{}' is blocked (blacklisted)", path_str));
        }

        // Sandbox check
        match self.sandbox.validate_path(path_str) {
            Ok(_) => {}
            Err(e) => {
                self.audit.log_security_event(
                    "PATH_TRAVERSAL",
                    &format!("write path={}: {}", path_str, e),
                );
                return ToolResult::Error(e);
            }
        }

        // Rate limit
        if let Err(e) = self.rate_limiter.check_write() {
            return ToolResult::Error(e);
        }

        // Check if file exists
        let full_path = self.project_root.join(path_str);
        let is_new = !full_path.exists();

        if !is_new {
            return ToolResult::Error(
                "File already exists. Use 'replace' tool to modify existing files.".to_string(),
            );
        }

        // Preview
        let preview = if content.len() > 200 {
            format!("{}...\n[{} bytes total]", &content[..200], content.len())
        } else {
            content.to_string()
        };

        ToolResult::NeedsApproval {
            tool_name: "write_file".to_string(),
            description: format!("Create new file: {}", path_str),
            details: preview,
            is_network: false,
            is_new_file: true,
        }
    }

    fn execute_write_approved(&mut self, args: &Value) -> ToolResult {
        let path_str = match args["path"].as_str() {
            Some(p) => p,
            None => return ToolResult::Error("Missing 'path' argument".to_string()),
        };
        let content = match args["content"].as_str() {
            Some(c) => c,
            None => return ToolResult::Error("Missing 'content' argument".to_string()),
        };

        if self.file_blacklist.is_blocked(path_str) {
            self.audit
                .log_security_event("FILE_BLOCKED", &format!("write path={}", path_str));
            return ToolResult::Error(format!("Access to '{}' is blocked (blacklisted)", path_str));
        }

        let full_path = match self.sandbox.validate_path(path_str) {
            Ok(p) => p,
            Err(e) => {
                self.audit.log_security_event(
                    "PATH_TRAVERSAL",
                    &format!("write path={}: {}", path_str, e),
                );
                return ToolResult::Error(e);
            }
        };

        if let Err(e) = self.rate_limiter.check_write() {
            self.audit
                .log_security_event("RATE_LIMIT_WRITE", &format!("path={}: {}", path_str, e));
            return ToolResult::Error(e);
        }

        if full_path.exists() {
            return ToolResult::Error(
                "File already exists. Use 'replace' tool to modify existing files.".to_string(),
            );
        }

        if let Some(parent) = full_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        match std::fs::write(&full_path, content) {
            Ok(_) => ToolResult::Success(format!(
                "File '{}' created ({} bytes)",
                path_str,
                content.len()
            )),
            Err(e) => ToolResult::Error(format!("Failed to write '{}': {}", path_str, e)),
        }
    }

    fn handle_replace(&mut self, args: &Value) -> ToolResult {
        let path_str = match args["path"].as_str() {
            Some(p) => p,
            None => return ToolResult::Error("Missing 'path' argument".to_string()),
        };
        let old_string = match args["old_string"].as_str() {
            Some(s) => s,
            None => return ToolResult::Error("Missing 'old_string' argument".to_string()),
        };
        let new_string = match args["new_string"].as_str() {
            Some(s) => s,
            None => return ToolResult::Error("Missing 'new_string' argument".to_string()),
        };

        // Blacklist check
        if self.file_blacklist.is_blocked(path_str) {
            self.audit
                .log_security_event("FILE_BLOCKED", &format!("replace path={}", path_str));
            return ToolResult::Error(format!("Access to '{}' is blocked (blacklisted)", path_str));
        }

        // Sandbox check
        let full_path = match self.sandbox.validate_path(path_str) {
            Ok(p) => p,
            Err(e) => {
                self.audit.log_security_event(
                    "PATH_TRAVERSAL",
                    &format!("replace path={}: {}", path_str, e),
                );
                return ToolResult::Error(e);
            }
        };

        // Rate limit
        if let Err(e) = self.rate_limiter.check_write() {
            return ToolResult::Error(e);
        }

        // Read current content
        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(e) => return ToolResult::Error(format!("Cannot read '{}': {}", path_str, e)),
        };

        // Find match (exact, then fuzzy)
        let match_range = if let Some(pos) = content.find(old_string) {
            Some(pos..pos + old_string.len())
        } else {
            find_fuzzy_match(&content, old_string)
        };

        if match_range.is_none() {
            return ToolResult::Error(format!(
                "'old_string' not found in '{}' (even with fuzzy matching)",
                path_str
            ));
        }

        let range = match_range.unwrap();
        let matched_text = &content[range.clone()];

        // Generate diff preview
        let new_content = format!(
            "{}{}{}",
            &content[..range.start],
            new_string,
            &content[range.end..]
        );
        let diff = generate_unified_diff(&content, &new_content, path_str);

        ToolResult::NeedsApproval {
            tool_name: "replace".to_string(),
            description: format!(
                "Replace text in {}: {} chars -> {} chars",
                path_str,
                matched_text.len(),
                new_string.len()
            ),
            details: diff,
            is_network: false,
            is_new_file: false,
        }
    }

    fn execute_replace_approved(&mut self, args: &Value) -> ToolResult {
        let path_str = match args["path"].as_str() {
            Some(p) => p,
            None => return ToolResult::Error("Missing 'path' argument".to_string()),
        };
        let old_string = match args["old_string"].as_str() {
            Some(s) => s,
            None => return ToolResult::Error("Missing 'old_string' argument".to_string()),
        };
        let new_string = match args["new_string"].as_str() {
            Some(s) => s,
            None => return ToolResult::Error("Missing 'new_string' argument".to_string()),
        };

        if self.file_blacklist.is_blocked(path_str) {
            self.audit
                .log_security_event("FILE_BLOCKED", &format!("replace path={}", path_str));
            return ToolResult::Error(format!("Access to '{}' is blocked (blacklisted)", path_str));
        }

        let full_path = match self.sandbox.validate_path(path_str) {
            Ok(p) => p,
            Err(e) => {
                self.audit.log_security_event(
                    "PATH_TRAVERSAL",
                    &format!("replace path={}: {}", path_str, e),
                );
                return ToolResult::Error(e);
            }
        };

        if let Err(e) = self.rate_limiter.check_write() {
            self.audit
                .log_security_event("RATE_LIMIT_WRITE", &format!("path={}: {}", path_str, e));
            return ToolResult::Error(e);
        };

        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(e) => return ToolResult::Error(format!("Cannot read '{}': {}", path_str, e)),
        };

        let match_range = if let Some(pos) = content.find(old_string) {
            Some(pos..pos + old_string.len())
        } else {
            find_fuzzy_match(&content, old_string)
        };

        let Some(range) = match_range else {
            return ToolResult::Error(format!("'old_string' not found in '{}'", path_str));
        };

        let new_content = format!(
            "{}{}{}",
            &content[..range.start],
            new_string,
            &content[range.end..]
        );

        match std::fs::write(&full_path, &new_content) {
            Ok(_) => ToolResult::Success(format!("Replaced text in '{}'", path_str)),
            Err(e) => ToolResult::Error(format!("Failed to write '{}': {}", path_str, e)),
        }
    }

    fn handle_list_files(&self) -> ToolResult {
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !matches!(
                    name.as_ref(),
                    "target" | ".git" | "node_modules" | "vendor" | ".polycredo"
                )
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && let Ok(rel) = entry.path().strip_prefix(&self.project_root)
            {
                let rel_str = rel.to_string_lossy().into_owned();
                if !self.file_blacklist.is_blocked(&rel_str) {
                    files.push(rel_str);
                }
            }
        }

        let total = files.len();
        if total > 300 {
            files.truncate(300);
        }

        let result = serde_json::json!({
            "files": files,
            "total_count": total,
            "truncated": total > 300,
        });

        ToolResult::Success(result.to_string())
    }

    fn handle_search(&self, args: &Value) -> ToolResult {
        let query = match args["query"].as_str() {
            Some(q) => q,
            None => return ToolResult::Error("Missing 'query' argument".to_string()),
        };

        let mut results = Vec::new();
        let max_results = 50;

        for entry in walkdir::WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !matches!(
                    name.as_ref(),
                    "target" | ".git" | "node_modules" | "vendor" | ".polycredo"
                )
            })
            .filter_map(|e| e.ok())
        {
            if results.len() >= max_results {
                break;
            }

            if !entry.file_type().is_file() {
                continue;
            }

            let Ok(rel) = entry.path().strip_prefix(&self.project_root) else {
                continue;
            };
            let rel_str = rel.to_string_lossy().into_owned();

            if self.file_blacklist.is_blocked(&rel_str) {
                continue;
            }

            let Ok(content) = std::fs::read_to_string(entry.path()) else {
                continue;
            };

            for (line_num, line) in content.lines().enumerate() {
                if results.len() >= max_results {
                    break;
                }
                if line.contains(query) {
                    results.push(format!("{}:{}: {}", rel_str, line_num + 1, line.trim()));
                }
            }
        }

        if results.is_empty() {
            ToolResult::Success(format!("No matches found for '{}'", query))
        } else {
            ToolResult::Success(results.join("\n"))
        }
    }

    fn handle_semantic_search(&self) -> ToolResult {
        ToolResult::Success(
            "Semantic search not yet implemented in native mode. Use 'search_project' for text search."
                .to_string(),
        )
    }

    // -----------------------------------------------------------------------
    // Scratch handlers
    // -----------------------------------------------------------------------

    fn handle_store_scratch(&mut self, args: &Value) -> ToolResult {
        let key = match args["key"].as_str() {
            Some(k) => k.to_string(),
            None => return ToolResult::Error("Missing 'key' argument".to_string()),
        };
        let value = match args["value"].as_str() {
            Some(v) => v.to_string(),
            None => return ToolResult::Error("Missing 'value' argument".to_string()),
        };
        self.scratch.insert(key.clone(), value);
        ToolResult::Success(format!("Stored scratch key '{}'", key))
    }

    fn handle_retrieve_scratch(&self, args: &Value) -> ToolResult {
        let key = match args["key"].as_str() {
            Some(k) => k,
            None => return ToolResult::Error("Missing 'key' argument".to_string()),
        };
        let value = self.scratch.get(key).cloned().unwrap_or_default();
        ToolResult::Success(value)
    }

    // -----------------------------------------------------------------------
    // Fact handlers (persistent, file-based)
    // -----------------------------------------------------------------------

    fn handle_store_fact(&mut self, args: &Value) -> ToolResult {
        let key = match args["key"].as_str() {
            Some(k) => k.to_string(),
            None => return ToolResult::Error("Missing 'key' argument".to_string()),
        };
        let value = match args["value"].as_str() {
            Some(v) => v.to_string(),
            None => return ToolResult::Error("Missing 'value' argument".to_string()),
        };
        self.facts.insert(key.clone(), value);
        self.save_facts();
        ToolResult::Success(format!("Fact '{}' stored", key))
    }

    fn handle_retrieve_fact(&self, args: &Value) -> ToolResult {
        let key = match args["key"].as_str() {
            Some(k) => k,
            None => return ToolResult::Error("Missing 'key' argument".to_string()),
        };
        let value = self.facts.get(key).cloned().unwrap_or_default();
        ToolResult::Success(value)
    }

    fn handle_list_facts(&self) -> ToolResult {
        let keys: Vec<&String> = self.facts.keys().collect();
        let result = serde_json::json!({ "keys": keys });
        ToolResult::Success(result.to_string())
    }

    fn handle_delete_fact(&mut self, args: &Value) -> ToolResult {
        let key = match args["key"].as_str() {
            Some(k) => k.to_string(),
            None => return ToolResult::Error("Missing 'key' argument".to_string()),
        };
        self.facts.remove(&key);
        self.save_facts();
        ToolResult::Success(format!("Fact '{}' deleted", key))
    }

    fn load_facts(path: &PathBuf) -> HashMap<String, String> {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => HashMap::new(),
        }
    }

    fn save_facts(&self) {
        if let Some(parent) = self.facts_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.facts)
            && let Err(e) = std::fs::write(&self.facts_path, json)
        {
            eprintln!("[ToolExecutor] Failed to save facts: {}", e);
        }
    }

    // -----------------------------------------------------------------------
    // Special tool handlers
    // -----------------------------------------------------------------------

    fn handle_ask_user(&self, args: &Value) -> ToolResult {
        let question = args["question"].as_str().unwrap_or("?").to_string();
        let options: Vec<String> = args["options"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        ToolResult::AskUser { question, options }
    }

    fn handle_announce_completion(&self, args: &Value) -> ToolResult {
        let summary = args["summary"]
            .as_str()
            .unwrap_or("Task completed.")
            .to_string();
        let files_modified: Vec<String> = args["files_modified"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let follow_up = args["follow_up"].as_str().map(str::to_string);

        ToolResult::Completion {
            summary,
            files_modified,
            follow_up,
        }
    }

    // -----------------------------------------------------------------------
    // Exec handler
    // -----------------------------------------------------------------------

    fn handle_exec(&mut self, args: &Value) -> ToolResult {
        let command = match args["command"].as_str() {
            Some(c) => c,
            None => return ToolResult::Error("Missing 'command' argument".to_string()),
        };

        // Rate limit
        if let Err(e) = self.rate_limiter.check_exec() {
            return ToolResult::Error(e);
        }

        // Classify command
        match CommandBlacklist::classify(command) {
            CommandClassification::Blocked => {
                self.audit
                    .log_security_event("COMMAND_BLOCKED", &format!("cmd={}", command));
                ToolResult::Error(format!("Blocked: dangerous command '{}'", command))
            }
            CommandClassification::NetworkWarning => ToolResult::NeedsApproval {
                tool_name: "exec".to_string(),
                description: format!(
                    "Execute: {} (Sitovy prikaz -- data mohou opustit pocitac)",
                    command
                ),
                details: command.to_string(),
                is_network: true,
                is_new_file: false,
            },
            CommandClassification::NeedsApproval => ToolResult::NeedsApproval {
                tool_name: "exec".to_string(),
                description: format!("Execute: {}", command),
                details: command.to_string(),
                is_network: false,
                is_new_file: false,
            },
        }
    }

    fn execute_exec_approved(&mut self, args: &Value) -> ToolResult {
        let command = match args["command"].as_str() {
            Some(c) => c,
            None => return ToolResult::Error("Missing 'command' argument".to_string()),
        };

        if let Err(e) = self.rate_limiter.check_exec() {
            self.audit
                .log_security_event("RATE_LIMIT_EXEC", &format!("cmd={}: {}", command, e));
            return ToolResult::Error(e);
        }

        if matches!(
            CommandBlacklist::classify(command),
            CommandClassification::Blocked
        ) {
            self.audit
                .log_security_event("COMMAND_BLOCKED", &format!("cmd={}", command));
            return ToolResult::Error(format!("Blocked: dangerous command '{}'", command));
        }

        use std::process::Command;
        use std::sync::mpsc;
        use std::time::Duration;

        let child = match Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.project_root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return ToolResult::Error(format!("Failed to spawn command: {}", e)),
        };

        let (tx, rx) = mpsc::channel();
        let timeout = Duration::from_secs(120);

        std::thread::spawn(move || {
            let result = child.wait_with_output();
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let mut full_output = format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);

                // Truncate if too long
                if full_output.len() > 10000 {
                    let first = &full_output[..5000];
                    let last_start = full_output.len().saturating_sub(2000);
                    let last = &full_output[last_start..];
                    full_output = format!("{}\n[...truncated...]\n{}", first, last);
                }

                ToolResult::Success(full_output)
            }
            Ok(Err(e)) => ToolResult::Error(format!("Command failed: {}", e)),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                ToolResult::Error("Command timed out after 120 seconds".to_string())
            }
            Err(e) => ToolResult::Error(format!("Channel error: {}", e)),
        }
    }
}

// ---------------------------------------------------------------------------
// Fuzzy matching (extracted from host/fs.rs)
// ---------------------------------------------------------------------------

/// Normalizes a string for fuzzy comparison (removes all non-essential whitespace and empty lines).
fn normalize_for_fuzzy(s: &str) -> String {
    s.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Attempts to find a block in content that matches old_string semantically,
/// ignoring whitespace differences and empty lines.
fn find_fuzzy_match(content: &str, old_string: &str) -> Option<Range<usize>> {
    let normalized_old = normalize_for_fuzzy(old_string);
    if normalized_old.is_empty() {
        return None;
    }

    let old_lines: Vec<&str> = normalized_old.lines().collect();

    let content_lines: Vec<(usize, &str)> = content
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, l)| (i, l.trim()))
        .collect();

    if content_lines.len() < old_lines.len() {
        return None;
    }

    for i in 0..=content_lines.len() - old_lines.len() {
        let mut match_found = true;
        for j in 0..old_lines.len() {
            if content_lines[i + j].1 != old_lines[j] {
                match_found = false;
                break;
            }
        }

        if match_found {
            let mut line_byte_offsets = Vec::new();
            let mut offset = 0;
            for line in content.lines() {
                line_byte_offsets.push(offset);
                offset += line.len() + 1;
            }

            let start_line_idx = content_lines[i].0;
            let end_line_idx = content_lines[i + old_lines.len() - 1].0;

            if start_line_idx < line_byte_offsets.len() {
                let start_byte = line_byte_offsets[start_line_idx];
                let end_byte = if end_line_idx + 1 < line_byte_offsets.len() {
                    line_byte_offsets[end_line_idx + 1]
                } else {
                    content.len()
                };
                return Some(start_byte..end_byte);
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Unified diff generation
// ---------------------------------------------------------------------------

/// Generates a standard unified diff with 3 lines context using the `similar` crate.
pub fn generate_unified_diff(old_content: &str, new_content: &str, file_path: &str) -> String {
    use similar::TextDiff;

    let diff = TextDiff::from_lines(old_content, new_content);
    let mut output = String::new();

    output.push_str(&format!("--- a/{}\n", file_path));
    output.push_str(&format!("+++ b/{}\n", file_path));

    for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
        output.push_str(&format!("{}", hunk));
    }

    output
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_executor() -> (tempfile::TempDir, ToolExecutor) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        // Create some test files
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/main.rs"),
            "fn main() {\n    println!(\"Hello\");\n}\n",
        )
        .unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
        )
        .unwrap();
        fs::write(root.join(".env"), "API_KEY=secret123").unwrap();

        let executor = ToolExecutor::new(root, None, None);
        (tmp, executor)
    }

    // --- Read file tests ---

    #[test]
    fn read_file_valid() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute(
            "read_project_file",
            &serde_json::json!({"path": "src/main.rs"}),
        );
        match result {
            ToolResult::Success(content) => {
                assert!(content.contains("fn main()"));
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[test]
    fn read_file_path_traversal() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute(
            "read_project_file",
            &serde_json::json!({"path": "../../etc/passwd"}),
        );
        match result {
            ToolResult::Error(msg) => {
                assert!(
                    msg.contains("traversal") || msg.contains("outside"),
                    "Error should mention traversal: {}",
                    msg
                );
            }
            other => panic!("Expected Error, got {:?}", other),
        }
    }

    #[test]
    fn read_file_blacklisted() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute("read_project_file", &serde_json::json!({"path": ".env"}));
        match result {
            ToolResult::Error(msg) => {
                assert!(msg.contains("blocked") || msg.contains("blacklisted"));
            }
            other => panic!("Expected Error, got {:?}", other),
        }
    }

    #[test]
    fn read_file_secrets_scrubbed() {
        let (tmp, mut executor) = setup_test_executor();
        // Write a file with a secret
        fs::write(
            tmp.path().join("src/config.rs"),
            "let TOKEN = \"abc123\";\nlet x = 42;\n",
        )
        .unwrap();
        let result = executor.execute(
            "read_project_file",
            &serde_json::json!({"path": "src/config.rs"}),
        );
        match result {
            ToolResult::Success(content) => {
                assert!(content.contains("TOKEN=[REDACTED]"));
                assert!(!content.contains("abc123"));
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    // --- Write file tests ---

    #[test]
    fn write_file_new_needs_approval() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute(
            "write_file",
            &serde_json::json!({"path": "new_file.rs", "content": "fn new() {}"}),
        );
        match result {
            ToolResult::NeedsApproval {
                tool_name,
                is_new_file,
                ..
            } => {
                assert_eq!(tool_name, "write_file");
                assert!(is_new_file);
            }
            other => panic!("Expected NeedsApproval, got {:?}", other),
        }
    }

    #[test]
    fn write_file_existing_blocked() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute(
            "write_file",
            &serde_json::json!({"path": "src/main.rs", "content": "new content"}),
        );
        match result {
            ToolResult::Error(msg) => {
                assert!(msg.contains("replace"));
            }
            other => panic!("Expected Error, got {:?}", other),
        }
    }

    // --- Replace tests ---

    #[test]
    fn replace_needs_approval_with_diff() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute(
            "replace",
            &serde_json::json!({
                "path": "src/main.rs",
                "old_string": "fn main()",
                "new_string": "fn main_new()"
            }),
        );
        match result {
            ToolResult::NeedsApproval {
                tool_name, details, ..
            } => {
                assert_eq!(tool_name, "replace");
                assert!(details.contains("---") || details.contains("+++"));
            }
            other => panic!("Expected NeedsApproval, got {:?}", other),
        }
    }

    // --- List files tests ---

    #[test]
    fn list_project_files() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute("list_project_files", &serde_json::json!({}));
        match result {
            ToolResult::Success(content) => {
                assert!(content.contains("main.rs"));
                assert!(content.contains("lib.rs"));
                // .env should be filtered by blacklist
                assert!(!content.contains(".env"));
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    // --- Search tests ---

    #[test]
    fn search_project_finds_match() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute("search_project", &serde_json::json!({"query": "fn main"}));
        match result {
            ToolResult::Success(content) => {
                assert!(content.contains("fn main"));
                assert!(content.contains("main.rs"));
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    // --- Scratch tests ---

    #[test]
    fn scratch_store_and_retrieve() {
        let (_tmp, mut executor) = setup_test_executor();
        let store_result = executor.execute(
            "store_scratch",
            &serde_json::json!({"key": "k", "value": "v"}),
        );
        assert!(matches!(store_result, ToolResult::Success(_)));

        let retrieve_result =
            executor.execute("retrieve_scratch", &serde_json::json!({"key": "k"}));
        match retrieve_result {
            ToolResult::Success(val) => assert_eq!(val, "v"),
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    // --- Fact tests ---

    #[test]
    fn fact_store_retrieve_list_delete() {
        let (_tmp, mut executor) = setup_test_executor();

        // Store
        executor.execute(
            "store_fact",
            &serde_json::json!({"key": "project_lang", "value": "Rust"}),
        );

        // Retrieve
        let result = executor.execute("retrieve_fact", &serde_json::json!({"key": "project_lang"}));
        match result {
            ToolResult::Success(val) => assert_eq!(val, "Rust"),
            other => panic!("Expected Success, got {:?}", other),
        }

        // List
        let result = executor.execute("list_facts", &serde_json::json!({}));
        match result {
            ToolResult::Success(val) => assert!(val.contains("project_lang")),
            other => panic!("Expected Success, got {:?}", other),
        }

        // Delete
        executor.execute("delete_fact", &serde_json::json!({"key": "project_lang"}));

        let result = executor.execute("retrieve_fact", &serde_json::json!({"key": "project_lang"}));
        match result {
            ToolResult::Success(val) => assert_eq!(val, ""),
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    // --- Ask user and completion tests ---

    #[test]
    fn ask_user_returns_special_result() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute(
            "ask_user",
            &serde_json::json!({"question": "Continue?", "options": ["yes", "no"]}),
        );
        match result {
            ToolResult::AskUser { question, options } => {
                assert_eq!(question, "Continue?");
                assert_eq!(options, vec!["yes", "no"]);
            }
            other => panic!("Expected AskUser, got {:?}", other),
        }
    }

    #[test]
    fn announce_completion_returns_special_result() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute(
            "announce_completion",
            &serde_json::json!({
                "summary": "Done!",
                "files_modified": ["src/main.rs"],
                "follow_up": "Run tests"
            }),
        );
        match result {
            ToolResult::Completion {
                summary,
                files_modified,
                follow_up,
            } => {
                assert_eq!(summary, "Done!");
                assert_eq!(files_modified, vec!["src/main.rs"]);
                assert_eq!(follow_up, Some("Run tests".to_string()));
            }
            other => panic!("Expected Completion, got {:?}", other),
        }
    }

    // --- Audit logging tests ---

    #[test]
    fn all_calls_are_audit_logged() {
        let (tmp, mut executor) = setup_test_executor();
        executor.execute(
            "read_project_file",
            &serde_json::json!({"path": "src/main.rs"}),
        );
        executor.execute(
            "write_file",
            &serde_json::json!({"path": "new.rs", "content": "x"}),
        );

        let log_path = tmp.path().join(".polycredo").join("ai-audit.log");
        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("[read_project_file]"));
        assert!(content.contains("[write_file]"));
    }

    // --- Exec handler tests ---

    #[test]
    fn exec_cargo_build_needs_approval() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute("exec", &serde_json::json!({"command": "cargo build"}));
        match result {
            ToolResult::NeedsApproval { is_network, .. } => {
                assert!(!is_network);
            }
            other => panic!("Expected NeedsApproval, got {:?}", other),
        }
    }

    #[test]
    fn exec_rm_rf_blocked() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute("exec", &serde_json::json!({"command": "rm -rf /"}));
        match result {
            ToolResult::Error(msg) => {
                assert!(msg.contains("Blocked"));
            }
            other => panic!("Expected Error, got {:?}", other),
        }
    }

    #[test]
    fn exec_curl_is_network_warning() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute(
            "exec",
            &serde_json::json!({"command": "curl https://example.com"}),
        );
        match result {
            ToolResult::NeedsApproval {
                is_network,
                description,
                ..
            } => {
                assert!(is_network);
                assert!(description.contains("Sitovy prikaz"));
            }
            other => panic!("Expected NeedsApproval, got {:?}", other),
        }
    }

    #[test]
    fn exec_approved_runs_command() {
        let (_tmp, mut executor) = setup_test_executor();
        let result =
            executor.execute_approved("exec", &serde_json::json!({"command": "echo hello"}));
        match result {
            ToolResult::Success(output) => {
                assert!(output.contains("hello"));
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[test]
    fn exec_approved_uses_project_root() {
        let (tmp, mut executor) = setup_test_executor();
        let result = executor.execute_approved("exec", &serde_json::json!({"command": "pwd"}));
        match result {
            ToolResult::Success(output) => {
                let root_str = tmp.path().to_string_lossy();
                assert!(
                    output.contains(&*root_str),
                    "pwd should show project root, got: {}",
                    output
                );
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[test]
    fn exec_output_secrets_scrubbed() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute_approved(
            "exec",
            &serde_json::json!({"command": "echo 'TOKEN=mysecret123'"}),
        );
        match result {
            ToolResult::Success(output) => {
                assert!(output.contains("[REDACTED]"));
                assert!(!output.contains("mysecret123"));
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    // --- Diff generation tests ---

    #[test]
    fn unified_diff_basic() {
        let old = "line1\nline2\nline3\nline4\nline5\n";
        let new = "line1\nline2\nLINE3_MODIFIED\nline4\nline5\n";
        let diff = generate_unified_diff(old, new, "test.rs");
        assert!(diff.contains("--- a/test.rs"));
        assert!(diff.contains("+++ b/test.rs"));
        assert!(diff.contains("-line3"));
        assert!(diff.contains("+LINE3_MODIFIED"));
    }

    #[test]
    fn unified_diff_context_lines() {
        let old = "a\nb\nc\nd\ne\nf\ng\nh\n";
        let new = "a\nb\nc\nD_CHANGED\ne\nf\ng\nh\n";
        let diff = generate_unified_diff(old, new, "test.rs");
        // Should have context around the change
        assert!(diff.contains("@@"));
        assert!(diff.contains("-d"));
        assert!(diff.contains("+D_CHANGED"));
    }

    // --- Fuzzy matching tests ---

    #[test]
    fn fuzzy_match_exact() {
        let content = "fn main() {\n    println!(\"hello\");\n}\n";
        let search = "fn main() {\n    println!(\"hello\");\n}";
        let result = find_fuzzy_match(content, search);
        assert!(result.is_some());
    }

    #[test]
    fn fuzzy_match_whitespace_diff() {
        let content = "fn main() {\n    let x = 1;\n}\n";
        let search = "fn main() {\n  let x = 1;\n}";
        let result = find_fuzzy_match(content, search);
        assert!(result.is_some());
    }

    #[test]
    fn fuzzy_match_no_match() {
        let content = "fn main() {\n    println!(\"hello\");\n}\n";
        let search = "fn totally_different()";
        let result = find_fuzzy_match(content, search);
        assert!(result.is_none());
    }

    // --- Rate limit tests ---

    #[test]
    fn write_rate_limit_enforced() {
        let (_tmp, mut executor) = setup_test_executor();
        // Exhaust rate limit (50 writes)
        for _ in 0..50 {
            let _ = executor.rate_limiter.check_write();
        }
        // Next write should fail
        let result = executor.execute(
            "write_file",
            &serde_json::json!({"path": "test.rs", "content": "x"}),
        );
        match result {
            ToolResult::Error(msg) => {
                assert!(msg.contains("rate limit") || msg.contains("Rate limit"))
            }
            other => panic!("Expected Error, got {:?}", other),
        }
    }

    #[test]
    fn exec_rate_limit_enforced() {
        let (_tmp, mut executor) = setup_test_executor();
        for _ in 0..20 {
            let _ = executor.rate_limiter.check_exec();
        }
        let result = executor.execute("exec", &serde_json::json!({"command": "echo test"}));
        match result {
            ToolResult::Error(msg) => {
                assert!(msg.contains("rate limit") || msg.contains("Rate limit"))
            }
            other => panic!("Expected Error, got {:?}", other),
        }
    }

    #[test]
    fn security_approved_write_blocks_path_traversal() {
        let (tmp, mut executor) = setup_test_executor();
        let outside = tmp.path().join("..").join("pwned.rs");
        if outside.exists() {
            std::fs::remove_file(&outside).unwrap();
        }

        let result = executor.execute_approved(
            "write_file",
            &serde_json::json!({"path": "../pwned.rs", "content": "fn pwned() {}"}),
        );

        match result {
            ToolResult::Error(msg) => {
                assert!(
                    msg.contains("traversal") || msg.contains("outside"),
                    "Expected sandbox rejection, got: {}",
                    msg
                );
            }
            other => panic!("Expected Error, got {:?}", other),
        }
        assert!(
            !outside.exists(),
            "Traversal path must not be written outside project root"
        );
    }

    #[test]
    fn security_approved_write_respects_file_blacklist() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute_approved(
            "write_file",
            &serde_json::json!({"path": ".env.prod", "content": "TOKEN=leak"}),
        );

        match result {
            ToolResult::Error(msg) => assert!(msg.contains("blocked") || msg.contains("blacklist")),
            other => panic!("Expected Error, got {:?}", other),
        }
    }

    #[test]
    fn security_approved_write_respects_rate_limit() {
        let (_tmp, mut executor) = setup_test_executor();
        for _ in 0..50 {
            executor.rate_limiter.check_write().expect("rate setup");
        }

        let result = executor.execute_approved(
            "write_file",
            &serde_json::json!({"path": "limited.rs", "content": "fn limited() {}"}),
        );
        match result {
            ToolResult::Error(msg) => {
                assert!(msg.contains("rate limit") || msg.contains("Rate limit"))
            }
            other => panic!("Expected Error, got {:?}", other),
        }
    }

    #[test]
    fn security_approved_replace_respects_file_blacklist() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute_approved(
            "replace",
            &serde_json::json!({
                "path": ".env",
                "old_string": "secret123",
                "new_string": "redacted"
            }),
        );

        match result {
            ToolResult::Error(msg) => assert!(msg.contains("blocked") || msg.contains("blacklist")),
            other => panic!("Expected Error, got {:?}", other),
        }
    }

    #[test]
    fn security_approved_exec_still_blocks_dangerous_command() {
        let (_tmp, mut executor) = setup_test_executor();
        let result = executor.execute_approved("exec", &serde_json::json!({"command": "sudo "}));
        match result {
            ToolResult::Error(msg) => assert!(msg.contains("Blocked") || msg.contains("dangerous")),
            other => panic!("Expected Error, got {:?}", other),
        }
    }

    // --- Approval response tests (TOOL-05) ---

    #[test]
    fn test_approval_approve_executes_tool() {
        let (_tmp, mut executor) = setup_test_executor();
        let args = serde_json::json!({"path": "approved.rs", "content": "fn approved() {}"});
        let result =
            executor.process_approval_response("write_file", &args, ApprovalDecision::Approve);
        match result {
            ToolResult::Success(msg) => assert!(msg.contains("approved.rs")),
            other => panic!("Expected Success after approve, got {:?}", other),
        }
    }

    #[test]
    fn test_approval_deny_returns_error() {
        let (_tmp, mut executor) = setup_test_executor();
        let args = serde_json::json!({"command": "echo test"});
        let result = executor.process_approval_response("exec", &args, ApprovalDecision::Deny);
        match result {
            ToolResult::Error(msg) => assert!(msg.contains("denied by user")),
            other => panic!("Expected Error after deny, got {:?}", other),
        }
    }

    #[test]
    fn test_approval_deny_error_mentions_tool_name() {
        let (_tmp, mut executor) = setup_test_executor();
        let args = serde_json::json!({"path": "denied.rs", "content": "fn denied() {}"});
        let result =
            executor.process_approval_response("write_file", &args, ApprovalDecision::Deny);
        match result {
            ToolResult::Error(msg) => {
                assert!(
                    msg.contains("write_file"),
                    "Deny message should include tool name for resume context: {}",
                    msg
                );
            }
            other => panic!("Expected Error after deny, got {:?}", other),
        }
    }

    #[test]
    fn test_approval_always_adds_to_auto_approved() {
        let (_tmp, mut executor) = setup_test_executor();
        let args = serde_json::json!({"path": "always.rs", "content": "fn always() {}"});
        assert!(!executor.check_always_approved("write_file"));

        let result =
            executor.process_approval_response("write_file", &args, ApprovalDecision::Always);
        match result {
            ToolResult::Success(_) => {}
            other => panic!("Expected Success after always, got {:?}", other),
        }

        assert!(executor.check_always_approved("write_file"));
    }

    #[test]
    fn test_approval_already_always_approved_skips() {
        let (_tmp, mut executor) = setup_test_executor();
        // Manually add to auto_approved
        executor.auto_approved.insert("write_file".to_string());
        assert!(executor.check_always_approved("write_file"));
    }

    #[test]
    fn test_approval_execute_error_propagates() {
        let (_tmp, mut executor) = setup_test_executor();
        // Try to approve an unknown tool — should get error from execute_approved
        let args = serde_json::json!({});
        let result =
            executor.process_approval_response("unknown_tool", &args, ApprovalDecision::Approve);
        match result {
            ToolResult::Error(msg) => assert!(msg.contains("unknown") || msg.contains("Unknown")),
            other => panic!("Expected Error for unknown tool, got {:?}", other),
        }
    }

    #[test]
    fn test_build_approval_messages_success() {
        let (assistant, tool_result) = ToolExecutor::build_approval_messages(
            "tc_1",
            "read_project_file",
            &serde_json::json!({"path": "src/main.rs"}),
            "fn main() {}",
            false,
        );

        assert_eq!(assistant.role, "assistant");
        assert_eq!(
            assistant.tool_call_name.as_deref(),
            Some("read_project_file")
        );
        assert_eq!(assistant.tool_call_id.as_deref(), Some("tc_1"));
        assert!(assistant.tool_call_arguments.is_some());

        assert_eq!(tool_result.role, "tool");
        assert_eq!(tool_result.tool_result_for_id.as_deref(), Some("tc_1"));
        assert_eq!(tool_result.content, "fn main() {}");
        assert!(!tool_result.tool_is_error);
    }

    #[test]
    fn test_build_approval_messages_error() {
        let (_, tool_result) = ToolExecutor::build_approval_messages(
            "tc_2",
            "exec",
            &serde_json::json!({"command": "failing"}),
            "Command failed",
            true,
        );

        assert!(tool_result.tool_is_error);
        assert_eq!(tool_result.content, "Command failed");
    }

    #[test]
    fn test_reset_scratch() {
        let (_tmp, mut executor) = setup_test_executor();
        executor.execute(
            "store_scratch",
            &serde_json::json!({"key": "k", "value": "v"}),
        );
        executor.reset_scratch();
        let result = executor.execute("retrieve_scratch", &serde_json::json!({"key": "k"}));
        match result {
            ToolResult::Success(val) => assert_eq!(val, ""),
            other => panic!("Expected empty Success after reset, got {:?}", other),
        }
    }
}
