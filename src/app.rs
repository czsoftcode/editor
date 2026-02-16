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

struct WorkspaceState {
    file_tree: FileTree,
    editor: Editor,
    watcher: FileWatcher,
    project_watcher: ProjectWatcher,
    claude_terminal: Option<Terminal>,
    build_terminal: Option<Terminal>,
    focused_panel: FocusedPanel,
    root_path: PathBuf,
}

pub struct EditorApp {
    state: Option<WorkspaceState>,
    path_buffer: String,
}

impl EditorApp {
    pub fn new(root_path: Option<PathBuf>) -> Self {
        let state = root_path.map(|p| Self::init_workspace(p));
        Self {
            path_buffer: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/"))
                .to_string_lossy()
                .to_string(),
            state,
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
        }
    }

    fn open_file(ws: &mut WorkspaceState, path: PathBuf) {
        ws.editor.open_file(&path);
        if let Some(parent) = path.parent() {
            ws.watcher.watch(parent);
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.state.is_none() {
            self.show_startup_dialog(ctx);
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
                if !response.has_focus() {
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
            ws.claude_terminal = Some(Terminal::new(0, ctx, &ws.root_path));
        }
        if ws.build_terminal.is_none() {
            ws.build_terminal = Some(Terminal::new(1, ctx, &ws.root_path));
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

        // Right panel — Claude terminál
        let dialog_open = ws.file_tree.has_open_dialog();
        let focused = ws.focused_panel;
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

        // Left panel — file tree nahoře + build terminál dole
        egui::SidePanel::left("left_panel")
            .default_width(300.0)
            .width_range(200.0..=500.0)
            .resizable(true)
            .show(ctx, |ui| {
                // Horní část — file tree
                let total_height = ui.available_height();
                let tree_height = (total_height * 0.55).max(100.0);

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
            });

        // Central panel — editor
        egui::CentralPanel::default().show(ctx, |ui| {
            if ws.editor.ui(ui, dialog_open) {
                ws.focused_panel = FocusedPanel::Editor;
            }
        });
    }
}
