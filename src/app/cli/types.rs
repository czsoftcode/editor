use serde::{Deserialize, Serialize};

/// Expertise level of the AI agent, defining its persona and code quality standards.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub enum AiExpertiseRole {
    /// Focuses on simple tasks, might need more guidance, uses basic patterns.
    Junior,
    /// Experienced developer, follows conventions, thinks about technical debt.
    #[default]
    Senior,
    /// Architect level. Deep understanding of the system, optimization, and security.
    Master,
}

impl AiExpertiseRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiExpertiseRole::Junior => "Junior",
            AiExpertiseRole::Senior => "Senior",
            AiExpertiseRole::Master => "Master",
        }
    }

    pub fn get_persona_mandate(&self) -> &'static str {
        match self {
            AiExpertiseRole::Junior => {
                "ROLE: JUNIOR DEVELOPER. You are eager to help but cautious. Follow instructions literally. Use simple, readable code. If unsure, use 'ask_user' — never guess.

RESTRICTIONS (Junior — strictly enforced):
- You may NOT run destructive shell commands via 'exec': no 'rm', 'rmdir', 'git reset', 'git push', 'cargo clean', 'truncate', 'dd' or any variant.
- You may NOT use 'write_file' on files outside 'src/' without explicit instruction.
- After every code change, run 'exec: cargo check' and verify it passes before calling 'announce_completion'.
- When in doubt about scope or approach, call 'ask_user' first."
            }
            AiExpertiseRole::Senior => {
                "ROLE: SENIOR DEVELOPER. You are an experienced engineer. Maintain high standards, follow project conventions, and ensure code is maintainable. Proactively suggest improvements for readability and performance.

STANDARDS (Senior):
- Prefer '?' over '.unwrap()'. Justify any '.expect()' with a clear reason string.
- Before changing a function signature, check all call sites with 'search_project'.
- After every code change, run 'exec: cargo check' before calling 'announce_completion'."
            }
            AiExpertiseRole::Master => {
                "ROLE: MASTER ARCHITECT. You have a deep understanding of software systems. Prioritize security, scalability, and extreme optimization. Think about long-term architectural impacts and edge cases. Your code must be impeccable.

STANDARDS (Master):
- For changes affecting more than 3 files or introducing new abstractions, first write a proposal to '.proposed_changes/PLAN.md' using 'write_file', then call 'ask_user' for approval before implementing.
- Never introduce 'unsafe' blocks without explicit user request and a documented safety invariant comment.
- Always check for existing patterns in 'src/config.rs' and existing 'Arc<Mutex<T>>' usage before introducing new synchronization primitives.
- After every code change, run 'exec: cargo check' before calling 'announce_completion'."
            }
        }
    }
}

/// Reasoning depth defining how much analysis the agent should perform.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub enum AiReasoningDepth {
    /// Quick responses, low token usage, minimal research.
    Fast,
    /// Standard balance, reads relevant files.
    #[default]
    Balanced,
    /// Deep analysis, multiple research steps, exhaustive validation.
    Deep,
}

impl AiReasoningDepth {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiReasoningDepth::Fast => "Fast",
            AiReasoningDepth::Balanced => "Balanced",
            AiReasoningDepth::Deep => "Deep",
        }
    }

    pub fn get_reasoning_mandate(&self) -> &'static str {
        match self {
            AiReasoningDepth::Fast => {
                "REASONING: FAST. Provide direct answers. Minimize file reading. Focus on the immediate prompt and currently open files."
            }
            AiReasoningDepth::Balanced => {
                "REASONING: BALANCED. Perform necessary research. Check 2-3 related files if needed to ensure consistency. Think before you act."
            }
            AiReasoningDepth::Deep => {
                "REASONING: DEEP. This is a complex task. You MUST perform exhaustive codebase research using semantic search and file reading. Map dependencies. Verify your logic through multi-step 'monologue' steps (> step). Do not rush."
            }
        }
    }
}

/// Data structure representing the current project context for AI agents.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AiContextPayload {
    pub open_files: Vec<AiFileContext>,
    pub build_errors: Vec<AiBuildErrorContext>,
    pub active_file: Option<AiFileContext>,
    #[serde(default)]
    pub memory_keys: Vec<String>,
    // --- Editor context ---
    /// Cursor line in the active file (1-based).
    #[serde(default)]
    pub cursor_line: Option<usize>,
    /// Cursor column in the active file (1-based).
    #[serde(default)]
    pub cursor_col: Option<usize>,
    /// Currently selected text in the active file (None if no selection).
    #[serde(default)]
    pub selected_text: Option<String>,
    // --- Project context ---
    /// Name of the currently open project (last segment of root_path).
    #[serde(default)]
    pub project_name: String,
    /// Project root expressed as "." (all paths in this payload are relative to it).
    #[serde(default)]
    pub project_root: String,
    // --- Git context ---
    /// Current git branch name.
    #[serde(default)]
    pub git_branch: Option<String>,
    /// Per-file git status (only modified/added/deleted/untracked files).
    #[serde(default)]
    pub git_status: Vec<AiGitFileStatus>,
    // --- Dependency context ---
    /// Extracted [package] + [dependencies] sections from Cargo.toml.
    #[serde(default)]
    pub cargo_toml_summary: Option<String>,
    // --- Runtime context ---
    /// Last N lines of terminal output (populated by caller).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_output: Option<String>,
    /// LSP diagnostic messages (populated by caller).
    #[serde(default)]
    pub lsp_diagnostics: Vec<String>,
}

