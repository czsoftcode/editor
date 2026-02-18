use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

use crate::ipc::{self, Ipc, IpcServer};

use eframe::egui;

use crate::editor::Editor;
use crate::file_tree::FileTree;
use crate::terminal::Terminal;
use crate::watcher::{FileWatcher, FsChange, ProjectWatcher};

// ---------------------------------------------------------------------------
// Pomocné typy
// ---------------------------------------------------------------------------

#[derive(PartialEq, Clone, Copy)]
enum FocusedPanel {
    Build,
    Claude,
    Editor,
}

#[derive(PartialEq, Clone, Copy)]
enum ProjectType {
    Rust,
    Symfony,
}

impl ProjectType {
    fn subdir(&self) -> &'static str {
        match self {
            ProjectType::Rust => "Rust",
            ProjectType::Symfony => "Symfony",
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum AiTool {
    ClaudeCode,
    Codex,
}

impl AiTool {
    fn command(&self) -> &'static str {
        match self {
            AiTool::ClaudeCode => "claude",
            AiTool::Codex => "codex",
        }
    }
}

const STORAGE_KEY: &str = "panel_state";

fn default_font_scale() -> u32 {
    100
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct PersistentState {
    show_left_panel: bool,
    show_right_panel: bool,
    show_build_terminal: bool,
    claude_float: bool,
    #[serde(default = "default_font_scale")]
    ai_font_scale: u32,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            show_left_panel: true,
            show_right_panel: true,
            show_build_terminal: true,
            claude_float: false,
            ai_font_scale: 100,
        }
    }
}

pub struct BuildError {
    pub file: PathBuf,
    pub line: usize,
    pub _column: usize,
    pub message: String,
    pub is_warning: bool,
}

// ---------------------------------------------------------------------------
// AppShared — sdílený stav mezi viewporty (chráněný Mutexem)
// ---------------------------------------------------------------------------

enum AppAction {
    /// Otevřít projekt v novém okně
    OpenInNewWindow(PathBuf),
    /// Zavřít sekundární viewport (po zavření × okna)
    CloseWorkspace(egui::ViewportId),
    /// Přidat cestu do nedávných projektů
    AddRecent(PathBuf),
    /// Ukončit celou aplikaci
    QuitAll,
}

struct AppShared {
    recent_projects: Vec<PathBuf>,
    actions: Vec<AppAction>,
}

// ---------------------------------------------------------------------------
// WorkspaceState — stav jednoho pracovního prostoru (okna projektu)
// ---------------------------------------------------------------------------

struct WorkspaceState {
    file_tree: FileTree,
    editor: Editor,
    watcher: FileWatcher,
    project_watcher: ProjectWatcher,
    claude_terminal: Option<Terminal>,
    build_terminal: Option<Terminal>,
    focused_panel: FocusedPanel,
    root_path: PathBuf,
    show_left_panel: bool,
    show_right_panel: bool,
    show_build_terminal: bool,
    show_about: bool,
    show_settings: bool,
    ai_font_scale: u32,
    build_errors: Vec<BuildError>,
    build_error_rx: Option<mpsc::Receiver<Vec<BuildError>>>,
    claude_tool: AiTool,
    claude_float: bool,
    // Wizard nového projektu (pro toto okno)
    show_new_project: bool,
    wizard_type: ProjectType,
    wizard_name: String,
    wizard_path: String,
    wizard_error: String,
}

// ---------------------------------------------------------------------------
// SecondaryWorkspace — sekundární viewport (jeden projekt v novém okně)
// ---------------------------------------------------------------------------

struct SecondaryWorkspace {
    viewport_id: egui::ViewportId,
    state: Arc<Mutex<WorkspaceState>>,
}

// ---------------------------------------------------------------------------
// EditorApp — hlavní aplikace (kořenový viewport)
// ---------------------------------------------------------------------------

pub struct EditorApp {
    /// Kořenový workspace (None = startup dialog)
    root_ws: Option<WorkspaceState>,
    /// Sekundární workspacy (za Arc<Mutex> pro deferred viewporty)
    secondary: Vec<SecondaryWorkspace>,
    /// Sdílený stav — komunikace mezi viewporty
    shared: Arc<Mutex<AppShared>>,
    /// Čítač pro jedinečné ViewportId
    next_viewport_counter: u64,

    /// Stav uložený pro obnovení při zavření/otevření workspace
    saved_panel_state: PersistentState,

    // --- Startup dialog ---
    path_buffer: String,

    // --- Wizard nového projektu (startup screen) ---
    show_startup_wizard: bool,
    startup_wizard_type: ProjectType,
    startup_wizard_name: String,
    startup_wizard_path: String,
    startup_wizard_error: String,

    // --- Ukončení aplikace ---
    show_quit_confirm: bool,
    quit_confirmed: bool,

    _ipc_server: Option<IpcServer>,
    focus_rx: mpsc::Receiver<()>,
}

// ---------------------------------------------------------------------------
// Pomocné free funkce
// ---------------------------------------------------------------------------

fn path_env() -> String {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let current = std::env::var("PATH").unwrap_or_default();
    let cargo_bin = home.join(".cargo/bin");
    let local_bin = home.join(".local/bin");
    format!("{}:{}:{}", cargo_bin.display(), local_bin.display(), current)
}

fn default_wizard_path() -> String {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join("MyProject")
        .to_string_lossy()
        .to_string()
}

