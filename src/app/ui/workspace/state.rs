use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};

use eframe::egui;

use super::super::super::build_runner::BuildError;
use super::super::super::types::{FocusedPanel, PersistentState, ProjectProfiles, Toast};
use super::super::background::{fetch_git_branch, fetch_git_status};
use super::super::dialogs::WizardState;
use super::super::editor::Editor;
use super::super::file_tree::FileTree;
use super::super::terminal::Terminal;
use super::super::widgets::command_palette::CommandPaletteState;
use crate::app::lsp::LspClient;
use crate::app::project_config::load_profiles;
use crate::watcher::FileWatcher;
use crate::watcher::ProjectWatcher;

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
    LocalHistory(PathBuf, String),
}

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
    pub show_semantic_indexing_modal: bool,
    pub gemini_show_settings: bool,
    pub selected_plugin_id: Option<String>,
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
    pub semantic_index: Arc<Mutex<super::semantic_index::SemanticIndex>>,
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
    pub gemini_inspector_open: bool,
    pub gemini_focus_requested: bool,
    pub gemini_last_payload: String,
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

pub fn spawn_ai_tool_check(
    check_list: Vec<(String, String)>,
) -> mpsc::Receiver<HashMap<String, bool>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut results = HashMap::new();
        for (id, cmd) in check_list {
            let found = std::process::Command::new("which")
                .arg(&cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            results.insert(id, found);
        }
        let _ = tx.send(results);
    });
    rx
}

pub(crate) fn open_and_jump(ws: &mut WorkspaceState, path: PathBuf, line: usize) {
    super::open_file_in_ws(ws, path);
    ws.editor.jump_to_location(line, 1);
    ws.focused_panel = FocusedPanel::Editor;
}

