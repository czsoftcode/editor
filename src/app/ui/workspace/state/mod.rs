pub mod actions;
pub mod init;
pub mod types;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, mpsc};

use crate::app::ai::{AiState, OllamaStatus};
pub use crate::app::ai::state::OllamaConnectionStatus;
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
pub struct SettingsConflict {
    pub new_settings: crate::settings::Settings,
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
    pub selected_plugin_id: Option<String>,
    pub selected_settings_category: Option<String>,
    pub profiles: ProjectProfiles,
    pub build_errors: Vec<BuildError>,
    pub build_error_rx: Option<mpsc::Receiver<Vec<BuildError>>>,
    pub selected_agent_id: String,
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
    pub ai_tool_available: HashMap<String, bool>,
    pub ai_tool_check_rx: Option<mpsc::Receiver<HashMap<String, bool>>>,
    pub ai_tool_last_check: std::time::Instant,
    pub win_tool_available: HashMap<String, bool>,
    pub win_tool_check_rx: Option<mpsc::Receiver<HashMap<String, bool>>>,
    pub win_tool_last_check: std::time::Instant,
    pub external_change_conflict: Option<PathBuf>,
    pub dep_wizard: crate::app::ui::dialogs::DependencyWizard,
    pub terminal_close_requested: Option<usize>,
    pub ai_viewport_open: bool,
    pub plugin_error: Option<String>,
    pub settings_conflict: Option<SettingsConflict>,
    pub ai: AiState,
    pub git_cancel: Arc<AtomicBool>,
    pub local_history: crate::app::local_history::LocalHistory,
    pub background_io_rx: Option<mpsc::Receiver<FsChangeResult>>,
    pub applied_settings_version: u64,
    pub pending_plugin_approval: Option<PendingPluginApproval>,
    /// Pending ask_user request: (plugin_id, question, options, answer_input_buffer, sender)
    pub pending_ask_user: Option<PendingAskUser>,
    /// Pending discard changes confirmation for a specific modal ID.
    pub confirm_discard_changes: Option<String>,
    /// Last time the user pressed a key. Used for repaint capping during active typing.
    pub last_keystroke_time: Option<std::time::Instant>,
    // --- Ollama native provider state ---
    pub ollama_status: OllamaConnectionStatus,
    pub ollama_models: Vec<String>,
    pub ollama_selected_model: String,
    pub ollama_check_rx: Option<mpsc::Receiver<OllamaStatus>>,
    pub ollama_last_check: std::time::Instant,
    pub ollama_base_url: String,
    pub ollama_api_key: Option<String>,
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
        self.retired_terminals
            .retain(|terminal| !terminal.is_exited());
    }

    pub fn retire_terminal(&mut self, mut terminal: Terminal) {
        terminal.request_graceful_exit();
        self.retired_terminals.push(terminal);
    }

}