fn ws_to_panel_state(ws: &WorkspaceState) -> PersistentState {
    PersistentState {
        show_left_panel: ws.show_left_panel,
        show_right_panel: ws.show_right_panel,
        show_build_terminal: ws.show_build_terminal,
        claude_float: ws.claude_float,
        ai_font_scale: ws.ai_font_scale,
    }
}

fn init_workspace(root_path: PathBuf, panel_state: &PersistentState) -> WorkspaceState {
    let mut file_tree = FileTree::new();
    file_tree.load(&root_path);
    let project_watcher = ProjectWatcher::new(&root_path);

    WorkspaceState {
        file_tree,
        editor: Editor::new(),
        watcher: FileWatcher::new(),
        project_watcher,
        claude_terminal: None,
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
        wizard_type: ProjectType::Rust,
        wizard_name: String::new(),
        wizard_path: default_wizard_path(),
        wizard_error: String::new(),
    }
}

fn open_file_in_ws(ws: &mut WorkspaceState, path: PathBuf) {
    ws.editor.open_file(&path);
    if let Some(parent) = path.parent() {
        ws.watcher.watch(parent);
    }
}

fn run_build_check(root_path: PathBuf) -> mpsc::Receiver<Vec<BuildError>> {
    let (tx, rx) = mpsc::channel();
    let env = path_env();
    std::thread::spawn(move || {
        let output = std::process::Command::new("cargo")
            .args(["build", "--color=never"])
            .current_dir(&root_path)
            .env("PATH", &env)
            .output();
        if let Ok(output) = output {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let _ = tx.send(parse_build_errors(&stderr));
        }
    });
    rx
}

fn parse_build_errors(stderr: &str) -> Vec<BuildError> {
    let mut errors = Vec::new();
    let mut current_message: Option<(String, bool)> = None;

    for line in stderr.lines() {
        if line.starts_with("error") || line.starts_with("warning") {
            let is_warning = line.starts_with("warning");
            current_message = Some((line.to_string(), is_warning));
        } else if let Some(location) = line.trim_start().strip_prefix("--> ") {
            if let Some((msg, is_warning)) = current_message.take() {
                let parts: Vec<&str> = location.rsplitn(3, ':').collect();
                if parts.len() >= 3 {
                    if let (Ok(line_num), Ok(col)) =
                        (parts[1].parse::<usize>(), parts[0].parse::<usize>())
                    {
                        errors.push(BuildError {
                            file: PathBuf::from(parts[2]),
                            line: line_num,
                            _column: col,
                            message: msg,
                            is_warning,
                        });
                    }
                }
            }
        }
    }
    errors
}

// ---------------------------------------------------------------------------
// render_workspace — vykreslí obsah jednoho pracovního prostoru
// Vrací Some(path) pokud má být workspace reinicializován s novou cestou.
// ---------------------------------------------------------------------------

