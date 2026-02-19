use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Výsledek asynchronního výběru složky.
/// bool = true → otevřít v novém okně; false → nahradit aktuální workspace.
type FolderPickResult = (Option<PathBuf>, bool);

use eframe::egui;

use super::build_runner::{BuildError, run_build_check};
use super::dialogs::{WizardState, show_project_wizard};
use super::modules::editor::Editor;
use super::modules::file_tree::FileTree;
use super::modules::terminal::Terminal;
use super::types::{
    AiTool, AppAction, AppShared, FocusedPanel, PersistentState, Toast, default_wizard_path,
};
use crate::config;
use crate::watcher::{FileWatcher, ProjectWatcher};

mod ai_panel;
mod background;
mod panels;
mod search_picker;

use ai_panel::render_ai_panel;
use background::{fetch_git_branch, fetch_git_status, process_background_events};
use panels::{render_left_panel, render_toasts};
use search_picker::{collect_project_files, render_file_picker, render_project_search_dialog};

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
    fn new(files: Vec<PathBuf>) -> Self {
        let filtered: Vec<usize> = (0..files.len()).collect();
        Self {
            query: String::new(),
            files,
            filtered,
            selected: 0,
            focus_requested: true,
        }
    }

    fn update_filter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = self
            .files
            .iter()
            .enumerate()
            .filter(|(_, p)| search_picker::fuzzy_match(&q, &p.to_string_lossy()))
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

fn spawn_file_index_scan(root: PathBuf) -> mpsc::Receiver<Vec<PathBuf>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(collect_project_files(&root));
    });
    rx
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

fn spawn_ai_tool_check() -> mpsc::Receiver<HashMap<AiTool, bool>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut status = HashMap::new();
        for tool in AiTool::ALL {
            status.insert(tool, is_command_available(tool.command()));
        }
        let _ = tx.send(status);
    });
    rx
}

pub(crate) fn init_workspace(root_path: PathBuf, panel_state: &PersistentState) -> WorkspaceState {
    let mut file_tree = FileTree::new();
    file_tree.load(&root_path);
    let project_watcher = ProjectWatcher::new(&root_path);
    let git_branch_rx = fetch_git_branch(&root_path);
    let git_status_rx = fetch_git_status(&root_path);
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
    }
}

pub(crate) fn open_file_in_ws(ws: &mut WorkspaceState, path: PathBuf) {
    ws.editor.open_file(&path);
    if let Some(parent) = path.parent() {
        ws.watcher.watch(parent);
    }
}

// ---------------------------------------------------------------------------
// Pomocné datové typy pro render_workspace
// ---------------------------------------------------------------------------

/// Akce vzniklé z menu baru — zpracovávají se po vykreslení menu.
#[derive(Default)]
struct MenuActions {
    open_folder: bool,
    save: bool,
    close_file: bool,
    quit: bool,
    new_project: bool,
    open_project: bool,
    open_recent: Option<PathBuf>,
    toggle_left: bool,
    toggle_right: bool,
    toggle_build: bool,
    toggle_float: bool,
    about: bool,
    settings: bool,
    build: bool,
    run: bool,
    open_file_picker: bool,
    project_search: bool,
}

// ---------------------------------------------------------------------------
// Pomocné funkce pro render_workspace
// ---------------------------------------------------------------------------