pub(crate) fn open_file_in_ws(ws: &mut WorkspaceState, path: PathBuf) {
    if !path.exists() {
        return;
    }
    if let Some(existing_idx) = ws.editor.tabs.iter().position(|t| t.path == path) {
        ws.editor.active_tab = Some(existing_idx);
    } else {
        ws.editor.open_file(&path);
    }
    ws.focused_panel = FocusedPanel::Editor;
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

pub(crate) fn init_workspace(
    root_path: PathBuf,
    panel_state: &PersistentState,
    egui_ctx: egui::Context,
    settings: &crate::settings::Settings,
) -> WorkspaceState {
    let sandbox = crate::app::sandbox::Sandbox::new(&root_path);
    let mut file_tree = FileTree::new();
    let file_tree_in_sandbox = settings.project_read_only;
    let target_tree_root = if file_tree_in_sandbox {
        &sandbox.root
    } else {
        &root_path
    };
    file_tree.load(target_tree_root);

    let mut project_watcher = ProjectWatcher::new(&root_path);
    project_watcher.add_path(&sandbox.root);
    let sandbox_staged_files = sandbox.get_staged_files();

    let git_cancel = Arc::new(AtomicBool::new(false));
    let git_branch_rx = fetch_git_branch(&root_path, Arc::clone(&git_cancel));
    let git_status_rx = fetch_git_status(&root_path, Arc::clone(&git_cancel));

    let project_index = Arc::new(super::ProjectIndex::new(root_path.clone()));
    let semantic_index = Arc::new(Mutex::new(super::semantic_index::SemanticIndex::new(
        root_path.clone(),
    )));
    project_index.full_rescan();

    // Start semantic index initialization and indexing in background
    let si_clone = Arc::clone(&semantic_index);
    let pi_clone = Arc::clone(&project_index);
    let thread_root = root_path.clone();
    let ctx_clone = egui_ctx.clone();
    let blacklist_patterns = settings.blacklist.clone();

    std::thread::spawn(move || {
        println!("[SemanticIndex] Thread started. Waiting for file scan...");

        // Mark as indexing immediately
        {
            let si = si_clone.lock().unwrap();
            si.is_indexing.store(true, Ordering::SeqCst);
            // Attempt to load existing cache
            if let Err(e) = si.load() {
                eprintln!("[SemanticIndex] Cache load failed: {}", e);
            }
        }

        // Wait for project index to have some files (max 10 seconds)
        let start_wait = std::time::Instant::now();
        while pi_clone.get_files().is_empty() && start_wait.elapsed().as_secs() < 10 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let files = pi_clone.get_files();
        {
            let si = si_clone.lock().unwrap();
            si.files_total.store(files.len(), Ordering::SeqCst);
        }
        ctx_clone.request_repaint();

        // 1. Initialize ML model
        let mut temp_si = super::semantic_index::SemanticIndex::new(thread_root.clone());
        if let Err(e) = temp_si.init() {
            let err_msg = format!("Failed to initialize semantic index: {}", e);
            eprintln!("[SemanticIndex] {}", err_msg);
            let si = si_clone.lock().unwrap();
            *si.error.lock().unwrap() = Some(err_msg);
            si.is_indexing.store(false, Ordering::SeqCst);
            ctx_clone.request_repaint();
            return;
        }

        let model = temp_si.model.as_ref().unwrap();
        let tokenizer = temp_si.tokenizer.as_ref().unwrap();

        // 2. Index files (Smart incremental indexing)
        for (idx, rel_path) in files.iter().enumerate() {
            let path_str = rel_path.to_string_lossy();

            {
                let si = si_clone.lock().unwrap();
                si.files_processed.store(idx + 1, Ordering::SeqCst);
                *si.current_file.lock().unwrap() = path_str.to_string();
            }
            ctx_clone.request_repaint();

            // --- DYNAMIC FILTRATION ---
            if path_str.starts_with('.') || path_str.contains("/.") {
                continue;
            }

            let mut is_blacklisted = false;
            for pattern in &blacklist_patterns {
                if path_str.contains(pattern.trim_matches('*')) {
                    is_blacklisted = true;
                    break;
                }
            }
            if is_blacklisted {
                continue;
            }

            let ext = rel_path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let is_code = matches!(
                ext,
                "rs" | "toml" | "js" | "ts" | "py" | "c" | "cpp" | "h" | "sh" | "ftl"
            );
            if !is_code {
                continue;
            }

            let abs_path = thread_root.join(rel_path);
            let mtime = std::fs::metadata(&abs_path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            // Check if we need to re-index this file
            let needs_indexing = {
                let si = si_clone.lock().unwrap();
                let snippets = si.snippets.lock().unwrap();
                !snippets
                    .iter()
                    .any(|s| s.mtime == mtime && &s.path == rel_path)
            };

            if needs_indexing && let Ok(content) = std::fs::read_to_string(&abs_path) {
                if content.len() > 100_000 {
                    continue;
                }

                if content.as_bytes().contains(&0) {
                    continue;
                }

                {
                    let si = si_clone.lock().unwrap();
                    let mut snippets = si.snippets.lock().unwrap();
                    snippets.retain(|s| &s.path != rel_path);
                }

                let lines: Vec<&str> = content.lines().collect();
                let chunk_size = 30;
                let overlap = 5;
                let mut start = 0;

                while start < lines.len() {
                    let end = (start + chunk_size).min(lines.len());
                    let chunk_text = lines[start..end].join("\n");

                    if !chunk_text.trim().is_empty()
                        && let Ok(embedding) =
                            temp_si.vectorize_with_model(&chunk_text, model, tokenizer)
                    {
                        let si = si_clone.lock().unwrap();
                        si.snippets.lock().unwrap().push(
                            crate::app::ui::workspace::semantic_index::SemanticSnippet {
                                path: rel_path.clone(),
                                line_start: start + 1,
                                content: chunk_text,
                                embedding,
                                mtime,
                            },
                        );
                    }
                    if end == lines.len() {
                        break;
                    }
                    start += chunk_size - overlap;
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        }

        // 3. Finalize and Save
        {
            let mut si = si_clone.lock().unwrap();
            si.model = temp_si.model;
            si.tokenizer = temp_si.tokenizer;
            si.is_indexing.store(false, Ordering::SeqCst);
            if let Err(e) = si.save() {
                eprintln!("[SemanticIndex] Save failed: {}", e);
            }
        }
        ctx_clone.request_repaint();
        println!("[SemanticIndex] Indexing complete.");
    });

    let i18n = crate::i18n::I18n::new(&settings.lang);
    let profiles = load_profiles(&root_path);

    WorkspaceState {
        file_tree,
        editor: Editor::new(),
        watcher: FileWatcher::new(),
        project_watcher,
        claude_tabs: Vec::new(),
        claude_active_tab: 0,
        next_claude_tab_id: 1,
        next_terminal_id: 2,
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
        show_semantic_indexing_modal: true,
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
        wizard: crate::app::ui::dialogs::WizardState::default(),
        toasts: Vec::new(),
        folder_pick_rx: None,
        command_palette: None,
        project_index,
        semantic_index,
        file_picker: None,
        project_search: ProjectSearch::default(),
        lsp_client: None,
        lsp_binary_missing: false,
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
            crate::app::ui::widgets::ai_cli::StandardAI::get_logo(
                crate::config::CLI_VERSION,
                "gemini-1.5-flash",
                crate::config::CLI_TIER,
            ),
        )],
        gemini_system_prompt: settings
            .plugins
            .get("gemini")
            .and_then(|s| s.config.get("SYSTEM_PROMPT").cloned())
            .unwrap_or_else(|| i18n.get("gemini-default-prompt")),
        gemini_language: settings
            .plugins
            .get("gemini")
            .and_then(|s| s.config.get("LANGUAGE").cloned())
            .unwrap_or_else(|| i18n.lang().to_string()),
        gemini_total_tokens: 0,
        gemini_inspector_open: false,
        gemini_focus_requested: true,
        gemini_last_payload: String::new(),
        gemini_response: None,
        gemini_loading: false,
        markdown_cache: egui_commonmark::CommonMarkCache::default(),
        sync_confirmation: None,
        pending_agent_id: None,
        build_in_sandbox: settings.project_read_only,
        file_tree_in_sandbox,
        git_cancel,
        local_history: crate::app::local_history::LocalHistory::new(&root_path),
        sandbox,
        sandbox_staged_files,
        sandbox_staged_rx: None,
        sandbox_staged_dirty: false,
        sandbox_staged_last_dirty: std::time::Instant::now(),
        sandbox_staged_last_refresh: std::time::Instant::now(),
        background_io_rx: None,
        applied_settings_version: 0,
    }
}