fn render_workspace(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) -> Option<PathBuf> {
    // Lazy init terminálů
    if ws.claude_terminal.is_none() {
        ws.claude_terminal = Some(Terminal::new(0, ctx, &ws.root_path, Some(ws.claude_tool.command())));
    }
    if ws.build_terminal.is_none() {
        ws.build_terminal = Some(Terminal::new(1, ctx, &ws.root_path, None));
    }

    // Změny souborů — watcher
    if let Some(changed_path) = ws.watcher.try_recv() {
        if let Some(editor_path) = ws.editor.active_path() {
            if let (Ok(a), Ok(b)) = (changed_path.canonicalize(), editor_path.canonicalize()) {
                if a == b && !ws.editor.is_modified() {
                    ws.editor.reload_from_disk();
                }
            }
        }
    }

    // Změny projektu — project watcher
    let fs_changes = ws.project_watcher.poll();
    if !fs_changes.is_empty() {
        let mut need_reload = false;
        let mut open_file: Option<PathBuf> = None;
        for change in &fs_changes {
            match change {
                FsChange::Created(path) => {
                    need_reload = true;
                    if path.is_file() {
                        open_file = Some(path.clone());
                    }
                }
                FsChange::Removed(path) => {
                    need_reload = true;
                    ws.editor.close_tabs_for_path(path);
                }
                FsChange::Modified => {
                    need_reload = true;
                }
            }
        }
        if need_reload {
            if let Some(ref path) = open_file {
                ws.file_tree.request_reload_and_expand(path);
            } else {
                ws.file_tree.request_reload();
            }
        }
        if let Some(path) = open_file {
            open_file_in_ws(ws, path);
        }
    }

    // Build výsledky
    if let Some(rx) = &ws.build_error_rx {
        if let Ok(errors) = rx.try_recv() {
            ws.build_errors = errors;
            ws.build_error_rx = None;
        }
    }

    // Autosave
    ws.editor.try_autosave();

    // Repaint pro autosave a watcher
    ctx.request_repaint_after(std::time::Duration::from_millis(250));

    // Klávesové zkratky
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
        ws.editor.save();
    }
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::W)) {
        ws.editor.clear();
    }

    // --- Menu bar ---
    let mut action_open_folder = false;
    let mut action_save = false;
    let mut action_close_file = false;
    let mut action_quit = false;
    let mut action_new_project = false;
    let mut action_open_project = false;
    let mut action_open_recent: Option<PathBuf> = None;
    let mut action_toggle_left = false;
    let mut action_toggle_right = false;
    let mut action_toggle_build = false;
    let mut action_toggle_float = false;
    let mut action_about = false;
    let mut action_settings = false;

    let recent_snapshot = shared.lock().unwrap().recent_projects.clone();

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Soubor", |ui| {
                if ui.button("Otevřít složku…").clicked() {
                    action_open_folder = true;
                    ui.close_menu();
                }
                if ui.add(egui::Button::new("Uložit").shortcut_text("Ctrl+S")).clicked() {
                    action_save = true;
                    ui.close_menu();
                }
                if ui.add(egui::Button::new("Zavřít soubor").shortcut_text("Ctrl+W")).clicked() {
                    action_close_file = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Ukončit").clicked() {
                    action_quit = true;
                    ui.close_menu();
                }
            });

            ui.menu_button("Projekt", |ui| {
                if ui.button("Otevřít projekt…").clicked() {
                    action_open_project = true;
                    ui.close_menu();
                }
                if ui.button("Nový projekt…").clicked() {
                    action_new_project = true;
                    ui.close_menu();
                }
                if !recent_snapshot.is_empty() {
                    ui.separator();
                    ui.menu_button("Nedávné projekty", |ui| {
                        for path in &recent_snapshot {
                            let name = path.file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| path.to_string_lossy().into_owned());
                            if ui.button(&name).on_hover_text(path.to_string_lossy()).clicked() {
                                action_open_recent = Some(path.clone());
                                ui.close_menu();
                            }
                        }
                    });
                }
            });

            ui.menu_button("Upravit", |ui| {
                ui.add_enabled(false, egui::Button::new("Kopírovat").shortcut_text("Ctrl+C"));
                ui.add_enabled(false, egui::Button::new("Vložit").shortcut_text("Ctrl+V"));
                ui.add_enabled(false, egui::Button::new("Vybrat vše").shortcut_text("Ctrl+A"));
                ui.separator();
                if ui.add(egui::Button::new("Hledat…").shortcut_text("Ctrl+F")).clicked() {
                    ui.close_menu();
                }
                if ui.add(egui::Button::new("Hledat a nahradit…").shortcut_text("Ctrl+H")).clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("Zobrazit", |ui| {
                let left_label = if ws.show_left_panel { "✓ Soubory" } else { "  Soubory" };
                if ui.button(left_label).clicked() {
                    action_toggle_left = true;
                    ui.close_menu();
                }
                let build_label = if ws.show_build_terminal { "✓ Build terminál" } else { "  Build terminál" };
                if ui.button(build_label).clicked() {
                    action_toggle_build = true;
                    ui.close_menu();
                }
                let right_label = if ws.show_right_panel { "✓ AI terminál" } else { "  AI terminál" };
                if ui.button(right_label).clicked() {
                    action_toggle_right = true;
                    ui.close_menu();
                }
                let float_label = if ws.claude_float { "✓ Plovoucí AI terminál" } else { "  Plovoucí AI terminál" };
                if ui.button(float_label).clicked() {
                    action_toggle_float = true;
                    ui.close_menu();
                }
            });

            ui.menu_button("Nápověda", |ui| {
                if ui.button("Nastavení…").clicked() {
                    action_settings = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("O aplikaci").clicked() {
                    action_about = true;
                    ui.close_menu();
                }
            });
        });
    });

    // Zpracování menu akcí
    if action_quit {
        shared.lock().unwrap().actions.push(AppAction::QuitAll);
    }
    if action_save { ws.editor.save(); }
    if action_close_file { ws.editor.clear(); }
    if action_toggle_left { ws.show_left_panel = !ws.show_left_panel; }
    if action_toggle_right { ws.show_right_panel = !ws.show_right_panel; }
    if action_toggle_float { ws.claude_float = !ws.claude_float; }
    if action_toggle_build { ws.show_build_terminal = !ws.show_build_terminal; }
    if action_about { ws.show_about = true; }
    if action_settings { ws.show_settings = true; }
    if action_new_project { ws.show_new_project = true; }

    if let Some(path) = action_open_recent {
        if path.is_dir() {
            let mut sh = shared.lock().unwrap();
            sh.actions.push(AppAction::AddRecent(path.clone()));
            sh.actions.push(AppAction::OpenInNewWindow(path));
        }
    }
    if action_open_project {
        let projects_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join("MyProject");
        let _ = std::fs::create_dir_all(&projects_dir);
        if let Some(dir) = rfd::FileDialog::new()
            .set_directory(&projects_dir)
            .pick_folder()
        {
            let path = dir.canonicalize().unwrap_or(dir);
            let mut sh = shared.lock().unwrap();
            sh.actions.push(AppAction::AddRecent(path.clone()));
            sh.actions.push(AppAction::OpenInNewWindow(path));
        }
    }
    // "Otevřít složku" = nahradit projekt v TOMTO okně
    let mut open_here_path: Option<PathBuf> = None;
    if action_open_folder {
        if let Some(dir) = rfd::FileDialog::new()
            .set_directory(&ws.root_path)
            .pick_folder()
        {
            open_here_path = Some(dir.canonicalize().unwrap_or(dir));
        }
    }

    // About dialog
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

    // Settings dialog
    if ws.show_settings {
        let modal = egui::Modal::new(egui::Id::new("settings_modal"));
        modal.show(ctx, |ui| {
            ui.heading("Nastavení");
            ui.add_space(12.0);
            ui.strong("AI terminál");
            ui.add_space(4.0);
            ui.label("Velikost fontu:");
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for &scale in &[100u32, 125, 150, 200] {
                    ui.radio_value(&mut ws.ai_font_scale, scale, format!("{}%", scale));
                }
            });
            ui.add_space(16.0);
            if ui.button("Zavřít").clicked() {
                ws.show_settings = false;
            }
        });
    }

    // Wizard nového projektu (pro toto okno)
    if ws.show_new_project {
        show_workspace_wizard(ctx, ws, shared);
    }

    // Status bar
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(22.0)
        .show(ctx, |ui| {
            ws.editor.status_bar(ui);
        });

    // Right panel — Claude terminál
    let dialog_open = ws.file_tree.has_open_dialog();
    let focused = ws.focused_panel;
    let mut any_terminal_clicked = false;

    if ws.show_right_panel {
        if ws.claude_float {
            let mut is_open = true;
            egui::Window::new("AI terminál")
                .id(egui::Id::new("claude_float_win"))
                .default_size([520.0, 420.0])
                .min_size([300.0, 200.0])
                .resizable(true)
                .collapsible(false)
                .open(&mut is_open)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let prev_tool = ws.claude_tool;
                        ui.radio_value(&mut ws.claude_tool, AiTool::ClaudeCode, "Claude Code");
                        ui.radio_value(&mut ws.claude_tool, AiTool::Codex, "Codex");
                        if ws.claude_tool != prev_tool {
                            let cmd = ws.claude_tool.command();
                            if let Some(terminal) = &mut ws.claude_terminal {
                                terminal.restart_with_command(ui.ctx(), Some(cmd));
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("⊟").on_hover_text("Přikovat do panelu").clicked() {
                                ws.claude_float = false;
                            }
                        });
                    });
                    ui.separator();
                    if !dialog_open {
                        let font_size = 14.0 * ws.ai_font_scale as f32 / 100.0;
                        if let Some(terminal) = &mut ws.claude_terminal {
                            if terminal.ui(ui, focused == FocusedPanel::Claude, font_size) {
                                ws.focused_panel = FocusedPanel::Claude;
                                any_terminal_clicked = true;
                            }
                        }
                    }
                });
            if !is_open {
                ws.show_right_panel = false;
            }
        } else {
            egui::SidePanel::right("claude_panel")
                .default_width(400.0)
                .width_range(200.0..=600.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("AI terminál");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("⧉").on_hover_text("Odpojit do plovoucího okna").clicked() {
                                ws.claude_float = true;
                            }
                        });
                    });
                    let prev_tool = ws.claude_tool;
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut ws.claude_tool, AiTool::ClaudeCode, "Claude Code");
                        ui.radio_value(&mut ws.claude_tool, AiTool::Codex, "Codex");
                    });
                    if ws.claude_tool != prev_tool {
                        let cmd = ws.claude_tool.command();
                        if let Some(terminal) = &mut ws.claude_terminal {
                            terminal.restart_with_command(ui.ctx(), Some(cmd));
                        }
                    }
                    ui.separator();
                    if !dialog_open {
                        let font_size = 14.0 * ws.ai_font_scale as f32 / 100.0;
                        if let Some(terminal) = &mut ws.claude_terminal {
                            if terminal.ui(ui, focused == FocusedPanel::Claude, font_size) {
                                ws.focused_panel = FocusedPanel::Claude;
                                any_terminal_clicked = true;
                            }
                        }
                    }
                });
        }
    }

    // Left panel — file tree + build terminál
    if ws.show_left_panel {
        egui::SidePanel::left("left_panel")
            .default_width(300.0)
            .width_range(200.0..=500.0)
            .resizable(true)
            .show(ctx, |ui| {
                let total_height = ui.available_height();
                let tree_height = if ws.show_build_terminal {
                    (total_height * 0.55).max(100.0)
                } else {
                    total_height
                };

                egui::Frame::NONE.show(ui, |ui| {
                    ui.set_max_height(tree_height);
                    ui.heading("Soubory");
                    ui.separator();
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let result = ws.file_tree.ui(ui);
                            if let Some(path) = result.selected {
                                open_file_in_ws(ws, path);
                            }
                            if let Some(path) = result.created_file {
                                open_file_in_ws(ws, path);
                            }
                            if let Some(deleted) = result.deleted {
                                ws.editor.close_tabs_for_path(&deleted);
                            }
                        });
                });

                if ws.show_build_terminal {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.strong("Build");
                        ui.separator();
                        if ui.small_button("\u{25B6} Build").clicked() {
                            if let Some(terminal) = &mut ws.build_terminal {
                                terminal.send_command("cargo build 2>&1");
                            }
                            let rx = run_build_check(ws.root_path.clone());
                            ws.build_error_rx = Some(rx);
                            ws.build_errors.clear();
                        }
                        if ui.small_button("\u{25B6} Run").clicked() {
                            if let Some(terminal) = &mut ws.build_terminal {
                                terminal.send_command("cargo run 2>&1");
                            }
                        }
                        if ui.small_button("\u{25B6} Test").clicked() {
                            if let Some(terminal) = &mut ws.build_terminal {
                                terminal.send_command("cargo test 2>&1");
                            }
                        }
                        if ui.small_button("\u{2716} Clean").clicked() {
                            if let Some(terminal) = &mut ws.build_terminal {
                                terminal.send_command("cargo clean");
                            }
                        }
                    });
                    ui.separator();

                    if !dialog_open {
                        if let Some(terminal) = &mut ws.build_terminal {
                            if terminal.ui(ui, focused == FocusedPanel::Build, 14.0) {
                                ws.focused_panel = FocusedPanel::Build;
                                any_terminal_clicked = true;
                            }
                        }
                    }

                    if !ws.build_errors.is_empty() {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Chyby ({})", ws.build_errors.len()))
                                .strong()
                                .size(12.0),
                        );
                        let mut open_error_file: Option<(PathBuf, usize)> = None;
                        egui::ScrollArea::vertical()
                            .id_salt("build_errors_scroll")
                            .max_height(150.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for error in &ws.build_errors {
                                    let color = if error.is_warning {
                                        egui::Color32::from_rgb(230, 180, 60)
                                    } else {
                                        egui::Color32::from_rgb(230, 80, 80)
                                    };
                                    let label_text = format!(
                                        "{}:{}  {}",
                                        error.file.display(),
                                        error.line,
                                        error.message,
                                    );
                                    let response = ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&label_text)
                                                .size(11.0)
                                                .color(color),
                                        )
                                        .sense(egui::Sense::click()),
                                    );
                                    if response.clicked() {
                                        let full_path = ws.root_path.join(&error.file);
                                        open_error_file = Some((full_path, error.line));
                                    }
                                }
                            });
                        if let Some((path, _line)) = open_error_file {
                            open_file_in_ws(ws, path);
                        }
                    }
                }
            });
    }

    // Central panel — editor
    egui::CentralPanel::default().show(ctx, |ui| {
        if ws.editor.ui(ui, dialog_open) {
            ws.focused_panel = FocusedPanel::Editor;
        }
    });

    // Focus follows mouse
    let in_terminal = ws.focused_panel == FocusedPanel::Claude
        || ws.focused_panel == FocusedPanel::Build;
    if !any_terminal_clicked && in_terminal {
        ws.focused_panel = FocusedPanel::Editor;
    }

    open_here_path
}


