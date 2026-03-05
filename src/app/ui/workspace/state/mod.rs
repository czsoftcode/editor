pub mod actions;
pub mod init;
pub mod types;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, mpsc};

use eframe::egui;

use crate::app::ai::{AiExpertiseRole, AiReasoningDepth};
use crate::app::build_runner::BuildError;
use crate::app::lsp::LspClient;
use crate::app::types::{FocusedPanel, ProjectProfiles, Toast};
use crate::app::ui::dialogs::WizardState;
use crate::app::ui::editor::Editor;
use crate::app::ui::file_tree::FileTree;
use crate::app::ui::git_status::GitVisualStatus;
use crate::app::ui::terminal::Terminal;
use crate::app::ui::widgets::command_palette::CommandPaletteState;
use crate::watcher::{FileWatcher, ProjectWatcher};

pub use self::actions::{open_and_jump, open_file_in_ws, spawn_ai_tool_check, ws_to_panel_state};
pub use self::init::init_workspace;
pub use self::types::{
    FilePicker, FolderPickResult, FsChangeResult, ProjectSearch, SearchResult, SecondaryWorkspace,
};

pub type PendingPluginApproval = (
    String,
    String,
    String,
    std::sync::mpsc::Sender<crate::app::types::PluginApprovalResponse>,
);

pub type PendingAskUser = (
    String,
    String,
    Vec<String>,
    String,
    std::sync::mpsc::Sender<String>,
);

#[derive(Clone)]
pub struct PendingSettingsSave {
    pub sandbox_off_confirmed: bool,
}

#[derive(Clone)]
pub struct SandboxApplyRequest {
    pub target_mode: bool,
    pub version: u64,
    pub defer_until_clear: bool,
    pub force_apply: bool,
    pub prompted: bool,
    pub notify_on_apply: bool,
}

#[derive(Clone)]
pub struct SandboxPersistFailure {
    pub draft: crate::settings::Settings,
    pub original: crate::settings::Settings,
}

#[derive(Clone)]
pub struct SettingsConflict {
    pub new_settings: crate::settings::Settings,
}

pub struct TabRemapRequest {
    pub from_root: PathBuf,
    pub to_root: PathBuf,
}

pub struct WorkspaceState {
    pub file_tree: FileTree,
    pub editor: Editor,
    pub watcher: FileWatcher,
    pub project_watcher: ProjectWatcher,
    pub claude_tabs: Vec<Terminal>,
    pub claude_active_tab: usize,
    pub next_claude_tab_id: u64,
    pub next_terminal_id: u64,
    pub build_terminal: Option<Terminal>,
    pub retired_terminals: Vec<Terminal>,
    pub focused_panel: FocusedPanel,
    pub root_path: PathBuf,
    pub show_left_panel: bool,
    pub show_right_panel: bool,
    pub show_build_terminal: bool,
    pub build_terminal_float: bool,
    pub left_panel_split: f32,
    pub show_about: bool,
    pub show_support: bool,
    pub show_settings: bool,
    pub show_plugins: bool,
    pub show_ai_chat: bool,
    pub show_semantic_indexing_modal: bool,
    pub ai_show_settings: bool,
    pub selected_plugin_id: Option<String>,
    pub selected_settings_category: Option<String>,
    pub ai_font_scale: u32,
    pub profiles: ProjectProfiles,
    pub build_errors: Vec<BuildError>,
    pub build_error_rx: Option<mpsc::Receiver<Vec<BuildError>>>,
    pub selected_agent_id: String,
    pub ai_selected_provider: String,
    pub claude_float: bool,
    pub show_new_project: bool,
    pub wizard: WizardState,
    pub toasts: Vec<Toast>,
    pub folder_pick_rx: Option<mpsc::Receiver<FolderPickResult>>,
    pub command_palette: Option<CommandPaletteState>,
    pub project_index: Arc<crate::app::ui::workspace::index::ProjectIndex>,
    pub semantic_index: Arc<Mutex<crate::app::ui::workspace::semantic_index::SemanticIndex>>,
    pub file_picker: Option<FilePicker>,
    pub project_search: ProjectSearch,
    pub lsp_client: Option<LspClient>,
    pub lsp_binary_missing: bool,
    pub lsp_install_rx: Option<mpsc::Receiver<Result<(), String>>>,
    pub git_branch: Option<String>,
    pub git_branch_rx: Option<mpsc::Receiver<Option<String>>>,
    pub git_status_rx: Option<mpsc::Receiver<HashMap<PathBuf, GitVisualStatus>>>,
    pub git_last_refresh: std::time::Instant,
    pub lsp_last_retry: std::time::Instant,
    pub settings_draft: Option<crate::settings::Settings>,
    pub settings_original: Option<crate::settings::Settings>,
    pub plugins_draft: Option<crate::settings::Settings>,
    pub settings_folder_pick_rx: Option<mpsc::Receiver<Option<PathBuf>>>,
    pub pending_settings_save: Option<PendingSettingsSave>,
    pub pending_sandbox_apply: Option<SandboxApplyRequest>,
    pub sandbox_persist_failure: Option<SandboxPersistFailure>,
    pub sandbox_persist_decision: Option<bool>,
    pub pending_tab_remap: Option<TabRemapRequest>,
    pub ai_tool_available: HashMap<String, bool>,
    pub ai_tool_check_rx: Option<mpsc::Receiver<HashMap<String, bool>>>,
    pub ai_tool_last_check: std::time::Instant,
    pub win_tool_available: HashMap<String, bool>,
    pub win_tool_check_rx: Option<mpsc::Receiver<HashMap<String, bool>>>,
    pub win_tool_last_check: std::time::Instant,
    pub external_change_conflict: Option<PathBuf>,
    pub dep_wizard: crate::app::ui::dialogs::DependencyWizard,
    pub sandbox_deletion_sync: Option<PathBuf>,
    pub terminal_close_requested: Option<usize>,
    pub ai_viewport_open: bool,
    pub promotion_success: Option<PathBuf>,
    pub show_sandbox_staged: bool,
    pub plugin_error: Option<String>,
    pub settings_conflict: Option<SettingsConflict>,
    pub ai_prompt: String,
    pub ai_history: Vec<String>,
    pub ai_history_index: Option<usize>,
    pub ai_monologue: Vec<String>,
    pub ai_conversation: Vec<(String, String)>,
    pub ai_system_prompt: String,
    pub ai_language: String,
    pub ai_expertise: AiExpertiseRole,
    pub ai_reasoning_depth: AiReasoningDepth,
    pub ai_in_tokens: u32,
    pub ai_out_tokens: u32,
    pub ai_inspector_open: bool,
    pub ai_focus_requested: bool,
    pub ai_last_payload: String,
    pub ai_response: Option<String>,
    pub ai_loading: bool,
    pub markdown_cache: egui_commonmark::CommonMarkCache,
    pub sync_confirmation: Option<crate::app::sandbox::SyncPlan>,
    pub sandbox_sync_confirmation: Option<crate::app::sandbox::SyncPlan>,
    pub sandbox_sync_rx: Option<mpsc::Receiver<Result<usize, String>>>,
    pub pending_agent_id: Option<String>,
    /// Stable sandbox mode loaded at workspace init; changes apply only after reopen.
    pub sandbox_mode_enabled: bool,
    pub build_in_sandbox: bool,
    pub file_tree_in_sandbox: bool,
    pub git_cancel: Arc<AtomicBool>,
    pub local_history: crate::app::local_history::LocalHistory,
    pub sandbox: crate::app::sandbox::Sandbox,
    pub sandbox_staged_files: Vec<PathBuf>,
    pub sandbox_staged_rx: Option<mpsc::Receiver<Vec<PathBuf>>>,
    pub sandbox_staged_dirty: bool,
    pub sandbox_staged_last_dirty: std::time::Instant,
    pub sandbox_staged_last_refresh: std::time::Instant,
    pub background_io_rx: Option<mpsc::Receiver<FsChangeResult>>,
    pub applied_settings_version: u64,
    pub pending_plugin_approval: Option<PendingPluginApproval>,
    /// Pending ask_user request: (plugin_id, question, options, answer_input_buffer, sender)
    pub pending_ask_user: Option<PendingAskUser>,
    pub ai_cancellation_token: Arc<AtomicBool>,
    /// Pending discard changes confirmation for a specific modal ID.
    pub confirm_discard_changes: Option<String>,
    /// Last time the user pressed a key. Used for repaint capping during active typing.
    pub last_keystroke_time: Option<std::time::Instant>,
}

