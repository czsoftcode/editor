pub mod actions;
pub mod init;
pub mod types;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, mpsc};

use crate::app::ai_prefs::AiPanelState;
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

#[derive(Clone)]
pub struct SettingsConflict {
    pub new_settings: crate::settings::Settings,
}

/// Mode of the pending unsaved close flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingCloseMode {
    /// Guard flow initiated for a single active tab.
    SingleTab,
    /// Guard flow initiated for closing the whole workspace/project.
    WorkspaceClose,
}

/// Input mode used for building the unsaved-close queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtyCloseQueueMode<'a> {
    /// Build queue for closing exactly one specific tab.
    SingleTab(&'a PathBuf),
    /// Build queue for workspace-level close (all dirty tabs).
    WorkspaceClose(Option<&'a PathBuf>),
}

/// State for the unsaved close guard flow.
#[derive(Debug, Clone)]
pub struct PendingCloseFlow {
    /// Whether the guard was triggered for a single tab or full workspace close.
    pub mode: PendingCloseMode,
    /// Snapshot queue of dirty tab paths to process in order.
    pub queue: Vec<PathBuf>,
    /// Index of the currently processed item in `queue`.
    pub current_index: usize,
    /// Inline error message for the current item, if saving failed.
    pub inline_error: Option<String>,
}

pub struct WorkspaceState {
    pub file_tree: FileTree,
    pub editor: Editor,
    pub watcher: FileWatcher,
    pub project_watcher: ProjectWatcher,
    pub project_watcher_active: bool,
    pub project_watcher_disconnect_reported: bool,
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
    pub ai_panel: AiPanelState,
    pub git_cancel: Arc<AtomicBool>,
    pub local_history: crate::app::local_history::LocalHistory,
    pub background_io_tx: mpsc::Sender<FsChangeResult>,
    pub background_io_rx: Option<mpsc::Receiver<FsChangeResult>>,
    pub applied_settings_version: u64,
    /// Pending discard changes confirmation for a specific modal ID.
    pub confirm_discard_changes: Option<String>,
    /// Last time the user pressed a key. Used for repaint capping during active typing.
    pub last_keystroke_time: Option<std::time::Instant>,
    /// Pending unsaved close guard flow, if any.
    pub pending_close_flow: Option<PendingCloseFlow>,
    /// Whether the last workspace-level unsaved close guard run (WorkspaceClose mode)
    /// was cancelled by the user. Used by root close orchestration to decide whether
    /// to proceed with Quit/Close Project/window close.
    pub last_unsaved_close_cancelled: bool,
    /// Stav otevřeného history panelu (None = panel není zobrazen).
    pub history_view: Option<crate::app::ui::workspace::history::HistoryViewState>,
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

    /// Pokud je aktivní soubor profiles.toml, načte znovu profily do build baru.
    pub fn refresh_profiles_if_active_path(&mut self) {
        let profiles_path = crate::app::project_config::profiles_path(&self.root_path);
        if self
            .editor
            .active_path()
            .is_some_and(|path| *path == profiles_path)
        {
            self.profiles = crate::app::project_config::load_profiles(&self.root_path);
        }
    }
}

/// Builds a stable queue of dirty tab paths for the unsaved close guard.
///
/// The queue always starts with the active tab (if it is dirty), followed by
/// the remaining dirty tabs in a deterministic order.
pub fn build_dirty_close_queue_for_mode(
    mode: DirtyCloseQueueMode<'_>,
    tabs: &[(PathBuf, bool)],
) -> Vec<PathBuf> {
    match mode {
        DirtyCloseQueueMode::SingleTab(target) => tabs
            .iter()
            .find_map(|(path, modified)| {
                if path == target && *modified {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .into_iter()
            .collect(),
        DirtyCloseQueueMode::WorkspaceClose(active_path) => {
            let mut dirty: Vec<PathBuf> = tabs
                .iter()
                .filter(|(_, modified)| *modified)
                .map(|(path, _)| path.clone())
                .collect();

            if dirty.is_empty() {
                return Vec::new();
            }

            // Sort to ensure deterministic order independent of input ordering.
            dirty.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

            if let Some(active) = active_path
                && let Some(pos) = dirty.iter().position(|p| p == active)
            {
                let active_entry = dirty.remove(pos);
                let mut ordered = Vec::with_capacity(dirty.len() + 1);
                ordered.push(active_entry);
                ordered.extend(dirty);
                return ordered;
            }

            dirty
        }
    }
}

pub fn build_dirty_close_queue(
    active_path: Option<&PathBuf>,
    tabs: &[(PathBuf, bool)],
) -> Vec<PathBuf> {
    build_dirty_close_queue_for_mode(DirtyCloseQueueMode::WorkspaceClose(active_path), tabs)
}

pub fn resolve_runtime_model(models: &[String], current: &str, preferred: &str) -> String {
    if models.iter().any(|m| m == current) {
        return current.to_string();
    }
    if !preferred.is_empty() && models.iter().any(|m| m == preferred) {
        return preferred.to_string();
    }
    if let Some(first) = models.first() {
        return first.clone();
    }
    current.to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        DirtyCloseQueueMode, build_dirty_close_queue, build_dirty_close_queue_for_mode,
        resolve_runtime_model,
    };
    use std::path::PathBuf;

    #[test]
    fn unsaved_close_guard_queue_single_tab_target() {
        let a = PathBuf::from("/project/a.txt");
        let b = PathBuf::from("/project/b.txt");
        let c = PathBuf::from("/project/c.txt");

        let tabs = vec![(b.clone(), true), (a.clone(), true), (c.clone(), false)];

        let queue = build_dirty_close_queue_for_mode(DirtyCloseQueueMode::SingleTab(&a), &tabs);
        assert_eq!(queue, vec![a]);
    }

    #[test]
    fn unsaved_close_guard_queue() {
        let a = PathBuf::from("/project/a.txt");
        let b = PathBuf::from("/project/b.txt");
        let c = PathBuf::from("/project/c.txt");

        let tabs = vec![(b.clone(), true), (a.clone(), true), (c.clone(), false)];

        let queue = build_dirty_close_queue(Some(&b), &tabs);
        assert_eq!(queue.first(), Some(&b));
        assert_eq!(queue.len(), 2);
        assert!(queue.contains(&a));
        assert!(!queue.contains(&c));
    }

    #[test]
    fn resolve_runtime_model_prefers_current_when_available() {
        let models = vec!["llama3.1".to_string(), "qwen3:latest".to_string()];
        assert_eq!(
            resolve_runtime_model(&models, "qwen3:latest", "llama3.1"),
            "qwen3:latest"
        );
    }

    #[test]
    fn resolve_runtime_model_falls_back_to_preferred_then_first() {
        let models = vec!["llama3.1".to_string(), "qwen3:latest".to_string()];
        assert_eq!(
            resolve_runtime_model(&models, "missing", "llama3.1"),
            "llama3.1"
        );
        assert_eq!(
            resolve_runtime_model(&models, "missing", "also-missing"),
            "llama3.1"
        );
    }

    #[test]
    fn resolve_runtime_model_keeps_current_when_no_models_available() {
        let models: Vec<String> = Vec::new();
        assert_eq!(
            resolve_runtime_model(&models, "existing-model", "preferred"),
            "existing-model"
        );
    }
}