impl AiContextPayload {
    /// Formats all context fields into a structured system message string.
    /// Sections with no data are omitted.
    pub fn to_system_message(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Header
        parts.push("# Editor Context".to_string());
        if !self.project_name.is_empty() {
            let git_info = if let Some(ref branch) = self.git_branch {
                let modified_count = self.git_status.len();
                format!(" | Git: {branch} | {modified_count} modified files")
            } else {
                String::new()
            };
            parts.push(format!(
                "Project: {} ({}){}",
                self.project_name, self.project_root, git_info
            ));
        }

        // Open Files
        if !self.open_files.is_empty() {
            let mut section = String::from("\n## Open Files");
            for f in &self.open_files {
                let mut label = format!("\n- {}", f.path);
                if f.is_active {
                    label.push_str(" (active");
                    if let Some(line) = self.cursor_line {
                        label.push_str(&format!(", cursor line {line}"));
                    }
                    label.push(')');
                }
                section.push_str(&label);
            }
            parts.push(section);
        }

        // Build Errors
        if !self.build_errors.is_empty() {
            let mut section = String::from("\n## Build Errors");
            for e in &self.build_errors {
                let prefix = if e.is_warning { "warning" } else { "error" };
                section.push_str(&format!(
                    "\n- {}:{}:{}: {}",
                    e.file, e.line, prefix, e.message
                ));
            }
            parts.push(section);
        }

        // Active File excerpt (+-50 lines around cursor)
        if let Some(ref active) = self.active_file {
            if let Some(ref content) = active.content {
                let lines: Vec<&str> = content.lines().collect();
                let cursor = self.cursor_line.unwrap_or(1);
                let start = cursor.saturating_sub(50).max(1);
                let end = (cursor + 50).min(lines.len());
                if start <= end && !lines.is_empty() {
                    let excerpt: Vec<&str> = lines[(start - 1)..end].to_vec();
                    parts.push(format!(
                        "\n## Active File ({}, lines {}-{})\n```\n{}\n```",
                        active.path,
                        start,
                        end,
                        excerpt.join("\n")
                    ));
                }
            }
        }

        // Terminal Output
        if let Some(ref terminal) = self.terminal_output {
            if !terminal.is_empty() {
                // Take last 50 lines
                let lines: Vec<&str> = terminal.lines().collect();
                let start = lines.len().saturating_sub(50);
                let excerpt = &lines[start..];
                parts.push(format!(
                    "\n## Terminal Output (last {} lines)\n```\n{}\n```",
                    excerpt.len(),
                    excerpt.join("\n")
                ));
            }
        }

        // LSP Diagnostics
        if !self.lsp_diagnostics.is_empty() {
            let mut section = String::from("\n## LSP Diagnostics");
            for d in &self.lsp_diagnostics {
                section.push_str(&format!("\n- {d}"));
            }
            parts.push(section);
        }

        // Cargo.toml summary
        if let Some(ref cargo) = self.cargo_toml_summary {
            if !cargo.is_empty() {
                parts.push(format!("\n## Cargo.toml Summary\n```toml\n{cargo}\n```"));
            }
        }

        parts.join("\n")
    }
}

