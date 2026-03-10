use crate::app::cli::AiManager;
use crate::app::types::AppShared;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::config;
use std::sync::{Arc, Mutex};

/// Marker prefix for system messages in conversation.
/// System messages are slash command output rendered with distinct styling.
pub const SYSTEM_MSG_MARKER: &str = "\x00SYS\x00";

/// Result of dispatching a slash command.
pub enum SlashResult {
    /// Markdown response shown in conversation as system message.
    Immediate(String),
    /// Async command: show placeholder in conversation, background thread will update it.
    Async { placeholder: String },
    /// No conversation output (e.g., /settings opens a modal).
    Silent,
    /// Not a slash command — pass through to AI.
    NotACommand,
}

struct SlashCommand {
    name: &'static str,
    description: &'static str,
}

const COMMANDS: &[SlashCommand] = &[
    SlashCommand {
        name: "help",
        description: "Show available commands",
    },
    SlashCommand {
        name: "clear",
        description: "Clear conversation",
    },
    SlashCommand {
        name: "new",
        description: "New conversation",
    },
    SlashCommand {
        name: "model",
        description: "List or switch AI model",
    },
    SlashCommand {
        name: "git",
        description: "Show git diff summary",
    },
    SlashCommand {
        name: "build",
        description: "Run cargo build",
    },
    SlashCommand {
        name: "settings",
        description: "Open settings",
    },
    SlashCommand {
        name: "gsd",
        description: "GSD project management (/gsd help for subcommands)",
    },
];

/// Returns matching commands for autocomplete. Each item is (name, description).
/// If filter is empty, returns all commands. Otherwise filters by prefix match on name.
pub fn matching_commands(filter: &str) -> Vec<(&'static str, &'static str)> {
    let lower = filter.to_lowercase();
    COMMANDS
        .iter()
        .filter(|cmd| lower.is_empty() || cmd.name.starts_with(&lower))
        .map(|cmd| (cmd.name, cmd.description))
        .collect()
}

/// Main entry point for slash command dispatch.
/// Called from `logic.rs` when prompt starts with `/`.
pub fn dispatch(ws: &mut WorkspaceState, shared: &Arc<Mutex<AppShared>>) {
    let prompt = ws.ai.chat.prompt.trim().to_string();

    // Parse command name: first word after `/`
    let parts: Vec<&str> = prompt.splitn(2, char::is_whitespace).collect();
    let cmd_word = &parts[0][1..]; // strip leading `/`

    // Record prompt in history (skip if duplicate of last)
    if ws.ai.chat.history.last().map(|h| h.as_str()) != Some(&prompt) {
        ws.ai.chat.history.push(prompt.clone());
    }
    ws.ai.chat.history_index = None;

    // Extract args (everything after the command name)
    let args = parts.get(1).unwrap_or(&"").trim();

    // Try strict lowercase match
    let result = match cmd_word.to_lowercase().as_str() {
        "help" => cmd_help(),
        "clear" => cmd_clear(ws),
        "new" => cmd_new(ws, shared),
        "settings" => cmd_settings(ws),
        "model" => cmd_model(ws, args),
        "git" => cmd_git(ws),
        "build" => cmd_build(ws),
        "gsd" => super::gsd::cmd_gsd(ws, args),
        _ => {
            // Fuzzy suggestion for unknown commands
            fuzzy_or_passthrough(cmd_word, ws, &prompt)
        }
    };

    match result {
        SlashResult::Immediate(response) => {
            ws.ai
                .chat
                .conversation
                .push((prompt, format!("{}{}", SYSTEM_MSG_MARKER, response)));
            ws.ai.chat.prompt.clear();
        }
        SlashResult::Async { placeholder } => {
            ws.ai
                .chat
                .conversation
                .push((prompt, format!("{}{}", SYSTEM_MSG_MARKER, placeholder)));
            ws.ai.chat.prompt.clear();
        }
        SlashResult::Silent => {
            ws.ai.chat.conversation.push((prompt, String::new()));
            ws.ai.chat.prompt.clear();
        }
        SlashResult::NotACommand => {
            // Do NOT clear prompt, do NOT push to conversation.
            // Undo the history push — send_query_to_agent will handle it.
            if ws.ai.chat.history.last().map(|h| h.as_str()) == Some(prompt.as_str()) {
                ws.ai.chat.history.pop();
            }
            ws.ai.chat.history_index = None;
        }
    }
}