/// Vykreslí menu bar a vrátí zaznamenané akce.
fn render_menu_bar(
    ctx: &egui::Context,
    ws: &WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) -> MenuActions {
    let mut actions = MenuActions::default();
    let recent_snapshot = shared.lock().unwrap().recent_projects.clone();

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Soubor", |ui| {
                if ui.button("Otevřít složku…").clicked() {
                    actions.open_folder = true;
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Uložit").shortcut_text("Ctrl+S"))
                    .clicked()
                {
                    actions.save = true;
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Zavřít soubor").shortcut_text("Ctrl+W"))
                    .clicked()
                {
                    actions.close_file = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Ukončit").clicked() {
                    actions.quit = true;
                    ui.close_menu();
                }
            });

            ui.menu_button("Projekt", |ui| {
                if ui.button("Otevřít projekt…").clicked() {
                    actions.open_project = true;
                    ui.close_menu();
                }
                if ui.button("Nový projekt…").clicked() {
                    actions.new_project = true;
                    ui.close_menu();
                }
                if !recent_snapshot.is_empty() {
                    ui.separator();
                    ui.menu_button("Nedávné projekty", |ui| {
                        for path in &recent_snapshot {
                            let name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| path.to_string_lossy().into_owned());
                            if ui
                                .button(&name)
                                .on_hover_text(path.to_string_lossy())
                                .clicked()
                            {
                                actions.open_recent = Some(path.clone());
                                ui.close_menu();
                            }
                        }
                    });
                }
            });

            ui.menu_button("Upravit", |ui| {
                ui.add_enabled(
                    false,
                    egui::Button::new("Kopírovat").shortcut_text("Ctrl+C"),
                );
                ui.add_enabled(false, egui::Button::new("Vložit").shortcut_text("Ctrl+V"));
                ui.add_enabled(
                    false,
                    egui::Button::new("Vybrat vše").shortcut_text("Ctrl+A"),
                );
                ui.separator();
                if ui
                    .add(egui::Button::new("Hledat…").shortcut_text("Ctrl+F"))
                    .clicked()
                {
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Hledat a nahradit…").shortcut_text("Ctrl+H"))
                    .clicked()
                {
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Přejít na řádek…").shortcut_text("Ctrl+G"))
                    .clicked()
                {
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Otevřít soubor…").shortcut_text("Ctrl+P"))
                    .clicked()
                {
                    actions.open_file_picker = true;
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Hledat v projektu…").shortcut_text("Ctrl+Shift+F"))
                    .clicked()
                {
                    actions.project_search = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .add(egui::Button::new("Build").shortcut_text("Ctrl+B"))
                    .clicked()
                {
                    actions.build = true;
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Run").shortcut_text("Ctrl+R"))
                    .clicked()
                {
                    actions.run = true;
                    ui.close_menu();
                }
            });

            ui.menu_button("Zobrazit", |ui| {
                let left_label = if ws.show_left_panel {
                    "✓ Soubory"
                } else {
                    "  Soubory"
                };
                if ui.button(left_label).clicked() {
                    actions.toggle_left = true;
                    ui.close_menu();
                }
                let build_label = if ws.show_build_terminal {
                    "✓ Build terminál"
                } else {
                    "  Build terminál"
                };
                if ui.button(build_label).clicked() {
                    actions.toggle_build = true;
                    ui.close_menu();
                }
                let right_label = if ws.show_right_panel {
                    "✓ AI terminál"
                } else {
                    "  AI terminál"
                };
                if ui.button(right_label).clicked() {
                    actions.toggle_right = true;
                    ui.close_menu();
                }
                let float_label = if ws.claude_float {
                    "✓ Plovoucí AI terminál"
                } else {
                    "  Plovoucí AI terminál"
                };
                if ui.button(float_label).clicked() {
                    actions.toggle_float = true;
                    ui.close_menu();
                }
            });

            ui.menu_button("Nápověda", |ui| {
                if ui.button("Nastavení…").clicked() {
                    actions.settings = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("O aplikaci").clicked() {
                    actions.about = true;
                    ui.close_menu();
                }
            });
        });
    });

    actions
}

/// Aplikuje menu akce na stav workspace. Vrací cestu pro reinicializaci (pokud byla vybrána složka).
fn process_menu_actions(
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    actions: MenuActions,
) -> Option<PathBuf> {
    if actions.quit {
        shared.lock().unwrap().actions.push(AppAction::QuitAll);
    }
    if actions.save {
        ws.editor.save();
    }
    if actions.close_file {
        ws.editor.clear();
    }
    if actions.toggle_left {
        ws.show_left_panel = !ws.show_left_panel;
    }
    if actions.toggle_right {
        ws.show_right_panel = !ws.show_right_panel;
    }
    if actions.toggle_float {
        ws.claude_float = !ws.claude_float;
    }
    if actions.toggle_build {
        ws.show_build_terminal = !ws.show_build_terminal;
    }
    if actions.about {
        ws.show_about = true;
    }
    if actions.settings {
        ws.show_settings = true;
    }
    if actions.new_project {
        ws.show_new_project = true;
    }
    if actions.build {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo build 2>&1");
        }
        ws.build_error_rx = Some(run_build_check(ws.root_path.clone()));
        ws.build_errors.clear();
    }
    if actions.run {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo run 2>&1");
        }
    }
    if actions.open_file_picker && ws.file_picker.is_none() {
        if ws.file_index_rx.is_none() {
            ws.file_index_rx = Some(spawn_file_index_scan(ws.root_path.clone()));
        }
        let files = ws.file_index_cache.clone();
        ws.file_picker = Some(FilePicker::new(files));
    }
    if actions.project_search {
        ws.project_search.show_input = true;
        ws.project_search.focus_requested = true;
    }

    if let Some(path) = actions.open_recent {
        if path.is_dir() {
            let mut sh = shared.lock().unwrap();
            sh.actions.push(AppAction::AddRecent(path.clone()));
            sh.actions.push(AppAction::OpenInNewWindow(path));
        }
    }

    // Výsledek předchozího async file dialogu
    let mut open_here_path: Option<PathBuf> = None;
    if let Some(rx) = &ws.folder_pick_rx {
        if let Ok((maybe_path, in_new_window)) = rx.try_recv() {
            ws.folder_pick_rx = None;
            if let Some(dir) = maybe_path {
                let path = dir.canonicalize().unwrap_or(dir);
                if in_new_window {
                    let mut sh = shared.lock().unwrap();
                    sh.actions.push(AppAction::AddRecent(path.clone()));
                    sh.actions.push(AppAction::OpenInNewWindow(path));
                } else {
                    open_here_path = Some(path);
                }
            }
        }
    }

    // Spuštění async file dialogu (neblokuje UI vlákno)
    if actions.open_project && ws.folder_pick_rx.is_none() {
        let projects_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join("MyProject");
        if let Err(e) = std::fs::create_dir_all(&projects_dir) {
            ws.toasts.push(Toast::error(format!(
                "Nelze připravit adresář projektů: {e}"
            )));
        }
        let (tx, rx) = mpsc::channel();
        ws.folder_pick_rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send((
                rfd::FileDialog::new()
                    .set_directory(&projects_dir)
                    .pick_folder(),
                true,
            ));
        });
    }
    if actions.open_folder && ws.folder_pick_rx.is_none() {
        let start_dir = ws.root_path.clone();
        let (tx, rx) = mpsc::channel();
        ws.folder_pick_rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send((
                rfd::FileDialog::new()
                    .set_directory(&start_dir)
                    .pick_folder(),
                false,
            ));
        });
    }

    open_here_path
}