// ---------------------------------------------------------------------------
// show_workspace_wizard — wizard nového projektu v pracovním okně
// ---------------------------------------------------------------------------

fn show_workspace_wizard(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) {
    if !ws.show_new_project { return; }

    let modal = egui::Modal::new(egui::Id::new("ws_new_project_modal"));
    let mut close_dialog = false;
    let mut create_project: Option<(ProjectType, String, String)> = None;

    modal.show(ctx, |ui| {
        ui.heading("Nový projekt");
        ui.add_space(12.0);

        ui.label("Typ projektu:");
        ui.horizontal(|ui| {
            ui.radio_value(&mut ws.wizard_type, ProjectType::Rust, "Rust");
            ui.radio_value(&mut ws.wizard_type, ProjectType::Symfony, "Symfony");
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Název:");
            ui.add(egui::TextEdit::singleline(&mut ws.wizard_name).desired_width(250.0));
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Pracovní adresář:");
            ui.add(egui::TextEdit::singleline(&mut ws.wizard_path).desired_width(200.0));
            if ui.button("Procházet…").clicked() {
                if let Some(dir) = rfd::FileDialog::new()
                    .set_directory(&ws.wizard_path)
                    .pick_folder()
                {
                    ws.wizard_path = dir.to_string_lossy().to_string();
                }
            }
        });

        let raw_name = ws.wizard_name.trim();
        let display_name = if ws.wizard_type == ProjectType::Rust {
            raw_name.to_lowercase()
        } else {
            raw_name.to_string()
        };
        if !display_name.is_empty() {
            let preview = PathBuf::from(ws.wizard_path.trim())
                .join(ws.wizard_type.subdir())
                .join(&display_name);
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Vytvoří se v:");
                ui.monospace(preview.to_string_lossy().to_string());
            });
        }

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let can_create = !ws.wizard_name.trim().is_empty() && !ws.wizard_path.trim().is_empty();
            if ui.add_enabled(can_create, egui::Button::new("Vytvořit")).clicked() {
                create_project = Some((ws.wizard_type, ws.wizard_name.trim().to_string(), ws.wizard_path.trim().to_string()));
            }
            if ui.button("Zrušit").clicked() {
                close_dialog = true;
            }
        });

        if !ws.wizard_error.is_empty() {
            ui.add_space(4.0);
            ui.colored_label(egui::Color32::RED, &ws.wizard_error);
        }
    });

    if close_dialog {
        ws.show_new_project = false;
        return;
    }

    if let Some((project_type, raw_name, base_path)) = create_project {
        let name = if project_type == ProjectType::Rust { raw_name.to_lowercase() } else { raw_name };
        let type_dir = PathBuf::from(&base_path).join(project_type.subdir());
        if !type_dir.exists() {
            let _ = std::fs::create_dir_all(&type_dir);
        }
        let full_path = type_dir.join(&name);
        let env = path_env();

        let result = match project_type {
            ProjectType::Rust => std::process::Command::new("cargo")
                .args(["new", &name])
                .current_dir(&type_dir)
                .env("PATH", &env)
                .output(),
            ProjectType::Symfony => std::process::Command::new("composer")
                .args(["create-project", "symfony/skeleton", &name])
                .current_dir(&type_dir)
                .env("PATH", &env)
                .output(),
        };

        ws.wizard_error.clear();
        match result {
            Ok(output) if output.status.success() => {
                let path = full_path.canonicalize().unwrap_or(full_path);
                ws.show_new_project = false;
                ws.wizard_name.clear();
                // Otevřít v novém okně
                let mut sh = shared.lock().unwrap();
                sh.actions.push(AppAction::AddRecent(path.clone()));
                sh.actions.push(AppAction::OpenInNewWindow(path));
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                ws.wizard_error = if !stderr.is_empty() {
                    stderr.to_string()
                } else if !stdout.is_empty() {
                    stdout.to_string()
                } else {
                    format!("Příkaz selhal s kódem: {}", output.status)
                };
            }
            Err(e) => {
                ws.wizard_error = format!("Nepodařilo se spustit příkaz: {}", e);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// EditorApp — implementace
// ---------------------------------------------------------------------------

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext, root_path: Option<PathBuf>) -> Self {
        let panel_state: PersistentState = cc.storage
            .and_then(|s| eframe::get_value(s, STORAGE_KEY))
            .unwrap_or_default();

        let ipc_server = IpcServer::start();
        let focus_rx = ipc::start_process_listener();

        // Načíst nedávné projekty
        let recent_projects = Ipc::recent();

        // Určit seznam projektů k otevření
        let paths_to_open: Vec<PathBuf> = if let Some(p) = root_path {
            // CLI argument — otevřít jen tento projekt
            vec![p]
        } else {
            // Session restore
            ipc::load_session()
        };

        // Registrovat všechny projekty
        for p in &paths_to_open {
            Ipc::register(p);
        }

        // Přidat do nedávných
        for p in &paths_to_open {
            Ipc::add_recent(p);
        }

        // Inicializovat kořenový workspace
        let root_ws = paths_to_open.first().map(|p| {
            init_workspace(p.clone(), &panel_state)
        });

        // Inicializovat sekundární workspacy ze session
        let mut counter = 0u64;
        let secondary: Vec<SecondaryWorkspace> = paths_to_open.get(1..)
            .unwrap_or(&[])
            .iter()
            .map(|p| {
                let vid = egui::ViewportId::from_hash_of(format!("workspace_{}", counter));
                counter += 1;
                SecondaryWorkspace {
                    viewport_id: vid,
                    state: Arc::new(Mutex::new(init_workspace(p.clone(), &panel_state))),
                }
            })
            .collect();

        let shared = Arc::new(Mutex::new(AppShared {
            recent_projects,
            actions: Vec::new(),
        }));

        // Aktualizovat lokální cache nedávných
        {
            let mut s = shared.lock().unwrap();
            for p in &paths_to_open {
                s.recent_projects.retain(|rp| rp != p);
                s.recent_projects.insert(0, p.clone());
            }
            s.recent_projects.truncate(10);
        }

        let home = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .to_string_lossy()
            .to_string();

        let projects_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join("MyProject")
            .to_string_lossy()
            .to_string();

        Self {
            root_ws,
            secondary,
            shared,
            next_viewport_counter: counter,
            saved_panel_state: panel_state,
            path_buffer: home,
            show_startup_wizard: false,
            startup_wizard_type: ProjectType::Rust,
            startup_wizard_name: String::new(),
            startup_wizard_path: projects_dir,
            startup_wizard_error: String::new(),
            show_quit_confirm: false,
            quit_confirmed: false,
            _ipc_server: ipc_server,
            focus_rx,
        }
    }

    fn current_panel_state(&self) -> PersistentState {
        self.root_ws.as_ref()
            .map(ws_to_panel_state)
            .unwrap_or_else(|| self.saved_panel_state.clone())
    }

    fn all_open_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(ws) = &self.root_ws {
            paths.push(ws.root_path.clone());
        }
        for sw in &self.secondary {
            if let Ok(ws) = sw.state.try_lock() {
                paths.push(ws.root_path.clone());
            }
        }
        paths
    }

    fn save_session(&self) {
        let paths = self.all_open_paths();
        ipc::save_session(&paths);
    }

    fn push_recent(&mut self, path: PathBuf) {
        Ipc::add_recent(&path);
        let mut shared = self.shared.lock().unwrap();
        shared.recent_projects.retain(|p| p != &path);
        shared.recent_projects.insert(0, path);
        shared.recent_projects.truncate(10);
    }

    fn open_in_new_window(&mut self, path: PathBuf, ctx: &egui::Context) {
        // Zkontrolovat, zda projekt již není otevřen v tomto procesu
        let already_open = self.root_ws.as_ref().map(|ws| ws.root_path == path).unwrap_or(false)
            || self.secondary.iter().any(|sw| {
                sw.state.try_lock().ok().map(|ws| ws.root_path == path).unwrap_or(false)
            });
        if already_open {
            return;
        }

        let vid = egui::ViewportId::from_hash_of(format!("workspace_{}", self.next_viewport_counter));
        self.next_viewport_counter += 1;
        let panel_state = self.current_panel_state();
        let ws = init_workspace(path.clone(), &panel_state);
        self.secondary.push(SecondaryWorkspace {
            viewport_id: vid,
            state: Arc::new(Mutex::new(ws)),
        });
        Ipc::register(&path);
        self.push_recent(path);
        self.save_session();
        // Vynutit okamžitý repaint, aby se nové okno registrovalo v tomto snímku
        ctx.request_repaint();
    }

    fn process_actions(&mut self, ctx: &egui::Context) {
        let actions = std::mem::take(&mut self.shared.lock().unwrap().actions);
        for action in actions {
            match action {
                AppAction::OpenInNewWindow(path) => {
                    self.open_in_new_window(path, ctx);
                }
                AppAction::CloseWorkspace(vid) => {
                    self.secondary.retain(|sw| sw.viewport_id != vid);
                    self.save_session();
                }
                AppAction::AddRecent(path) => {
                    self.push_recent(path);
                }
                AppAction::QuitAll => {
                    self.show_quit_confirm = true;
                }
            }
        }
    }

    fn register_deferred_viewports(&self, ctx: &egui::Context) {
        for sw in &self.secondary {
            let ws_arc = Arc::clone(&sw.state);
            let shared_arc = Arc::clone(&self.shared);
            let vid = sw.viewport_id;

            let title = sw.state.try_lock()
                .map(|ws| format!("Rust Editor — {}", ws.root_path.display()))
                .unwrap_or_else(|_| "Rust Editor".to_string());

            ctx.show_viewport_deferred(
                vid,
                egui::ViewportBuilder::default()
                    .with_title(title)
                    .with_inner_size([1200.0, 800.0]),
                move |ctx, _class| {
                    // Zavření sekundárního okna — informovat root viewport
                    if ctx.input(|i| i.viewport().close_requested()) {
                        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                        shared_arc.lock().unwrap().actions.push(AppAction::CloseWorkspace(vid));
                        return;
                    }

                    let mut ws = ws_arc.lock().unwrap();
                    if let Some(new_path) = render_workspace(ctx, &mut ws, &shared_arc) {
                        let panel = ws_to_panel_state(&ws);
                        let new_path_clone = new_path.clone();
                        *ws = init_workspace(new_path, &panel);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                            format!("Rust Editor — {}", ws.root_path.display())
                        ));
                        shared_arc.lock().unwrap().actions.push(AppAction::AddRecent(new_path_clone));
                    }
                },
            );
        }
    }

    fn show_quit_confirm_dialog(&mut self, ctx: &egui::Context) {
        let modal = egui::Modal::new(egui::Id::new("quit_confirm_modal"));
        let mut confirmed = false;
        let mut cancelled = false;

        modal.show(ctx, |ui| {
            ui.heading("Ukončit aplikaci");
            ui.add_space(8.0);
            ui.label("Opravdu chcete ukončit Rust Editor?");
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Ukončit").clicked() { confirmed = true; }
                if ui.button("Zrušit").clicked() { cancelled = true; }
            });
        });

        if confirmed {
            self.save_session();
            self.quit_confirmed = true;
            self.secondary.clear(); // sekundární okna se zavřou s rootem
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if cancelled {
            self.show_quit_confirm = false;
        }
    }

    fn show_startup_dialog(&mut self, ctx: &egui::Context) {
        let mut should_open = false;
        let mut browse = false;
        let mut open_recent: Option<PathBuf> = None;

        egui::CentralPanel::default().show(ctx, |_ui| {});

        let recent_snapshot = self.shared.lock().unwrap().recent_projects.clone();

        let modal = egui::Modal::new(egui::Id::new("startup_modal"));
        modal.show(ctx, |ui| {
            let dlg_size = 15.0;
            ui.heading("Otevřít projekt");
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Cesta:").size(dlg_size));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.path_buffer)
                        .font(egui::TextStyle::Body)
                        .desired_width(350.0),
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    should_open = true;
                }
                if !response.has_focus() && !self.show_startup_wizard {
                    response.request_focus();
                }
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("Otevřít").size(dlg_size)).clicked() {
                    should_open = true;
                }
                if ui.button(egui::RichText::new("Procházet…").size(dlg_size)).clicked() {
                    browse = true;
                }
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            if ui.button(egui::RichText::new("Založit nový projekt…").size(dlg_size)).clicked() {
                self.show_startup_wizard = true;
            }

            if !recent_snapshot.is_empty() {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Nedávné projekty:").size(dlg_size));
                ui.add_space(4.0);
                for path in &recent_snapshot {
                    let name = path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.to_string_lossy().into_owned());
                    let resp = ui.add(
                        egui::Label::new(egui::RichText::new(&name).size(dlg_size))
                            .sense(egui::Sense::click()),
                    ).on_hover_text(path.to_string_lossy());
                    if resp.clicked() {
                        open_recent = Some(path.clone());
                    }
                }
            }
        });

        if browse {
            if let Some(dir) = rfd::FileDialog::new()
                .set_directory(&self.path_buffer)
                .pick_folder()
            {
                self.path_buffer = dir.to_string_lossy().to_string();
                should_open = true;
            }
        }

        if let Some(path) = open_recent {
            if path.is_dir() {
                self.open_workspace_from_startup(ctx, path);
                return;
            }
        }

        if should_open {
            let path = PathBuf::from(self.path_buffer.trim());
            if path.is_dir() {
                let path = path.canonicalize().unwrap_or(path);
                self.open_workspace_from_startup(ctx, path);
            }
        }
    }

    fn open_workspace_from_startup(&mut self, ctx: &egui::Context, path: PathBuf) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            format!("Rust Editor — {}", path.display()),
        ));
        Ipc::register(&path);
        self.push_recent(path.clone());
        let ps = self.current_panel_state();
        self.root_ws = Some(init_workspace(path, &ps));
        self.save_session();
    }

    fn show_startup_wizard_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_startup_wizard { return; }

        let modal = egui::Modal::new(egui::Id::new("startup_wizard_modal"));
        let mut close_dialog = false;
        let mut create_project: Option<(ProjectType, String, String)> = None;

        modal.show(ctx, |ui| {
            ui.heading("Nový projekt");
            ui.add_space(12.0);

            ui.label("Typ projektu:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.startup_wizard_type, ProjectType::Rust, "Rust");
                ui.radio_value(&mut self.startup_wizard_type, ProjectType::Symfony, "Symfony");
            });
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Název:");
                ui.add(egui::TextEdit::singleline(&mut self.startup_wizard_name).desired_width(250.0));
            });
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Pracovní adresář:");
                ui.add(egui::TextEdit::singleline(&mut self.startup_wizard_path).desired_width(200.0));
                if ui.button("Procházet…").clicked() {
                    if let Some(dir) = rfd::FileDialog::new()
                        .set_directory(&self.startup_wizard_path)
                        .pick_folder()
                    {
                        self.startup_wizard_path = dir.to_string_lossy().to_string();
                    }
                }
            });

            let raw_name = self.startup_wizard_name.trim();
            let display_name = if self.startup_wizard_type == ProjectType::Rust {
                raw_name.to_lowercase()
            } else {
                raw_name.to_string()
            };
            if !display_name.is_empty() {
                let preview = PathBuf::from(self.startup_wizard_path.trim())
                    .join(self.startup_wizard_type.subdir())
                    .join(&display_name);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Vytvoří se v:");
                    ui.monospace(preview.to_string_lossy().to_string());
                });
            }

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let can_create = !self.startup_wizard_name.trim().is_empty()
                    && !self.startup_wizard_path.trim().is_empty();
                if ui.add_enabled(can_create, egui::Button::new("Vytvořit")).clicked() {
                    create_project = Some((
                        self.startup_wizard_type,
                        self.startup_wizard_name.trim().to_string(),
                        self.startup_wizard_path.trim().to_string(),
                    ));
                }
                if ui.button("Zrušit").clicked() {
                    close_dialog = true;
                }
            });

            if !self.startup_wizard_error.is_empty() {
                ui.add_space(4.0);
                ui.colored_label(egui::Color32::RED, &self.startup_wizard_error);
            }
        });

        if close_dialog {
            self.show_startup_wizard = false;
            return;
        }

        if let Some((project_type, raw_name, base_path)) = create_project {
            let name = if project_type == ProjectType::Rust { raw_name.to_lowercase() } else { raw_name };
            let type_dir = PathBuf::from(&base_path).join(project_type.subdir());
            if !type_dir.exists() {
                let _ = std::fs::create_dir_all(&type_dir);
            }
            let full_path = type_dir.join(&name);
            let env = path_env();

            let result = match project_type {
                ProjectType::Rust => std::process::Command::new("cargo")
                    .args(["new", &name])
                    .current_dir(&type_dir)
                    .env("PATH", &env)
                    .output(),
                ProjectType::Symfony => std::process::Command::new("composer")
                    .args(["create-project", "symfony/skeleton", &name])
                    .current_dir(&type_dir)
                    .env("PATH", &env)
                    .output(),
            };

            self.startup_wizard_error.clear();
            match result {
                Ok(output) if output.status.success() => {
                    let path = full_path.canonicalize().unwrap_or(full_path);
                    self.show_startup_wizard = false;
                    self.startup_wizard_name.clear();
                    // Ze startup obrazovky otevřít přímo v kořenovém workspace
                    // (bez dotazu — žádný jiný projekt není otevřen)
                    self.open_workspace_from_startup(ctx, path);
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    self.startup_wizard_error = if !stderr.is_empty() {
                        stderr.to_string()
                    } else if !stdout.is_empty() {
                        stdout.to_string()
                    } else {
                        format!("Příkaz selhal s kódem: {}", output.status)
                    };
                }
                Err(e) => {
                    self.startup_wizard_error = format!("Nepodařilo se spustit příkaz: {}", e);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Drop — úklid při ukončení
// ---------------------------------------------------------------------------

impl Drop for EditorApp {
    fn drop(&mut self) {
        self.save_session();
        Ipc::unregister();
        let _ = std::fs::remove_file(ipc::process_socket_path(std::process::id()));
    }
}

// ---------------------------------------------------------------------------
// eframe::App implementace
// ---------------------------------------------------------------------------

impl eframe::App for EditorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let ps = self.current_panel_state();
        eframe::set_value(storage, STORAGE_KEY, &ps);
        self.save_session();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // IPC focus request
        if self.focus_rx.try_recv().is_ok() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // Zachytit křížek kořenového okna
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.quit_confirmed {
                // Potvrzeno — nechat zavřít
            } else if self.root_ws.is_some() {
                // Zavřít aktuální workspace (přejít na startup)
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.save_session();
                self.root_ws = None;
                ctx.send_viewport_cmd(egui::ViewportCommand::Title("Rust Editor".to_string()));
            } else {
                // Startup dialog — ukončit aplikaci
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_quit_confirm = true;
            }
        }

        // Registrovat deferred viewporty pro stávající sekundární workspacy
        // (musí být voláno každý snímek, aby okna zůstala otevřená)
        self.register_deferred_viewports(ctx);

        // Renderovat obsah kořenového viewportu
        if self.root_ws.is_none() {
            self.show_startup_dialog(ctx);
            self.show_startup_wizard_dialog(ctx);
        } else {
            // Dočasně vyjmout root_ws, aby bylo možné volat &mut self
            let mut ws = self.root_ws.take().unwrap();
            let reinit = render_workspace(ctx, &mut ws, &self.shared);
            if let Some(new_path) = reinit {
                let panel = ws_to_panel_state(&ws);
                let new_path_clone = new_path.clone();
                ws = init_workspace(new_path, &panel);
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                    format!("Rust Editor — {}", ws.root_path.display())
                ));
                self.push_recent(new_path_clone);
                self.save_session();
            }
            self.root_ws = Some(ws);
        }

        // Zpracovat akce z tohoto snímku (nové projekty, zavřené workspacy atd.)
        // Voláno ZA renderem, aby se akce z kliknutí zpracovaly okamžitě
        self.process_actions(ctx);

        // Znovu zaregistrovat viewporty — pro nové sekundární workspacy přidané výše
        self.register_deferred_viewports(ctx);

        // Dialog potvrzení ukončení
        if self.show_quit_confirm {
            self.show_quit_confirm_dialog(ctx);
        }
    }
}
