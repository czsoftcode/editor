use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui;

use crate::editor::Editor;
use crate::file_tree::FileTree;
use crate::terminal::Terminal;
use crate::watcher::{FileWatcher, FsChange, ProjectWatcher};

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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct PersistentState {
    show_left_panel: bool,
    show_right_panel: bool,
    show_build_terminal: bool,
    claude_float: bool,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            show_left_panel: true,
            show_right_panel: true,
            show_build_terminal: true,
            claude_float: false,
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
    build_errors: Vec<BuildError>,
    build_error_rx: Option<mpsc::Receiver<Vec<BuildError>>>,
    claude_tool: AiTool,
    claude_float: bool,
}

pub struct EditorApp {
    state: Option<WorkspaceState>,
    saved_panel_state: PersistentState, // záloha pro případ, že workspace ještě není otevřen
    path_buffer: String,
    show_new_project: bool,
    new_project_type: ProjectType,
    new_project_name: String,
    new_project_path: String,
    new_project_error: String,
    show_quit_confirm: bool,
    quit_confirmed: bool,
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext, root_path: Option<PathBuf>) -> Self {
        let panel_state: PersistentState = cc.storage
            .and_then(|s| eframe::get_value(s, STORAGE_KEY))
            .unwrap_or_default();

        let state = root_path.map(|p| Self::init_workspace(p, &panel_state));
        let home = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .to_string_lossy()
            .to_string();
        let projects_dir = {
            let mut p = PathBuf::from(&home);
            p.push("MyProject");
            p.to_string_lossy().to_string()
        };
        Self {
            path_buffer: home,
            state,
            saved_panel_state: panel_state,
            show_new_project: false,
            new_project_type: ProjectType::Rust,
            new_project_name: String::new(),
            new_project_path: projects_dir,
            new_project_error: String::new(),
            show_quit_confirm: false,
            quit_confirmed: false,
        }
    }

    fn current_panel_state(&self) -> PersistentState {
        self.state.as_ref().map(|ws| PersistentState {
            show_left_panel: ws.show_left_panel,
            show_right_panel: ws.show_right_panel,
            show_build_terminal: ws.show_build_terminal,
            claude_float: ws.claude_float,
        }).unwrap_or_else(|| self.saved_panel_state.clone())
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
            build_errors: Vec::new(),
            build_error_rx: None,
            claude_tool: AiTool::ClaudeCode,
            claude_float: panel_state.claude_float,
        }
    }

    fn open_file(ws: &mut WorkspaceState, path: PathBuf) {
        ws.editor.open_file(&path);
        if let Some(parent) = path.parent() {
            ws.watcher.watch(parent);
        }
    }

    fn path_env() -> String {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let current = std::env::var("PATH").unwrap_or_default();
        let cargo_bin = home.join(".cargo/bin");
        let local_bin = home.join(".local/bin");
        format!("{}:{}:{}", cargo_bin.display(), local_bin.display(), current)
    }

    fn run_build_check(root_path: PathBuf) -> mpsc::Receiver<Vec<BuildError>> {
        let (tx, rx) = mpsc::channel();
        let path_env = Self::path_env();
        std::thread::spawn(move || {
            let output = std::process::Command::new("cargo")
                .args(["build", "--color=never"])
                .current_dir(&root_path)
                .env("PATH", &path_env)
                .output();

            if let Ok(output) = output {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let errors = parse_build_errors(&stderr);
                let _ = tx.send(errors);
            }
        });
        rx
    }

    /// Zobrazí dialog pro vytvoření nového projektu.
    /// Vrací `true` pokud byl projekt právě vytvořen (a workspace reinicializován).
    fn show_new_project_dialog(&mut self, ctx: &egui::Context) -> bool {
        if !self.show_new_project {
            return false;
        }

        let modal = egui::Modal::new(egui::Id::new("new_project_modal"));
        let mut close_dialog = false;
        let mut create_project: Option<(ProjectType, String, String)> = None;

        modal.show(ctx, |ui| {
            ui.heading("Nový projekt");
            ui.add_space(12.0);

            ui.label("Typ projektu:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.new_project_type, ProjectType::Rust, "Rust");
                ui.radio_value(&mut self.new_project_type, ProjectType::Symfony, "Symfony");
            });
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Název:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_project_name)
                        .desired_width(250.0),
                );
            });
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Pracovní adresář:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_project_path)
                        .desired_width(200.0),
                );
                if ui.button("Procházet…").clicked() {
                    if let Some(dir) = rfd::FileDialog::new()
                        .set_directory(&self.new_project_path)
                        .pick_folder()
                    {
                        self.new_project_path = dir.to_string_lossy().to_string();
                    }
                }
            });

            // Náhled výsledné cesty
            let raw_name = self.new_project_name.trim();
            let display_name = if self.new_project_type == ProjectType::Rust {
                raw_name.to_lowercase()
            } else {
                raw_name.to_string()
            };
            if !display_name.is_empty() {
                let preview = PathBuf::from(self.new_project_path.trim())
                    .join(self.new_project_type.subdir())
                    .join(&display_name);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Vytvoří se v:");
                    ui.monospace(preview.to_string_lossy().to_string());
                });
            }

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                let can_create = !self.new_project_name.trim().is_empty()
                    && !self.new_project_path.trim().is_empty();
                if ui.add_enabled(can_create, egui::Button::new("Vytvořit")).clicked() {
                    create_project = Some((
                        self.new_project_type,
                        self.new_project_name.trim().to_string(),
                        self.new_project_path.trim().to_string(),
                    ));
                }
                if ui.button("Zrušit").clicked() {
                    close_dialog = true;
                }
            });

            if !self.new_project_error.is_empty() {
                ui.add_space(4.0);
                ui.colored_label(egui::Color32::RED, &self.new_project_error);
            }
        });

        if close_dialog {
            self.show_new_project = false;
            return false;
        }

        if let Some((project_type, raw_name, base_path)) = create_project {
            let name = if project_type == ProjectType::Rust {
                raw_name.to_lowercase()
            } else {
                raw_name
            };
            let type_dir = PathBuf::from(&base_path).join(project_type.subdir());
            if !type_dir.exists() {
                let _ = std::fs::create_dir_all(&type_dir);
            }
            let full_path = type_dir.join(&name);
            let path_env = Self::path_env();

            let result = match project_type {
                ProjectType::Rust => {
                    std::process::Command::new("cargo")
                        .args(["new", &name])
                        .current_dir(&type_dir)
                        .env("PATH", &path_env)
                        .output()
                }
                ProjectType::Symfony => {
                    std::process::Command::new("composer")
                        .args(["create-project", "symfony/skeleton", &name])
                        .current_dir(&type_dir)
                        .env("PATH", &path_env)
                        .output()
                }
            };

            self.new_project_error.clear();
            match result {
                Ok(output) if output.status.success() => {
                    let path = full_path.canonicalize().unwrap_or(full_path);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                        format!("Rust Editor — {}", path.display()),
                    ));
                    let ps = self.current_panel_state();
                    self.state = Some(Self::init_workspace(path, &ps));
                    self.show_new_project = false;
                    self.new_project_name.clear();
                    return true;
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    self.new_project_error = if !stderr.is_empty() {
                        stderr.to_string()
                    } else if !stdout.is_empty() {
                        stdout.to_string()
                    } else {
                        format!("Příkaz selhal s kódem: {}", output.status)
                    };
                }
                Err(e) => {
                    self.new_project_error = format!("Nepodařilo se spustit příkaz: {}", e);
                }
            }
        }

        false
    }
}

