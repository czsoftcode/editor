use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex, mpsc};

use eframe::egui;

use super::super::super::build_runner::BuildError;
use super::super::super::types::{
    FocusedPanel, PersistentState, ProjectProfiles, Toast, default_wizard_path,
};
use super::super::background::{fetch_git_branch, fetch_git_status};
use super::super::dialogs::WizardState;
use super::super::editor::Editor;
use super::super::file_tree::FileTree;
use super::super::terminal::Terminal;
use super::super::widgets::command_palette::CommandPaletteState;
use crate::app::lsp::LspClient;
use crate::app::project_config::load_profiles;
use crate::watcher::{FileWatcher, ProjectWatcher};
use async_lsp::lsp_types::Url;

/// Result of an asynchronous folder selection.
pub(super) type FolderPickResult = (Option<PathBuf>, bool);

// ---------------------------------------------------------------------------
// FilePicker — Ctrl+P quick file opening
// ---------------------------------------------------------------------------

pub(crate) struct FilePicker {
    pub query: String,
    pub files: Arc<Vec<PathBuf>>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub focus_requested: bool,
}

impl FilePicker {
    pub(super) fn new(files: Arc<Vec<PathBuf>>) -> Self {
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
// Background I/O Results
// ---------------------------------------------------------------------------

pub(crate) enum FsChangeResult {
    AiDiff(String, String, String),
    #[allow(dead_code)]
    LocalHistory(PathBuf, String),
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
    pub next_terminal_id: u64,
    pub build_terminal: Option<Terminal>,
    pub focused_panel: FocusedPanel,
    pub root_path: PathBuf,
    pub show_left_panel: bool,
    pub show_right_panel: bool,
    pub show_build_terminal: bool,
    pub show_about: bool,
    pub show_settings: bool,
    pub show_plugins: bool,
    pub show_gemini: bool,
    pub gemini_show_settings: bool,
    /// Currently selected plugin in the Plugins modal.
    pub selected_plugin_id: Option<String>,
    /// Currently selected category in the Settings modal.
    pub selected_settings_category: Option<String>,
    pub ai_font_scale: u32,
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
    pub project_index: Arc<super::ProjectIndex>,
    pub file_picker: Option<FilePicker>,
    pub project_search: ProjectSearch,
    pub lsp_client: Option<LspClient>,
    pub lsp_binary_missing: bool,
    pub lsp_install_rx: Option<mpsc::Receiver<Result<(), String>>>,
    pub git_branch: Option<String>,
    pub git_branch_rx: Option<mpsc::Receiver<Option<String>>>,
    pub git_status_rx: Option<mpsc::Receiver<std::collections::HashMap<PathBuf, egui::Color32>>>,
    pub git_last_refresh: std::time::Instant,
    pub lsp_last_retry: std::time::Instant,
    pub settings_draft: Option<crate::settings::Settings>,
    pub plugins_draft: Option<crate::settings::Settings>,
    pub settings_folder_pick_rx: Option<mpsc::Receiver<Option<PathBuf>>>,
    pub ai_tool_available: HashMap<String, bool>,
    pub ai_tool_check_rx: Option<mpsc::Receiver<HashMap<String, bool>>>,
    pub ai_tool_last_check: std::time::Instant,
    pub external_change_conflict: Option<PathBuf>,
    /// Pending sync: file was deleted in sandbox but still exists in project.
    /// Value = relative path to file.
    pub sandbox_deletion_sync: Option<PathBuf>,
    pub terminal_close_requested: Option<usize>,
    pub ai_viewport_open: bool,
    pub promotion_success: Option<PathBuf>,
    pub show_sandbox_staged: bool,
    pub plugin_error: Option<String>,
    pub gemini_prompt: String,
    pub gemini_history: Vec<String>,
    pub gemini_history_index: Option<usize>,
    pub gemini_monologue: Vec<String>,
    pub gemini_conversation: Vec<(String, String)>,
    pub gemini_system_prompt: String,
    pub gemini_language: String,
    pub gemini_total_tokens: u32,
    pub gemini_response: Option<String>,
    pub gemini_loading: bool,
    pub markdown_cache: egui_commonmark::CommonMarkCache,
    pub sync_confirmation: Option<crate::app::sandbox::SyncPlan>,
    pub pending_agent_id: Option<String>,
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
    /// Last settings version applied to this viewport's context (Audit S-4).
    pub applied_settings_version: u64,
}

impl Drop for WorkspaceState {
    fn drop(&mut self) {
        self.git_cancel
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

pub(crate) struct SecondaryWorkspace {
    pub viewport_id: egui::ViewportId,
    pub state: Arc<Mutex<WorkspaceState>>,
    pub close_requested: Arc<AtomicBool>,
}

pub(crate) fn ws_to_panel_state(ws: &WorkspaceState) -> PersistentState {
    PersistentState {
        show_left_panel: ws.show_left_panel,
        show_right_panel: ws.show_right_panel,
        show_build_terminal: ws.show_build_terminal,
        claude_float: ws.claude_float,
        ai_font_scale: ws.ai_font_scale,
        gemini_system_prompt: Some(ws.gemini_system_prompt.clone()),
        gemini_language: Some(ws.gemini_language.clone()),
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

pub(crate) fn spawn_ai_tool_check(
    agents: Vec<(String, String)>,
) -> mpsc::Receiver<HashMap<String, bool>> {
    crate::app::ui::background::spawn_task(move || {
        let mut status = HashMap::new();
        for (id, cmd) in agents {
            status.insert(id, is_command_available(&cmd));
        }
        status
    })
}

pub(crate) fn init_workspace(
    root_path: PathBuf,
    panel_state: &PersistentState,
    egui_ctx: egui::Context,
    settings: &crate::settings::Settings,
) -> WorkspaceState {
    let mut file_tree = FileTree::new();
    file_tree.load(&root_path);
    let mut project_watcher = ProjectWatcher::new(&root_path);
    let sandbox = crate::app::sandbox::Sandbox::new(&root_path);
    project_watcher.add_path(&sandbox.root);
    let sandbox_staged_files = sandbox.get_staged_files();

    let git_cancel = Arc::new(AtomicBool::new(false));
    let git_branch_rx = fetch_git_branch(&root_path, Arc::clone(&git_cancel));
    let git_status_rx = fetch_git_status(&root_path, Arc::clone(&git_cancel));
    let project_index = Arc::new(super::ProjectIndex::new(root_path.clone()));
    project_index.full_rescan();
    let profiles = load_profiles(&root_path);
    let mut wizard = WizardState::default();
    wizard.path = default_wizard_path();

    let is_rust = root_path.join("Cargo.toml").exists();
    let lsp_installed = LspClient::is_installed();

    let (lsp_client, lsp_binary_missing) = if is_rust {
        if lsp_installed {
            let root_uri = Url::from_directory_path(&root_path).expect("valid root path for Url");
            let client = LspClient::new(egui_ctx.clone(), root_uri);
            let missing = client.is_none();
            (client, missing)
        } else {
            (None, true)
        }
    } else {
        (None, false)
    };

    let local_history = crate::app::local_history::LocalHistory::new(&root_path);
    local_history.cleanup(50);

    let i18n = crate::i18n::I18n::from_system();

    let gemini_model = settings
        .plugins
        .get("gemini")
        .and_then(|s| s.config.get("MODEL").cloned())
        .unwrap_or_else(|| "gemini-1.5-flash".to_string());

    WorkspaceState {
        file_tree,
        editor: Editor::new(),
        watcher: FileWatcher::new(),
        project_watcher,
        claude_tabs: Vec::new(),
        claude_active_tab: 0,
        next_claude_tab_id: 100,
        next_terminal_id: 1000,
        build_terminal: None,
        focused_panel: FocusedPanel::Editor,
        root_path: root_path.clone(),
        show_left_panel: panel_state.show_left_panel,
        show_right_panel: panel_state.show_right_panel,
        show_build_terminal: panel_state.show_build_terminal,
        show_about: false,
        show_settings: false,
        show_plugins: false,
        show_gemini: false,
        gemini_show_settings: false,
        selected_plugin_id: None,
        selected_settings_category: None,
        ai_font_scale: panel_state.ai_font_scale,
        profiles,
        build_errors: Vec::new(),
        build_error_rx: None,
        selected_agent_id: "gemini".to_string(),
        claude_float: panel_state.claude_float,
        show_new_project: false,
        wizard,
        toasts: Vec::new(),
        folder_pick_rx: None,
        command_palette: None,
        project_index,
        file_picker: None,
        project_search: ProjectSearch::default(),
        lsp_client,
        lsp_binary_missing,
        lsp_install_rx: None,
        git_branch: None,
        git_branch_rx: Some(git_branch_rx),
        git_status_rx: Some(git_status_rx),
        git_last_refresh: std::time::Instant::now(),
        lsp_last_retry: std::time::Instant::now(),
        settings_draft: None,
        plugins_draft: None,
        settings_folder_pick_rx: None,
        ai_tool_available: HashMap::new(),
        ai_tool_check_rx: None,
        ai_tool_last_check: std::time::Instant::now(),
        external_change_conflict: None,
        sandbox_deletion_sync: None,
        terminal_close_requested: None,
        ai_viewport_open: false,
        promotion_success: None,
        show_sandbox_staged: false,
        plugin_error: None,
        gemini_prompt: String::new(),
        gemini_history: Vec::new(),
        gemini_history_index: None,
        gemini_monologue: Vec::new(),
        gemini_conversation: vec![(
            String::new(),
            format!(
                r#"    ____        __      ______              __
   / __ \____  / /_  __/ ____/_______  ____/ /___
  / /_/ / __ \/ / / / / /   / ___/ _ \/ __  / __ \
 / ____/ /_/ / / /_/ / /___/ /  /  __/ /_/ / /_/ /
/_/    \____/_/\__, /\____/_/   \___/\__,_/\____/
              /____/                              CLI

 Version: {}
 Model:   {}
 Plan:    {}"#,
                crate::config::CLI_VERSION,
                gemini_model,
                crate::config::CLI_TIER
            ),
        )],
        gemini_system_prompt: panel_state
            .gemini_system_prompt
            .clone()
            .unwrap_or_else(|| i18n.get("gemini-default-prompt")),
        gemini_language: panel_state
            .gemini_language
            .clone()
            .unwrap_or_else(|| i18n.lang().to_string()),
        gemini_total_tokens: 0,
        gemini_response: None,
        gemini_loading: false,
        markdown_cache: egui_commonmark::CommonMarkCache::default(),
        sync_confirmation: None,
        pending_agent_id: None,
        build_in_sandbox: false,
        file_tree_in_sandbox: false,
        git_cancel,
        local_history,
        sandbox,
        sandbox_staged_files,
        sandbox_staged_rx: None,
        sandbox_staged_dirty: true,
        sandbox_staged_last_dirty: std::time::Instant::now(),
        sandbox_staged_last_refresh: std::time::Instant::now(),
        background_io_rx: None,
        applied_settings_version: 0,
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
