pub mod actions;
pub mod init;
pub mod types;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, mpsc};

use crate::app::cli::AiState;
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

use std::collections::HashSet;

/// Pending native tool approval request (shown in chat UI).
pub struct PendingToolApproval {
    pub tool_call_id: String,
    pub tool_name: String,
    pub description: String,
    /// Diff preview or command text.
    pub details: String,
    pub is_network: bool,
    pub is_new_file: bool,
    /// Original args for execute_approved.
    pub args: serde_json::Value,
    /// Send approval decision: true = approve, false = deny.
    pub response_tx: mpsc::Sender<bool>,
}

/// Pending ask_user request from the AI tool executor.
pub struct PendingToolAsk {
    pub question: String,
    pub options: Vec<String>,
    /// Send user's text response.
    pub response_tx: mpsc::Sender<String>,
    /// Input buffer for free-text field.
    pub input_buffer: String,
}

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
    pub show_ai_chat: bool,
    pub show_semantic_indexing_modal: bool,
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
    pub settings_conflict: Option<SettingsConflict>,
    pub ai: AiState,
    pub git_cancel: Arc<AtomicBool>,
    pub local_history: crate::app::local_history::LocalHistory,
    pub background_io_rx: Option<mpsc::Receiver<FsChangeResult>>,
    pub applied_settings_version: u64,
    /// Pending discard changes confirmation for a specific modal ID.
    pub confirm_discard_changes: Option<String>,
    /// Last time the user pressed a key. Used for repaint capping during active typing.
    pub last_keystroke_time: Option<std::time::Instant>,

    // --- Native tool execution state (Phase 16) ---
    /// Native tool executor for AI tool calls (lazily initialized on first AI chat).
    pub tool_executor: Option<crate::app::cli::executor::ToolExecutor>,
    /// Pending native tool approval request.
    pub pending_tool_approval: Option<PendingToolApproval>,
    /// Pending native ask_user request from tool executor.
    pub pending_tool_ask: Option<PendingToolAsk>,
    /// Tool names that user chose "Always approve" for (runtime only).
    pub tool_always_approved: HashSet<String>,
    /// Receiver for native tool approval responses from the UI.
    pub tool_approval_rx: Option<mpsc::Receiver<bool>>,
    /// Receiver for native ask_user responses from the UI.
    pub tool_ask_rx: Option<mpsc::Receiver<String>>,

    // --- Slash command async state (Phase 19) ---
    /// Receiver for slash /build async result.
    pub slash_build_rx: Option<mpsc::Receiver<Vec<crate::app::build_runner::BuildError>>>,
    /// Receiver for slash /git async result.
    pub slash_git_rx: Option<mpsc::Receiver<String>>,
    /// Generation counter for conversation — incremented on /clear and /new to detect stale async results.
    pub slash_conversation_gen: u64,
    /// Generation at which the slash build was started (to detect if conversation was cleared).
    pub slash_build_gen: u64,
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