impl eframe::App for EditorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let ps = self.current_panel_state();
        eframe::set_value(storage, STORAGE_KEY, &ps);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Zachytit křížek okna — zrušit okamžité zavření a ukázat dialog
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.quit_confirmed {
                // Uživatel potvrdil — nechat zavřít
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_quit_confirm = true;
            }
        }

        if self.state.is_none() {
            self.show_startup_dialog(ctx);
            self.show_new_project_dialog(ctx);
        } else {
            self.update_workspace(ctx);
        }

        // Potvrzovací dialog — zobrazit nad ostatním obsahem
        if self.show_quit_confirm {
            self.show_quit_confirm_dialog(ctx);
        }
    }
}

impl EditorApp {
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
                if ui.button("Ukončit").clicked() {
                    confirmed = true;
                }
                if ui.button("Zrušit").clicked() {
                    cancelled = true;
                }
            });
        });

        if confirmed {
            self.quit_confirmed = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if cancelled {
            self.show_quit_confirm = false;
        }
    }

    fn show_startup_dialog(&mut self, ctx: &egui::Context) {
        let mut should_open = false;
        let mut browse = false;

        egui::CentralPanel::default().show(ctx, |_ui| {});

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
                if response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    should_open = true;
                }
                if !response.has_focus() && !self.show_new_project {
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
                self.show_new_project = true;
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

        if should_open {
            let path = PathBuf::from(self.path_buffer.trim());
            if path.is_dir() {
                let path = path.canonicalize().unwrap_or(path);
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                    format!("Rust Editor — {}", path.display()),
                ));
                let ps = self.current_panel_state();
                self.state = Some(Self::init_workspace(path, &ps));
            }
        }
    }

    fn update_workspace(&mut self, ctx: &egui::Context) {
        let ws = self.state.as_mut().unwrap();

        // Lazy init terminálů
        if ws.claude_terminal.is_none() {
            ws.claude_terminal = Some(Terminal::new(0, ctx, &ws.root_path, Some(ws.claude_tool.command())));
        }
        if ws.build_terminal.is_none() {
            ws.build_terminal = Some(Terminal::new(1, ctx, &ws.root_path, None));
        }

        // Check for file changes from watcher
        if let Some(changed_path) = ws.watcher.try_recv() {
            if let Some(editor_path) = ws.editor.active_path() {
                if let (Ok(a), Ok(b)) = (changed_path.canonicalize(), editor_path.canonicalize()) {
                    if a == b && !ws.editor.is_modified() {
                        ws.editor.reload_from_disk();
                    }
                }
            }
        }

        // Check for project-wide filesystem changes
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
                Self::open_file(ws, path);
            }
        }

        // Check for build error results
        if let Some(rx) = &ws.build_error_rx {
            if let Ok(errors) = rx.try_recv() {
                ws.build_errors = errors;
                ws.build_error_rx = None;
            }
        }

        // Autosave
        ws.editor.try_autosave();

        // Request repaint for autosave and watcher checks
        ctx.request_repaint_after(std::time::Duration::from_millis(250));

        // Ctrl+S shortcut
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            ws.editor.save();
        }

        // Ctrl+W — zavřít aktivní záložku
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
        let mut action_toggle_left = false;
        let mut action_toggle_right = false;
        let mut action_toggle_build = false;
        let mut action_toggle_float = false;
        let mut action_about = false;

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
                    if ui.button("O aplikaci").clicked() {
                        action_about = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Handle menu actions
        if action_quit {
            self.show_quit_confirm = true;
        }
        if action_save {
            ws.editor.save();
        }
        if action_close_file {
            ws.editor.clear();
        }
        if action_toggle_left {
            ws.show_left_panel = !ws.show_left_panel;
        }
        if action_toggle_right {
            ws.show_right_panel = !ws.show_right_panel;
        }
        if action_toggle_float {
            ws.claude_float = !ws.claude_float;
        }
        if action_toggle_build {
            ws.show_build_terminal = !ws.show_build_terminal;
        }
        if action_about {
            ws.show_about = true;
        }
        if action_new_project {
            self.show_new_project = true;
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
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                    format!("Rust Editor — {}", path.display()),
                ));
                let ps = self.current_panel_state();
                self.state = Some(Self::init_workspace(path, &ps));
                return;
            }
        }
        if action_open_folder {
            if let Some(dir) = rfd::FileDialog::new()
                .set_directory(&ws.root_path)
                .pick_folder()
            {
                let path = dir.canonicalize().unwrap_or(dir);
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                    format!("Rust Editor — {}", path.display()),
                ));
                let ps = self.current_panel_state();
                self.state = Some(Self::init_workspace(path, &ps));
                return;
            }
        }

        // Re-borrow after potential action_open_folder
        let ws = self.state.as_mut().unwrap();

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

        // New project dialog
        if self.show_new_project_dialog(ctx) {
            return;
        }

        let ws = self.state.as_mut().unwrap();

        // Status bar — musí být před SidePanel, aby se roztáhl přes celou šířku
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(22.0)
            .show(ctx, |ui| {
                ws.editor.status_bar(ui);
            });

        // Right panel — Claude terminál (dokovaný nebo plovoucí)
        let dialog_open = ws.file_tree.has_open_dialog();
        let focused = ws.focused_panel;
        let mut any_terminal_clicked = false;
        if ws.show_right_panel {
            if ws.claude_float {
                // Plovoucí okno
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
                            if let Some(terminal) = &mut ws.claude_terminal {
                                if terminal.ui(ui, focused == FocusedPanel::Claude) {
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
                // Dokovaný panel
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
                            if let Some(terminal) = &mut ws.claude_terminal {
                                if terminal.ui(ui, focused == FocusedPanel::Claude) {
                                    ws.focused_panel = FocusedPanel::Claude;
                                    any_terminal_clicked = true;
                                }
                            }
                        }
                    });
            }
        }

        // Left panel — file tree nahoře + build terminál dole
        if ws.show_left_panel {
            egui::SidePanel::left("left_panel")
                .default_width(300.0)
                .width_range(200.0..=500.0)
                .resizable(true)
                .show(ctx, |ui| {
                    // Horní část — file tree
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
                                    Self::open_file(ws, path);
                                }
                                if let Some(path) = result.created_file {
                                    Self::open_file(ws, path);
                                }
                                if let Some(deleted) = result.deleted {
                                    ws.editor.close_tabs_for_path(&deleted);
                                }
                            });
                    });

                    if ws.show_build_terminal {
                        ui.separator();

                        // Build toolbar
                        ui.horizontal(|ui| {
                            ui.strong("Build");
                            ui.separator();

                            if ui.small_button("\u{25B6} Build").clicked() {
                                if let Some(terminal) = &mut ws.build_terminal {
                                    terminal.send_command("cargo build 2>&1");
                                }
                                let rx = Self::run_build_check(ws.root_path.clone());
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
                                if terminal.ui(ui, focused == FocusedPanel::Build) {
                                    ws.focused_panel = FocusedPanel::Build;
                                    any_terminal_clicked = true;
                                }
                            }
                        }

                        // Build errors list
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
                                Self::open_file(ws, path);
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

        // Focus follows mouse: pokud myš není nad žádným terminálem, přepnout fokus na editor
        let in_terminal = ws.focused_panel == FocusedPanel::Claude
            || ws.focused_panel == FocusedPanel::Build;
        if !any_terminal_clicked && in_terminal {
            ws.focused_panel = FocusedPanel::Editor;
        }
    }
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
                // Parse file:line:col
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