/// Fuzzy match or pass through to AI.
fn fuzzy_or_passthrough(cmd_word: &str, _ws: &mut WorkspaceState, _prompt: &str) -> SlashResult {
    // Only treat as unknown command if short (<= 10 chars)
    if cmd_word.len() > 10 {
        return SlashResult::NotACommand;
    }

    let lower = cmd_word.to_lowercase();
    let mut best_match: Option<(&str, usize)> = None;

    for cmd in COMMANDS {
        let dist = levenshtein(&lower, cmd.name);
        if dist <= 2 {
            if best_match.is_none() || dist < best_match.unwrap().1 {
                best_match = Some((cmd.name, dist));
            }
        }
    }

    if let Some((suggestion, _)) = best_match {
        SlashResult::Immediate(format!(
            "Unknown command: `/{cmd_word}`. Did you mean: `/{suggestion}`? Type `/help` for available commands."
        ))
    } else {
        SlashResult::NotACommand
    }
}

/// Levenshtein edit distance between two strings.
fn levenshtein(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0; b_len + 1];

    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(curr[j] + 1);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

fn cmd_help() -> SlashResult {
    let mut table = String::from("| Command | Description |\n|---------|-------------|\n");
    for cmd in COMMANDS {
        table.push_str(&format!("| /{} | {} |\n", cmd.name, cmd.description));
    }
    SlashResult::Immediate(table)
}

fn cmd_clear(ws: &mut WorkspaceState) -> SlashResult {
    ws.ai.chat.conversation.clear();
    ws.ai.chat.in_tokens = 0;
    ws.ai.chat.out_tokens = 0;
    ws.ai.chat.thinking_history.clear();
    ws.slash_conversation_gen += 1;
    // Do NOT clear ws.ai.chat.history — preserve prompt recall
    SlashResult::Immediate("Conversation cleared.".to_string())
}

fn cmd_new(ws: &mut WorkspaceState, shared: &Arc<Mutex<AppShared>>) -> SlashResult {
    // Clear conversation (same as /clear)
    ws.ai.chat.conversation.clear();
    ws.ai.chat.in_tokens = 0;
    ws.ai.chat.out_tokens = 0;
    ws.ai.chat.thinking_history.clear();
    ws.ai.chat.response = None;
    ws.ai.chat.monologue.clear();
    ws.slash_conversation_gen += 1;

    // Get model name from shared settings (same as NewQuery in mod.rs)
    let model = {
        let sh = shared.lock().expect("lock");
        if !sh.settings.ai_default_model.is_empty() {
            sh.settings.ai_default_model.clone()
        } else {
            "llama3.1".to_string()
        }
    };

    // Push ASCII logo
    ws.ai.chat.conversation.push((
        String::new(),
        AiManager::get_logo(
            config::CLI_VERSION,
            &model,
            ws.ai.settings.expertise,
            ws.ai.settings.reasoning_depth,
        ),
    ));

    SlashResult::Silent
}

fn cmd_settings(ws: &mut WorkspaceState) -> SlashResult {
    ws.show_settings = true;
    SlashResult::Silent
}

