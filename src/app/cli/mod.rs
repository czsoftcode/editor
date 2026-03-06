pub mod audit;
pub mod executor;
pub mod ollama;
pub mod provider;
pub mod security;
pub mod state;
pub mod tools;
pub mod types;

pub use types::*;
pub use ollama::{OllamaStatus, spawn_ollama_check};
pub use state::AiState;

use crate::app::ui::git_status::GitVisualStatus;
use crate::app::ui::workspace::state::WorkspaceState;

/// Centralized logic for AI agents.
pub struct AiManager;

impl AiManager {
    /// Returns the centralized system mandates for an agent based on its configuration.
    pub fn get_system_mandates(
        role: AiExpertiseRole,
        depth: AiReasoningDepth,
        language_name: &str,
    ) -> String {
        let mut mandates = format!(
            "{}
{}

CORE MISSION PROTOCOL: 
You operate as a disciplined software engineer. For EVERY response, you MUST follow this structural thinking process in your monologue:

1. **REFLECTION**: Analyze the output of your last tool call. Did it succeed? What EXACTLY did you learn? If it failed, why? (Do not repeat failed logic).
2. **STATUS UPDATE**: Maintain a 'sub-goals' table in your scratchpad (use 'store_scratch' with key 'mission_status'). Mark tasks as [DONE], [ACTIVE], or [TODO].
3. **PLANNING**: Define the single next logical step based on the Hierarchy of Truth (Local Code > Config > Host Hints > Web).

MANDATORY RULES:
- Language: ALWAYS use '{}' for everything. This applies to both thoughts and final responses.
- Code: Use 'replace' for modifications. Never 'write_file' on existing source.
- Context: Every 5 steps, provide a 'MISSION SUMMARY' to keep your context clean.",
            role.get_persona_mandate(),
            depth.get_reasoning_mandate(),
            language_name
        );

        // --- CENTRALIZED PROJECT STANDARDS ---
        if let Ok(guide) = std::fs::read_to_string("AI_GUIDE.md") {
            mandates.push_str("\n\nPROJECT-SPECIFIC MANIFESTO (Read this first):\n");
            mandates.push_str(&guide);
        }

        mandates
    }

    /// Generates a unified context payload from the current workspace state.
    /// `terminal_output` and `lsp_diagnostics` are passed by caller (wired in Plan 04).
    pub fn generate_context(
        ws: &WorkspaceState,
        _shared: &std::sync::Arc<std::sync::Mutex<crate::app::types::AppShared>>,
        terminal_output: Option<String>,
        lsp_diagnostics: Vec<String>,
    ) -> AiContextPayload {
        let mut payload = AiContextPayload::default();
        payload.terminal_output = terminal_output;
        payload.lsp_diagnostics = lsp_diagnostics;

        // Memory keys are no longer available from WASM plugin system.

        // 1. Gather Open Files
        for (i, tab) in ws.editor.tabs.iter().enumerate() {
            let rel_path = tab
                .path
                .strip_prefix(&ws.root_path)
                .unwrap_or(&tab.path)
                .to_string_lossy()
                .into_owned();

            let is_active = Some(i) == ws.editor.active_tab;
            let file_ctx = AiFileContext {
                path: rel_path.clone(),
                content: if is_active {
                    Some(tab.content.clone())
                } else {
                    None
                },
                is_active,
            };

            payload.open_files.push(file_ctx.clone());
            if is_active {
                payload.active_file = Some(file_ctx);
            }
        }

        // 2. Gather Build Errors
        for err in &ws.build_errors {
            let rel_path = err
                .file
                .strip_prefix(&ws.root_path)
                .unwrap_or(&err.file)
                .to_string_lossy()
                .into_owned();

            payload.build_errors.push(AiBuildErrorContext {
                file: rel_path,
                line: err.line,
                message: err.message.clone(),
                is_warning: err.is_warning,
            });
        }

        // 3. Cursor position and selected text from active tab
        if let Some(tab) = ws.editor.active()
            && let Some(cr) = tab.last_cursor_range
        {
            payload.cursor_line = Some(cr.primary.rcursor.row + 1);
            payload.cursor_col = Some(cr.primary.rcursor.column + 1);
            let start = cr.primary.ccursor.index.min(cr.secondary.ccursor.index);
            let end = cr.primary.ccursor.index.max(cr.secondary.ccursor.index);
            if start != end {
                let text: String = tab.content.chars().skip(start).take(end - start).collect();
                payload.selected_text = Some(text);
            }
        }

        // 4. Project name and root
        payload.project_name = ws
            .root_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        payload.project_root = ".".to_string();

        // 5. Git context
        payload.git_branch = ws.git_branch.clone();
        for (abs_path, status) in &ws.file_tree.git_statuses {
            let rel = abs_path
                .strip_prefix(&ws.root_path)
                .unwrap_or(abs_path)
                .to_string_lossy()
                .into_owned();
            payload.git_status.push(AiGitFileStatus {
                path: rel,
                status: git_visual_status_to_code(*status).to_string(),
            });
        }

        // 6. Cargo.toml summary
        payload.cargo_toml_summary = extract_cargo_summary(&ws.root_path);

        payload
    }

    /// Builds a system message containing the full editor context.
    /// Called before each stream_chat to inject current state per CONTEXT.md locked decision:
    /// "Kontext se pripoji jako system message pri KAZDE zprave"
    pub fn build_system_message(
        ws: &WorkspaceState,
        shared: &std::sync::Arc<std::sync::Mutex<crate::app::types::AppShared>>,
    ) -> AiMessage {
        let payload = Self::generate_context(ws, shared, None, Vec::new());
        let content = payload.to_system_message();
        AiMessage {
            role: "system".to_string(),
            content,
            monologue: Vec::new(),
            timestamp: 0,
            tool_call_name: None,
            tool_call_id: None,
            tool_result_for_id: None,
            tool_is_error: false,
            tool_call_arguments: None,
        }
    }

    /// Returns the centralized ASCII logo for all CLI agents.
    pub fn get_logo(
        version: &str,
        model: &str,
        role: AiExpertiseRole,
        depth: AiReasoningDepth,
    ) -> String {
        format!(
            r#"    ____        __       ______              __
   / __ \____  / /_  __ / ____/_______  ____/ /___
  / /_/ / __ \/ / / / // /   / ___/ _ \/ __  / __ \
 / ____/ /_/ / / /_/ // /___/ /  /  __/ /_/ / /_/ /
/_/    \____/_/\__, / \____/_/   \___/\__,_/\____/
              /____/                              CLI

 Version: {}
 Model:   {}
 Rank:    {} ({})"#,
            version,
            model,
            role.as_str(),
            depth.as_str()
        )
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Converts semantic git status to the short status code expected by AI context.
fn git_visual_status_to_code(status: GitVisualStatus) -> &'static str {
    match status {
        GitVisualStatus::Modified => "M",
        GitVisualStatus::Added => "A",
        GitVisualStatus::Deleted => "D",
        GitVisualStatus::Untracked => "??",
    }
}

/// Extracts [package] and [dependencies] sections from Cargo.toml as a compact summary.
fn extract_cargo_summary(root: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(root.join("Cargo.toml")).ok()?;
    let mut lines: Vec<&str> = Vec::new();
    let mut in_relevant = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" || trimmed == "[dependencies]" || trimmed == "[dev-dependencies]"
        {
            in_relevant = true;
            lines.push(line);
        } else if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            in_relevant = false;
        } else if in_relevant && !trimmed.is_empty() {
            lines.push(line);
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}
