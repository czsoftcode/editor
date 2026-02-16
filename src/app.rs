use std::path::PathBuf;

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
}

pub struct EditorApp {
    state: Option<WorkspaceState>,
    path_buffer: String,
    show_new_project: bool,
    new_project_type: ProjectType,
    new_project_name: String,
    new_project_path: String,
    new_project_error: String,
}

impl EditorApp {
    pub fn new(root_path: Option<PathBuf>) -> Self {
        let state = root_path.map(|p| Self::init_workspace(p));
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
            show_new_project: false,
            new_project_type: ProjectType::Rust,
            new_project_name: String::new(),
            new_project_path: projects_dir,
            new_project_error: String::new(),
        }
    }

    fn init_workspace(root_path: PathBuf) -> WorkspaceState {
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
            show_left_panel: true,
            show_right_panel: true,
            show_build_terminal: true,
            show_about: false,
        }
    }

    fn open_file(ws: &mut WorkspaceState, path: PathBuf) {
        ws.editor.open_file(&path);
        if let Some(parent) = path.parent() {
            ws.watcher.watch(parent);
        }
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
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
            let path_env = {
                let current = std::env::var("PATH").unwrap_or_default();
                let cargo_bin = home.join(".cargo/bin");
                let local_bin = home.join(".local/bin");
                format!("{}:{}:{}", cargo_bin.display(), local_bin.display(), current)
            };

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
                    self.state = Some(Self::init_workspace(path));
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.state.is_none() {
            self.show_startup_dialog(ctx);
            self.show_new_project_dialog(ctx);
            return;
        }

        self.update_workspace(ctx);
    }
}

impl EditorApp {
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
                self.state = Some(Self::init_workspace(path));
            }
        }
    }

    fn update_workspace(&mut self, ctx: &egui::Context) {
        let ws = self.state.as_mut().unwrap();

        // Lazy init terminálů
        if ws.claude_terminal.is_none() {
            ws.claude_terminal = Some(Terminal::new(0, ctx, &ws.root_path, Some("claude")));
        }
        if ws.build_terminal.is_none() {
            ws.build_terminal = Some(Terminal::new(1, ctx, &ws.root_path, None));
        }

        // Check for file changes from watcher
        if let Some(changed_path) = ws.watcher.try_recv() {
            if let Some(editor_path) = &ws.editor.path {
                if let (Ok(a), Ok(b)) = (changed_path.canonicalize(), editor_path.canonicalize()) {
                    if a == b && !ws.editor.modified {
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
                        if let Some(editor_path) = &ws.editor.path {
                            if editor_path == path || editor_path.starts_with(path) {
                                ws.editor.clear();
                            }
                        }
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

        // Autosave
        ws.editor.try_autosave();

        // Request repaint for autosave and watcher checks
        ctx.request_repaint_after(std::time::Duration::from_millis(250));

        // Ctrl+S shortcut
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            ws.editor.save();
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
                    if ui.button("Zavřít soubor").clicked() {
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
                    let right_label = if ws.show_right_panel { "✓ Claude" } else { "  Claude" };
                    if ui.button(right_label).clicked() {
                        action_toggle_right = true;
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
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
                self.state = Some(Self::init_workspace(path));
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
                self.state = Some(Self::init_workspace(path));
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

        // Right panel — Claude terminál
        let dialog_open = ws.file_tree.has_open_dialog();
        let focused = ws.focused_panel;
        if ws.show_right_panel {
            egui::SidePanel::right("claude_panel")
                .default_width(400.0)
                .width_range(200.0..=600.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Claude");
                    ui.separator();

                    if !dialog_open {
                        if let Some(terminal) = &mut ws.claude_terminal {
                            if terminal.ui(ui, focused == FocusedPanel::Claude) {
                                ws.focused_panel = FocusedPanel::Claude;
                            }
                        }
                    }
                });
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
                                    if let Some(editor_path) = &ws.editor.path {
                                        if editor_path.starts_with(&deleted) || *editor_path == deleted {
                                            ws.editor.clear();
                                        }
                                    }
                                }
                            });
                    });

                    if ws.show_build_terminal {
                        ui.separator();

                        // Dolní část — build terminál
                        ui.heading("Build");
                        ui.separator();

                        if !dialog_open {
                            if let Some(terminal) = &mut ws.build_terminal {
                                if terminal.ui(ui, focused == FocusedPanel::Build) {
                                    ws.focused_panel = FocusedPanel::Build;
                                }
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
    }
}
