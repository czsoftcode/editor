use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{mpsc, Arc, Mutex};

use eframe::egui;

use super::super::super::build_runner::BuildError;
use super::super::super::types::{
    AiTool, FocusedPanel, PersistentState, Toast, default_wizard_path,
};
use super::super::background::{fetch_git_branch, fetch_git_status};
use super::super::dialogs::WizardState;
use super::super::editor::Editor;
use super::super::file_tree::FileTree;
use super::super::terminal::Terminal;
use super::super::widgets::command_palette::CommandPaletteState;
use crate::watcher::{FileWatcher, ProjectWatcher};

/// Result of an asynchronous folder selection.
/// bool = true → open in a new window; false → replace current workspace.
pub(super) type FolderPickResult = (Option<PathBuf>, bool);

// ---------------------------------------------------------------------------
// FilePicker — Ctrl+P quick file opening
// ---------------------------------------------------------------------------

pub(crate) struct FilePicker {
    pub query: String,
    /// All project files (relative paths)
    pub files: Vec<PathBuf>,
    /// Indices into `files` matching the current filter
    pub filtered: Vec<usize>,
    /// Currently highlighted item in the list
    pub selected: usize,
    pub focus_requested: bool,
}

impl FilePicker {
    pub(super) fn new(files: Vec<PathBuf>) -> Self {
        let filtered: Vec<usize> = (0..files.len()).collect();
        Self {
            query: String::new(),
            files,
            filtered,
            selected: 0,
            focus_requested: true,
        }
    }

    pub(crate) fn update_filter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = self
            .files
            .iter()
            .enumerate()
            .filter(|(_, p)| super::super::search_picker::fuzzy_match(&q, &p.to_string_lossy()))
            .map(|(i, _)| i)
            .collect();
        self.selected = 0;
    }
}

// ---------------------------------------------------------------------------
// ProjectSearch — project-wide search
// ---------------------------------------------------------------------------

pub(crate) struct SearchResult {
    pub file: PathBuf,
    pub line: usize,
    pub text: String,
}

pub(crate) struct ProjectSearch {
    pub show_input: bool,
    pub query: String,
    pub results: Vec<SearchResult>,
    pub rx: Option<mpsc::Receiver<Vec<SearchResult>>>,
    pub focus_requested: bool,
    pub cancel_epoch: Arc<AtomicU64>,
}

impl Default for ProjectSearch {
    fn default() -> Self {
        Self {
            show_input: false,
            query: String::new(),
            results: Vec::new(),
            rx: None,
            focus_requested: false,
            cancel_epoch: Arc::new(AtomicU64::new(0)),
        }
    }
}

// ---------------------------------------------------------------------------
// WorkspaceState — state of a single workspace (project window)
// ---------------------------------------------------------------------------

pub(crate) struct WorkspaceState {
    pub file_tree: FileTree,
    pub editor: Editor,
    pub watcher: FileWatcher,
    pub project_watcher: ProjectWatcher,
    pub claude_tabs: Vec<Terminal>,
    pub claude_active_tab: usize,
    pub next_claude_tab_id: u64,
    pub build_terminal: Option<Terminal>,
    pub focused_panel: FocusedPanel,
    pub root_path: PathBuf,
    pub show_left_panel: bool,
    pub show_right_panel: bool,
    pub show_build_terminal: bool,
    pub show_about: bool,
    pub show_settings: bool,
    pub ai_font_scale: u32,
    pub build_errors: Vec<BuildError>,
    pub build_error_rx: Option<mpsc::Receiver<Vec<BuildError>>>,
    pub claude_tool: AiTool,
    pub claude_float: bool,
    // New project wizard (for this window)
    pub show_new_project: bool,
    pub wizard: WizardState,
    pub toasts: Vec<Toast>,
    /// Channel for the result of an asynchronous file dialog (folder selection).
    pub folder_pick_rx: Option<mpsc::Receiver<FolderPickResult>>,
    /// Ctrl+Shift+P — command palette
    pub command_palette: Option<CommandPaletteState>,
    /// Shared file index for Ctrl+P, search, etc.
    pub project_index: Arc<super::ProjectIndex>,
    /// Ctrl+P — fuzzy file picker
    pub file_picker: Option<FilePicker>,
    /// Project-wide search
    pub project_search: ProjectSearch,
    /// Git — current branch
    pub git_branch: Option<String>,
    pub git_branch_rx: Option<mpsc::Receiver<Option<String>>>,
    /// Git — file status (absolute path → color for file tree)
    pub git_status_rx: Option<mpsc::Receiver<std::collections::HashMap<PathBuf, egui::Color32>>>,
    /// Timer for periodic git refresh
    pub git_last_refresh: std::time::Instant,
    /// Settings draft — initialized when dialog opens, discarded when closed
    pub settings_draft: Option<crate::settings::Settings>,
    /// Asynchronous default projects path selection in settings dialog
    pub settings_folder_pick_rx: Option<mpsc::Receiver<Option<PathBuf>>>,
    /// Availability of AI CLI tools (according to PATH)
    pub ai_tool_available: HashMap<AiTool, bool>,
    /// Asynchronous AI CLI tool availability check
    pub ai_tool_check_rx: Option<mpsc::Receiver<HashMap<AiTool, bool>>>,
    /// Time of the last (automatic) AI CLI tool re-detection
    pub ai_tool_last_check: std::time::Instant,
    /// Pending conflict: file was modified externally, but tab has unsaved changes.
    /// Value = path to conflict; None = no conflict.
    pub external_change_conflict: Option<PathBuf>,
    /// Cancellation flag for git refresh threads.
    /// Set to true on workspace drop → threads terminate git process and do not process result.
    pub git_cancel: Arc<AtomicBool>,
}