impl Drop for WorkspaceState {
    fn drop(&mut self) {
        self.git_cancel
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

impl WorkspaceState {
    pub fn tick_retired_terminals(&mut self) {
        for terminal in &mut self.retired_terminals {
            terminal.tick_background();
        }
        self.retired_terminals.retain(|terminal| !terminal.is_exited());
    }

    pub fn retire_terminal(&mut self, mut terminal: Terminal) {
        terminal.request_graceful_exit();
        self.retired_terminals.push(terminal);
    }

    pub fn apply_sandbox_mode_change(&mut self, ctx: &egui::Context, target_mode: bool) {
        if self.sandbox_mode_enabled == target_mode {
            return;
        }

        self.sandbox_mode_enabled = target_mode;
        self.build_in_sandbox = target_mode;
        self.file_tree_in_sandbox = target_mode;

        let target_root = if target_mode {
            self.sandbox.root.clone()
        } else {
            self.root_path.clone()
        };
        self.file_tree.load(&target_root);

        let old_tabs = std::mem::take(&mut self.claude_tabs);
        let mut new_tabs = Vec::with_capacity(old_tabs.len());
        for terminal in old_tabs {
            let init_command = terminal.init_command.clone();
            let id = terminal.id;
            self.retire_terminal(terminal);
            new_tabs.push(Terminal::new(
                id,
                ctx,
                &target_root,
                init_command.as_deref(),
            ));
        }
        self.claude_tabs = new_tabs;

        if let Some(old_terminal) = self.build_terminal.take() {
            self.retire_terminal(old_terminal);
            self.next_terminal_id += 1;
            self.build_terminal =
                Some(Terminal::new(self.next_terminal_id, ctx, &target_root, None));
        }
    }
}

pub(crate) fn should_apply_sandbox_request(
    defer_until_clear: bool,
    dialog_open: bool,
    force_apply: bool,
) -> bool {
    if force_apply {
        return true;
    }
    if defer_until_clear && dialog_open {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::should_apply_sandbox_request;

    #[test]
    fn test_should_apply_sandbox_request_blocks_when_deferred() {
        assert!(!should_apply_sandbox_request(true, true, false));
    }

    #[test]
    fn test_should_apply_sandbox_request_allows_when_clear() {
        assert!(should_apply_sandbox_request(true, false, false));
    }

    #[test]
    fn test_should_apply_sandbox_request_forces_apply() {
        assert!(should_apply_sandbox_request(true, true, true));
    }
}