fn cmd_model(ws: &mut WorkspaceState, args: &str) -> SlashResult {
    if args.is_empty() {
        // List all models
        if ws.ai.ollama.models.is_empty() {
            return SlashResult::Immediate(
                "No models available. Check Ollama connection.".to_string(),
            );
        }
        let mut out = String::from("## Models\n\n");
        for model in &ws.ai.ollama.models {
            if *model == ws.ai.ollama.selected_model {
                out.push_str(&format!("* **{}** (active)\n", model));
            } else {
                out.push_str(&format!("* {}\n", model));
            }
        }
        SlashResult::Immediate(out)
    } else {
        // Switch model
        let target = args.to_string();
        if ws.ai.ollama.models.iter().any(|m| m == &target) {
            ws.ai.ollama.selected_model = target.clone();
            SlashResult::Immediate(format!("Switched to model: **{}**", target))
        } else {
            // Fuzzy suggest closest model
            let mut best: Option<(&str, usize)> = None;
            for model in &ws.ai.ollama.models {
                let dist = levenshtein(&target.to_lowercase(), &model.to_lowercase());
                if dist <= 3 {
                    if best.is_none() || dist < best.unwrap().1 {
                        best = Some((model, dist));
                    }
                }
            }
            if let Some((suggestion, _)) = best {
                SlashResult::Immediate(format!(
                    "Model '{}' not found. Did you mean: **{}**?",
                    target, suggestion
                ))
            } else {
                let available: Vec<&str> = ws.ai.ollama.models.iter().map(|s| s.as_str()).collect();
                SlashResult::Immediate(format!(
                    "Model '{}' not found. Available models: {}",
                    target,
                    available.join(", ")
                ))
            }
        }
    }
}

fn cmd_git(ws: &mut WorkspaceState) -> SlashResult {
    let root = ws.root_path.clone();
    let branch = ws.git_branch.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let output = std::process::Command::new("git")
            .args(["diff", "--stat", "HEAD"])
            .current_dir(&root)
            .output();
        let result = match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let branch_str = branch.as_deref().unwrap_or("unknown");
                if stdout.trim().is_empty() {
                    format!("**Branch:** `{}`\n\nNo uncommitted changes.", branch_str)
                } else {
                    format!(
                        "**Branch:** `{}`\n\n```\n{}\n```",
                        branch_str,
                        stdout.trim()
                    )
                }
            }
            Err(e) => format!("Git error: {}", e),
        };
        let _ = tx.send(result);
    });
    ws.slash_git_rx = Some(rx);
    SlashResult::Async {
        placeholder: "Loading git status...".to_string(),
    }
}