impl Drop for WorkspaceState {
    fn drop(&mut self) {
        // Signal git refresh threads to terminate git process and not return result.
        self.git_cancel.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

// ---------------------------------------------------------------------------
// SecondaryWorkspace — secondary viewport (one project in a new window)
// ---------------------------------------------------------------------------

pub(crate) struct SecondaryWorkspace {
    pub viewport_id: egui::ViewportId,
    pub state: Arc<Mutex<WorkspaceState>>,
    pub close_requested: Arc<AtomicBool>,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

pub(crate) fn ws_to_panel_state(ws: &WorkspaceState) -> PersistentState {
    PersistentState {
        show_left_panel: ws.show_left_panel,
        show_right_panel: ws.show_right_panel,
        show_build_terminal: ws.show_build_terminal,
        claude_float: ws.claude_float,
        ai_font_scale: ws.ai_font_scale,
    }
}

fn is_command_available(command: &str) -> bool {
    #[cfg(windows)]
    {
        return std::process::Command::new("cmd")
            .args(["/C", "where", command])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("sh")
            .args(["-lc", &format!("command -v {command} >/dev/null 2>&1")])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

pub(crate) fn spawn_ai_tool_check() -> mpsc::Receiver<HashMap<AiTool, bool>> {
    crate::app::ui::background::spawn_task(move || {
        let mut status = HashMap::new();
        for tool in AiTool::ALL {
            status.insert(tool, is_command_available(tool.command()));
        }
        status
    })
}

pub(crate) fn init_workspace(root_path: PathBuf, panel_state: &PersistentState) -> WorkspaceState {
    let mut file_tree = FileTree::new();
    file_tree.load(&root_path);
    let project_watcher = ProjectWatcher::new(&root_path);
    let git_cancel = Arc::new(AtomicBool::new(false));
    let git_branch_rx = fetch_git_branch(&root_path, Arc::clone(&git_cancel));
    let git_status_rx = fetch_git_status(&root_path, Arc::clone(&git_cancel));
    let project_index = Arc::new(super::ProjectIndex::new(root_path.clone()));
    project_index.full_rescan();
    let ai_tool_check_rx = spawn_ai_tool_check();
    let mut wizard = WizardState::default();
    wizard.path = default_wizard_path();

    WorkspaceState {
        file_tree,
        editor: Editor::new(),
        watcher: FileWatcher::new(),
        project_watcher,
        claude_tabs: Vec::new(),
        claude_active_tab: 0,
        next_claude_tab_id: 100,
        build_terminal: None,
        focused_panel: FocusedPanel::Editor,
        root_path,
        show_left_panel: panel_state.show_left_panel,
        show_right_panel: panel_state.show_right_panel,
        show_build_terminal: panel_state.show_build_terminal,
        show_about: false,
        show_settings: false,
        ai_font_scale: panel_state.ai_font_scale,
        build_errors: Vec::new(),
        build_error_rx: None,
        claude_tool: AiTool::ClaudeCode,
        claude_float: panel_state.claude_float,
        show_new_project: false,
        wizard,
        toasts: Vec::new(),
        folder_pick_rx: None,
        command_palette: None,
        project_index,
        file_picker: None,
        project_search: ProjectSearch::default(),
        git_branch: None,
        git_branch_rx: Some(git_branch_rx),
        git_status_rx: Some(git_status_rx),
        git_last_refresh: std::time::Instant::now(),
        settings_draft: None,
        settings_folder_pick_rx: None,
        ai_tool_available: HashMap::new(),
        ai_tool_check_rx: Some(ai_tool_check_rx),
        ai_tool_last_check: std::time::Instant::now(),
        external_change_conflict: None,
        git_cancel,
    }
}

pub(crate) fn open_file_in_ws(ws: &mut WorkspaceState, path: PathBuf) {
    ws.editor.open_file(&path);
    if let Some(parent) = path.parent() {
        ws.watcher.watch(parent);
    }
}

pub(crate) fn open_and_jump(ws: &mut WorkspaceState, path: PathBuf, line: usize) {
    open_file_in_ws(ws, path);
    ws.editor.jump_to_line(line);
}
