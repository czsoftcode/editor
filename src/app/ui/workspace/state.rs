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
use super::super::search_picker::collect_project_files;
use super::super::terminal::Terminal;
use crate::watcher::{FileWatcher, ProjectWatcher};

/// Výsledek asynchronního výběru složky.
/// bool = true → otevřít v novém okně; false → nahradit aktuální workspace.
pub(super) type FolderPickResult = (Option<PathBuf>, bool);

// ---------------------------------------------------------------------------
// FilePicker — Ctrl+P rychlé otevření souboru
// ---------------------------------------------------------------------------

pub(crate) struct FilePicker {
    pub query: String,
    /// Všechny soubory projektu (relativní cesty)
    pub files: Vec<PathBuf>,
    /// Indexy do `files` odpovídající aktuálnímu filtru
    pub filtered: Vec<usize>,
    /// Aktuálně označená položka v seznamu
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
// ProjectSearch — hledání napříč projektem
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
// WorkspaceState — stav jednoho pracovního prostoru (okna projektu)
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
    // Wizard nového projektu (pro toto okno)
    pub show_new_project: bool,
    pub wizard: WizardState,
    pub toasts: Vec<Toast>,
    /// Kanál pro výsledek asynchronního file dialogu (výběr složky).
    pub folder_pick_rx: Option<mpsc::Receiver<FolderPickResult>>,
    /// Ctrl+P — fuzzy file picker
    pub file_picker: Option<FilePicker>,
    /// Cache indexu souborů pro Ctrl+P (relativní cesty)
    pub file_index_cache: Vec<PathBuf>,
    /// Probíhající background scan souborů
    pub file_index_rx: Option<mpsc::Receiver<Vec<PathBuf>>>,
    /// Hledání napříč projektem
    pub project_search: ProjectSearch,
    /// Git — aktuální větev
    pub git_branch: Option<String>,
    pub git_branch_rx: Option<mpsc::Receiver<Option<String>>>,
    /// Git — stav souborů (absolutní cesta → barva pro file tree)
    pub git_status_rx: Option<mpsc::Receiver<std::collections::HashMap<PathBuf, egui::Color32>>>,
    /// Časovač pro periodický refresh gitu
    pub git_last_refresh: std::time::Instant,
    /// Draft nastavení — inicializuje se při otevření dialogu, zahazuje se při zavření
    pub settings_draft: Option<crate::settings::Settings>,
    /// Asynchronní výběr výchozí cesty projektů v dialogu nastavení
    pub settings_folder_pick_rx: Option<mpsc::Receiver<Option<PathBuf>>>,
    /// Dostupnost AI CLI nástrojů (podle PATH)
    pub ai_tool_available: HashMap<AiTool, bool>,
    /// Asynchronní kontrola dostupnosti AI CLI nástrojů
    pub ai_tool_check_rx: Option<mpsc::Receiver<HashMap<AiTool, bool>>>,
    /// Čas poslední (automatické) re-detekce AI CLI nástrojů
    pub ai_tool_last_check: std::time::Instant,
    /// Čekající konflikt: soubor byl změněn externě, ale záložka má neuložené změny.
    /// Hodnota = cesta ke konfliktu; None = žádný konflikt.
    pub external_change_conflict: Option<PathBuf>,
    /// Zrušovací příznak pro git refresh vlákna.
    /// Při drop workspacu se nastaví na true → vlákna ukončí git proces a nezpracují výsledek.
    pub git_cancel: Arc<AtomicBool>,
}

impl Drop for WorkspaceState {
    fn drop(&mut self) {
        // Signalizujeme git refresh vláknům, aby ukončila git proces a nevracela výsledek.
        self.git_cancel.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

// ---------------------------------------------------------------------------
// SecondaryWorkspace — sekundární viewport (jeden projekt v novém okně)
// ---------------------------------------------------------------------------

pub(crate) struct SecondaryWorkspace {
    pub viewport_id: egui::ViewportId,
    pub state: Arc<Mutex<WorkspaceState>>,
    pub close_requested: Arc<AtomicBool>,
}

// ---------------------------------------------------------------------------
// Pomocné funkce
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

pub(crate) fn spawn_file_index_scan(root: PathBuf) -> mpsc::Receiver<Vec<PathBuf>> {
    crate::app::ui::background::spawn_task(move || collect_project_files(&root))
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
    let file_index_rx = spawn_file_index_scan(root_path.clone());
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
        file_picker: None,
        file_index_cache: Vec::new(),
        file_index_rx: Some(file_index_rx),
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