fn cmd_build(ws: &mut WorkspaceState) -> SlashResult {
    let rx = crate::app::build_runner::run_build_check(ws.root_path.clone());
    ws.slash_build_rx = Some(rx);
    ws.slash_build_gen = ws.slash_conversation_gen;
    SlashResult::Async {
        placeholder: "Building...".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("help", "help"), 0);
        assert_eq!(levenshtein("help", "halp"), 1);
        assert_eq!(levenshtein("help", "build"), 4);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn test_dispatch_help() {
        let result = cmd_help();
        match result {
            SlashResult::Immediate(text) => {
                assert!(text.contains("| Command | Description |"));
                assert!(text.contains("/help"));
                assert!(text.contains("/clear"));
                assert!(text.contains("/new"));
                assert!(text.contains("/model"));
                assert!(text.contains("/git"));
                assert!(text.contains("/build"));
                assert!(text.contains("/settings"));
                assert!(text.contains("/gsd"));
            }
            _ => panic!("Expected Immediate result from /help"),
        }
    }

    #[test]
    fn test_fuzzy_suggestion() {
        // "halp" -> should suggest "help" (distance 1)
        let lower = "halp";
        let mut best: Option<(&str, usize)> = None;
        for cmd in COMMANDS {
            let dist = levenshtein(lower, cmd.name);
            if dist <= 2 {
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((cmd.name, dist));
                }
            }
        }
        assert_eq!(best.map(|(n, _)| n), Some("help"));

        // "bild" -> should suggest "build" (distance 1)
        let lower2 = "bild";
        let mut best2: Option<(&str, usize)> = None;
        for cmd in COMMANDS {
            let dist = levenshtein(lower2, cmd.name);
            if dist <= 2 {
                if best2.is_none() || dist < best2.unwrap().1 {
                    best2 = Some((cmd.name, dist));
                }
            }
        }
        assert_eq!(best2.map(|(n, _)| n), Some("build"));
    }

    fn make_test_ws_with_models(models: Vec<String>, selected: &str) -> WorkspaceState {
        use crate::app::cli::state::*;
        let mut ws = WorkspaceState {
            file_tree: crate::app::ui::file_tree::FileTree::new(),
            editor: crate::app::ui::editor::Editor::new(),
            watcher: crate::watcher::FileWatcher::new(),
            project_watcher: crate::watcher::ProjectWatcher::new(&std::path::PathBuf::from(
                "/tmp/test",
            )),
            claude_tabs: Vec::new(),
            claude_active_tab: 0,
            next_claude_tab_id: 1,
            next_terminal_id: 2,
            build_terminal: None,
            retired_terminals: Vec::new(),
            focused_panel: crate::app::types::FocusedPanel::Editor,
            root_path: std::path::PathBuf::from("/tmp/test"),
            show_left_panel: false,
            show_right_panel: false,
            show_build_terminal: false,
            build_terminal_float: false,
            left_panel_split: 0.5,
            show_about: false,
            show_support: false,
            show_settings: false,
            show_ai_chat: false,
            show_semantic_indexing_modal: false,
            selected_settings_category: None,
            profiles: crate::app::types::ProjectProfiles::default(),
            build_errors: Vec::new(),
            build_error_rx: None,
            selected_agent_id: String::new(),
            claude_float: false,
            show_new_project: false,
            wizard: crate::app::ui::dialogs::WizardState::default(),
            toasts: Vec::new(),
            folder_pick_rx: None,
            command_palette: None,
            project_index: std::sync::Arc::new(
                crate::app::ui::workspace::index::ProjectIndex::new(std::path::PathBuf::from(
                    "/tmp/test",
                )),
            ),
            semantic_index: std::sync::Arc::new(std::sync::Mutex::new(
                crate::app::ui::workspace::semantic_index::SemanticIndex::new(
                    std::path::PathBuf::from("/tmp/test"),
                ),
            )),
            file_picker: None,
            project_search: crate::app::ui::workspace::state::types::ProjectSearch::default(),
            lsp_client: None,
            lsp_binary_missing: false,
            lsp_install_rx: None,
            git_branch: None,
            git_branch_rx: None,
            git_status_rx: None,
            git_last_refresh: std::time::Instant::now(),
            lsp_last_retry: std::time::Instant::now(),
            settings_draft: None,
            settings_original: None,
            settings_folder_pick_rx: None,
            ai_tool_available: std::collections::HashMap::new(),
            ai_tool_check_rx: None,
            ai_tool_last_check: std::time::Instant::now(),
            win_tool_available: std::collections::HashMap::new(),
            win_tool_check_rx: None,
            win_tool_last_check: std::time::Instant::now(),
            external_change_conflict: None,
            dep_wizard: crate::app::ui::dialogs::DependencyWizard::new(),
            terminal_close_requested: None,
            ai_viewport_open: false,
            settings_conflict: None,
            ai: AiState::default(),
            git_cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            local_history: crate::app::local_history::LocalHistory::new(&std::path::PathBuf::from(
                "/tmp/test",
            )),
            background_io_rx: None,
            applied_settings_version: 0,
            confirm_discard_changes: None,
            last_keystroke_time: None,
            pending_close_flow: None,
            last_unsaved_close_cancelled: false,
            tool_executor: None,
            pending_tool_approval: None,
            pending_tool_ask: None,
            tool_always_approved: std::collections::HashSet::new(),
            tool_approval_rx: None,
            tool_ask_rx: None,
            slash_build_rx: None,
            slash_git_rx: None,
            slash_conversation_gen: 0,
            slash_build_gen: 0,
            slash_autocomplete: Default::default(),
        };
        ws.ai.ollama.models = models;
        ws.ai.ollama.selected_model = selected.to_string();
        ws
    }

    #[test]
    fn test_cmd_model_list() {
        let mut ws = make_test_ws_with_models(
            vec![
                "llama3.1".to_string(),
                "deepseek-r1".to_string(),
                "codellama".to_string(),
            ],
            "llama3.1",
        );
        let result = cmd_model(&mut ws, "");
        match result {
            SlashResult::Immediate(text) => {
                assert!(text.contains("## Models"), "Should have heading");
                assert!(
                    text.contains("**llama3.1** (active)"),
                    "Active model should be marked"
                );
                assert!(text.contains("* deepseek-r1"), "Other models listed");
                assert!(text.contains("* codellama"), "Other models listed");
            }
            _ => panic!("Expected Immediate result from /model"),
        }
    }

    #[test]
    fn test_cmd_model_switch_valid() {
        let mut ws = make_test_ws_with_models(
            vec!["llama3.1".to_string(), "deepseek-r1".to_string()],
            "llama3.1",
        );
        let result = cmd_model(&mut ws, "deepseek-r1");
        match result {
            SlashResult::Immediate(text) => {
                assert!(text.contains("Switched to model: **deepseek-r1**"));
                assert_eq!(ws.ai.ollama.selected_model, "deepseek-r1");
            }
            _ => panic!("Expected Immediate result from /model switch"),
        }
    }

    #[test]
    fn test_cmd_model_switch_invalid() {
        let mut ws = make_test_ws_with_models(
            vec!["llama3.1".to_string(), "deepseek-r1".to_string()],
            "llama3.1",
        );
        // Close enough to "llama3.1" with Levenshtein <= 3
        let result = cmd_model(&mut ws, "llama3");
        match result {
            SlashResult::Immediate(text) => {
                assert!(text.contains("not found"), "Should report not found");
                assert!(text.contains("llama3.1"), "Should suggest closest match");
            }
            _ => panic!("Expected Immediate result from /model invalid"),
        }

        // Totally different name
        let result2 = cmd_model(&mut ws, "gpt-4o");
        match result2 {
            SlashResult::Immediate(text) => {
                assert!(text.contains("not found"), "Should report not found");
                assert!(
                    text.contains("Available models:"),
                    "Should list available models"
                );
            }
            _ => panic!("Expected Immediate result from /model unknown"),
        }
    }

    #[test]
    fn test_cmd_model_empty_list() {
        let mut ws = make_test_ws_with_models(Vec::new(), "");
        let result = cmd_model(&mut ws, "");
        match result {
            SlashResult::Immediate(text) => {
                assert!(text.contains("No models available"));
            }
            _ => panic!("Expected Immediate result from /model with no models"),
        }
    }

    #[test]
    fn test_matching_commands() {
        let all = matching_commands("");
        assert_eq!(all.len(), 8);
        let filtered = matching_commands("he");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].0, "help");
        let none = matching_commands("xyz");
        assert!(none.is_empty());
    }

    #[test]
    fn test_long_word_passes_to_ai() {
        // Words > 10 chars should return NotACommand
        let cmd_word = "thisisaverylongword";
        assert!(cmd_word.len() > 10);
        // The fuzzy_or_passthrough logic checks length > 10
        // We test the levenshtein approach directly:
        // No command name is close to this, so it should be NotACommand
        let lower = cmd_word.to_lowercase();
        let mut any_close = false;
        for cmd in COMMANDS {
            if levenshtein(&lower, cmd.name) <= 2 {
                any_close = true;
            }
        }
        assert!(!any_close, "Long word should not match any command");
    }
}
