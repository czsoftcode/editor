pub mod tools;
pub mod types;

pub use types::*;
// Explicitly re-export standard tools
pub use tools::get_standard_tools;

use crate::app::ui::workspace::state::WorkspaceState;

/// Centralized logic for AI agents.
pub struct AiManager;

impl AiManager {
    /// Returns the centralized system mandates for an agent based on its configuration.
    pub fn get_system_mandates(role: AiExpertiseRole, depth: AiReasoningDepth) -> String {
        format!(
            "{}
{}

CORE MANDATE: Use 'replace' for ALL code modifications in existing files. Use 'write_file' ONLY for new files or final reports. Ensure context in 'old_string' is sufficient (3+ lines) to avoid ambiguity.

Strictly adhere to these levels of expertise and reasoning depth.",
            role.get_persona_mandate(),
            depth.get_reasoning_mandate()
        )
    }

    /// Generates a unified context payload from the current workspace state.
    pub fn generate_context(ws: &WorkspaceState) -> AiContextPayload {
        let mut payload = AiContextPayload::default();

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

        payload
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
  / /_/ / __ \/ / / / // /   / ___/ _ \/ __  / __ 
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
