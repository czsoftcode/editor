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
                "ROLE: JUNIOR DEVELOPER. You are eager to help but cautious. Follow instructions literally. Use simple, readable code. If unsure, ask for clarification. Do not over-engineer."
            }
            AiExpertiseRole::Senior => {
                "ROLE: SENIOR DEVELOPER. You are an experienced engineer. Maintain high standards, follow project conventions, and ensure code is maintainable. Proactively suggest improvements for readability and performance."
            }
            AiExpertiseRole::Master => {
                "ROLE: MASTER ARCHITECT. You have a deep understanding of software systems. Prioritize security, scalability, and extreme optimization. Think about long-term architectural impacts and edge cases. Your code must be impeccable."
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
}

/// Represents a full conversation thread.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AiConversation {
    pub messages: Vec<AiMessage>,
    pub total_in_tokens: u64,
    pub total_out_tokens: u64,
}