/// Vykreslí modální dialogy (O aplikaci, Nastavení, Nový projekt).
fn render_dialogs(ctx: &egui::Context, ws: &mut WorkspaceState, shared: &Arc<Mutex<AppShared>>) {
    if ws.show_about {
        let modal = egui::Modal::new(egui::Id::new("about_modal"));
        modal.show(ctx, |ui| {
            ui.heading("Rust Editor");
            ui.add_space(8.0);
            ui.label(format!("Verze: {}", env!("BUILD_VERSION")));
            ui.add_space(8.0);
            ui.label("Jednoduchý textový editor napsaný v Rustu s eframe/egui.");
            ui.add_space(12.0);
            if ui.button("Zavřít").clicked() {
                ws.show_about = false;
            }
        });
    }

    if ws.show_settings {
        // Inicializovat draft pouze jednou (při prvním otevření dialogu)
        if ws.settings_draft.is_none() {
            ws.settings_draft = Some(shared.lock().unwrap().settings.clone());
        }
        if let Some(rx) = ws.settings_folder_pick_rx.as_ref() {
            if let Ok(picked) = rx.try_recv() {
                ws.settings_folder_pick_rx = None;
                if let Some(dir) = picked {
                    if let Some(draft) = ws.settings_draft.as_mut() {
                        draft.default_project_path = dir.to_string_lossy().to_string();
                    }
                }
            }
        }

        let mut do_save = false;
        let mut do_close = false;
        let mut request_settings_browse = false;
        let mut browse_start_dir: Option<String> = None;

        let modal = egui::Modal::new(egui::Id::new("settings_modal"));
        modal.show(ctx, |ui| {
            ui.heading("Nastavení");
            ui.add_space(10.0);

            let draft = ws.settings_draft.as_mut().unwrap();

            // Téma
            ui.strong("Téma");
            ui.horizontal(|ui| {
                ui.radio_value(&mut draft.dark_theme, true, "Tmavé");
                ui.radio_value(&mut draft.dark_theme, false, "Světlé");
            });
            ui.add_space(10.0);

            // Font editoru
            ui.strong("Editor — velikost fontu");
            ui.add_space(4.0);
            ui.add(
                egui::Slider::new(&mut draft.editor_font_size, 10.0..=24.0)
                    .step_by(1.0)
                    .suffix(" px")
                    .clamping(egui::SliderClamping::Always),
            );
            ui.add_space(10.0);

            // AI terminál font scale (per-workspace, mimo global settings)
            ui.strong("AI terminál — velikost fontu");
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for &scale in &[100u32, 125, 150, 200] {
                    ui.radio_value(&mut ws.ai_font_scale, scale, format!("{}%", scale));
                }
            });
            ui.add_space(10.0);

            // Výchozí cesta projektů
            ui.strong("Výchozí cesta projektů");
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut draft.default_project_path)
                        .desired_width(280.0),
                );
                if ui.button("…").clicked() {
                    request_settings_browse = true;
                    browse_start_dir = Some(draft.default_project_path.clone());
                }
            });
            ui.add_space(14.0);

            ui.horizontal(|ui| {
                if ui.button("Uložit").clicked() {
                    do_save = true;
                }
                if ui.button("Zavřít").clicked() {
                    do_close = true;
                }
            });
        });

        if request_settings_browse && ws.settings_folder_pick_rx.is_none() {
            let start_dir = browse_start_dir.unwrap_or_default();
            let (tx, rx) = mpsc::channel();
            ws.settings_folder_pick_rx = Some(rx);
            std::thread::spawn(move || {
                let dialog = rfd::FileDialog::new();
                let picked = if start_dir.trim().is_empty() {
                    dialog.pick_folder()
                } else {
                    dialog.set_directory(start_dir).pick_folder()
                };
                let _ = tx.send(picked);
            });
        }

        if do_save {
            let draft = ws.settings_draft.take().unwrap();
            draft.save();
            ws.wizard.path = draft.default_project_path.clone();
            shared.lock().unwrap().settings = draft;
            ws.show_settings = false;
            ws.settings_folder_pick_rx = None;
        } else if do_close {
            ws.settings_draft = None;
            ws.show_settings = false;
            ws.settings_folder_pick_rx = None;
        }
    }

    if ws.show_new_project {
        show_project_wizard(
            ctx,
            &mut ws.wizard,
            &mut ws.show_new_project,
            "ws_new_project_modal",
            shared,
            |path, sh| {
                let mut sh = sh.lock().unwrap();
                sh.actions.push(AppAction::AddRecent(path.clone()));
                sh.actions.push(AppAction::OpenInNewWindow(path));
            },
        );
    }
}

