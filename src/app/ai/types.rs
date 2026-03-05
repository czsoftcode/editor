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
}

/// Represents a full conversation thread.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AiConversation {
    pub messages: Vec<AiMessage>,
    pub total_in_tokens: u64,
    pub total_out_tokens: u64,
}