/// Git status of a single file.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiGitFileStatus {
    pub path: String,
    /// Short git status code: "M" (modified), "A" (added), "D" (deleted), "??" (untracked).
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiFileContext {
    pub path: String,
    pub content: Option<String>,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiBuildErrorContext {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub is_warning: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiToolDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Represents a single message in a conversation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiMessage {
    pub role: String, // "user", "assistant", "system", "tool"
    pub content: String,
    pub monologue: Vec<String>,
    pub timestamp: u64,
    // --- Tool call metadata (backwards-compatible) ---
    /// Name of the tool called in this message (for assistant "tool_use" turns).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_name: Option<String>,
    /// Opaque ID linking a tool_use turn to its tool_result turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// ID of the tool_use turn this result belongs to (for "tool" role messages).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_result_for_id: Option<String>,
    /// Whether this tool result represents an error.
    #[serde(default)]
    pub tool_is_error: bool,
    /// Raw tool_calls arguments (for replaying assistant tool_calls back to the model).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_arguments: Option<serde_json::Value>,
}

/// Represents a full conversation thread.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AiConversation {
    pub messages: Vec<AiMessage>,
    pub total_in_tokens: u64,
    pub total_out_tokens: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_payload_has_terminal_output_field() {
        let mut payload = AiContextPayload::default();
        assert!(payload.terminal_output.is_none());
        payload.terminal_output = Some("cargo build OK".to_string());
        assert_eq!(payload.terminal_output.as_deref(), Some("cargo build OK"));
    }

    #[test]
    fn context_payload_has_lsp_diagnostics_field() {
        let mut payload = AiContextPayload::default();
        assert!(payload.lsp_diagnostics.is_empty());
        payload
            .lsp_diagnostics
            .push("warning: unused variable".to_string());
        assert_eq!(payload.lsp_diagnostics.len(), 1);
    }

    #[test]
    fn to_system_message_includes_project_and_git() {
        let payload = AiContextPayload {
            project_name: "myproject".to_string(),
            project_root: ".".to_string(),
            git_branch: Some("main".to_string()),
            git_status: vec![AiGitFileStatus {
                path: "src/lib.rs".to_string(),
                status: "M".to_string(),
            }],
            ..Default::default()
        };
        let msg = payload.to_system_message();
        assert!(msg.contains("Project: myproject"));
        assert!(msg.contains("Git: main"));
        assert!(msg.contains("1 modified files"));
    }

    #[test]
    fn to_system_message_omits_empty_optional_sections() {
        let payload = AiContextPayload {
            project_name: "test".to_string(),
            project_root: ".".to_string(),
            ..Default::default()
        };
        let msg = payload.to_system_message();
        assert!(!msg.contains("Terminal"));
        assert!(!msg.contains("LSP Diagnostics"));
        assert!(!msg.contains("Build Errors"));
    }

    #[test]
    fn to_system_message_includes_terminal_output() {
        let payload = AiContextPayload {
            project_name: "test".to_string(),
            project_root: ".".to_string(),
            terminal_output: Some("error[E0308]: mismatched types".to_string()),
            ..Default::default()
        };
        let msg = payload.to_system_message();
        assert!(msg.contains("## Terminal Output"));
        assert!(msg.contains("error[E0308]: mismatched types"));
    }

    #[test]
    fn to_system_message_includes_lsp_diagnostics() {
        let payload = AiContextPayload {
            project_name: "test".to_string(),
            project_root: ".".to_string(),
            lsp_diagnostics: vec!["warning: unused variable `x` at src/main.rs:15".to_string()],
            ..Default::default()
        };
        let msg = payload.to_system_message();
        assert!(msg.contains("## LSP Diagnostics"));
        assert!(msg.contains("unused variable"));
    }

    #[test]
    fn to_system_message_active_file_excerpt_around_cursor() {
        // Create a 100-line file, cursor at line 60
        let content: String = (1..=100).map(|i| format!("line {i}\n")).collect();
        let payload = AiContextPayload {
            project_name: "test".to_string(),
            project_root: ".".to_string(),
            cursor_line: Some(60),
            active_file: Some(AiFileContext {
                path: "src/main.rs".to_string(),
                content: Some(content),
                is_active: true,
            }),
            open_files: vec![AiFileContext {
                path: "src/main.rs".to_string(),
                content: None,
                is_active: true,
            }],
            ..Default::default()
        };
        let msg = payload.to_system_message();
        assert!(msg.contains("## Active File (src/main.rs, lines 10-100)"));
        assert!(msg.contains("line 10"));
        assert!(msg.contains("line 100"));
        // Should not contain line 9 (outside window)
        assert!(!msg.contains("\nline 9\n") || msg.contains("line 9")); // line 9 is outside +-50 of 60
    }

    #[test]
    fn to_system_message_build_errors_section() {
        let payload = AiContextPayload {
            project_name: "test".to_string(),
            project_root: ".".to_string(),
            build_errors: vec![
                AiBuildErrorContext {
                    file: "src/main.rs".to_string(),
                    line: 10,
                    message: "expected `;`".to_string(),
                    is_warning: false,
                },
                AiBuildErrorContext {
                    file: "src/lib.rs".to_string(),
                    line: 5,
                    message: "unused import".to_string(),
                    is_warning: true,
                },
            ],
            ..Default::default()
        };
        let msg = payload.to_system_message();
        assert!(msg.contains("## Build Errors"));
        assert!(msg.contains("src/main.rs:10:error: expected `;`"));
        assert!(msg.contains("src/lib.rs:5:warning: unused import"));
    }

    #[test]
    fn ai_message_has_tool_call_arguments() {
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
        assert!(msg.tool_call_arguments.is_some());
        assert_eq!(msg.tool_call_arguments.unwrap()["path"], "src/main.rs");
    }
}