// ---------------------------------------------------------------------------
// render_workspace — orchestrátor vykreslení jednoho pracovního prostoru
// Vrací Some(path) pokud má být workspace reinicializován s novou cestou.
// ---------------------------------------------------------------------------

pub(crate) fn render_workspace(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) -> Option<PathBuf> {
    // Lazy init terminálů
    if ws.claude_tabs.is_empty() {
        let root = ws.root_path.clone();
        let id = ws.next_claude_tab_id;
        ws.next_claude_tab_id += 1;
        ws.claude_tabs.push(Terminal::new(id, ctx, &root, None));
    }
    if ws.build_terminal.is_none() {
        ws.build_terminal = Some(Terminal::new(1, ctx, &ws.root_path, None));
    }

    // Události na pozadí (watcher, build, autosave)
    process_background_events(ws);

    // Pravidelné překreslování pro autosave a watcher
    ctx.request_repaint_after(std::time::Duration::from_millis(
        config::REPAINT_INTERVAL_MS,
    ));

    // Klávesové zkratky
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
        ws.editor.save();
        // Po uložení okamžitě aktualizujeme git status
        if ws.git_status_rx.is_none() {
            ws.git_status_rx = Some(fetch_git_status(&ws.root_path));
        }
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::W)) {
        ws.editor.clear();
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::B)) {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo build 2>&1");
        }
        ws.build_error_rx = Some(run_build_check(ws.root_path.clone()));
        ws.build_errors.clear();
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R)) {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo run 2>&1");
        }
    }
    // Ctrl+P — fuzzy file picker
    if ctx.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::P)) {
        if ws.file_picker.is_none() {
            if ws.file_index_rx.is_none() {
                ws.file_index_rx = Some(spawn_file_index_scan(ws.root_path.clone()));
            }
            let files = ws.file_index_cache.clone();
            ws.file_picker = Some(FilePicker::new(files));
        } else {
            ws.file_picker = None;
        }
    }
    // Ctrl+Shift+F — hledání napříč projektem
    if ctx.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::F)) {
        ws.project_search.show_input = true;
        ws.project_search.focus_requested = true;
    }

    // Menu bar + zpracování akcí
    let actions = render_menu_bar(ctx, ws, shared);
    let open_here_path = process_menu_actions(ws, shared, actions);

    // Modální dialogy
    render_dialogs(ctx, ws, shared);

    // File picker (Ctrl+P)
    if let Some(path) = render_file_picker(ctx, ws) {
        open_file_in_ws(ws, path);
    }

    // Hledání napříč projektem
    render_project_search_dialog(ctx, ws);

    // Status bar (musí být před SidePanel)
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(config::STATUS_BAR_HEIGHT)
        .show(ctx, |ui| {
            ws.editor.status_bar(ui, ws.git_branch.as_deref());
        });

    let dialog_open = ws.file_tree.has_open_dialog();

    // Panely (pořadí: pravý, levý, centrální)
    let ai_clicked = render_ai_panel(ctx, ws, dialog_open);
    let left_clicked = render_left_panel(ctx, ws, dialog_open);

    egui::CentralPanel::default().show(ctx, |ui| {
        if ws.editor.ui(ui, dialog_open) {
            ws.focused_panel = FocusedPanel::Editor;
        }
    });

    // Focus follows mouse — vrátit fokus editoru pokud terminál nebyl aktivně kliknut
    if !ai_clicked && !left_clicked {
        let in_terminal =
            ws.focused_panel == FocusedPanel::Claude || ws.focused_panel == FocusedPanel::Build;
        if in_terminal {
            ws.focused_panel = FocusedPanel::Editor;
            ws.editor.request_editor_focus();
        }
    }

    // Toast notifikace
    render_toasts(ctx, ws);

    open_here_path
}
