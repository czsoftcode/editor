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

pub struct EditorApp {
    file_tree: FileTree,
    editor: Editor,
    watcher: FileWatcher,
    project_watcher: ProjectWatcher,
    claude_terminal: Option<Terminal>,
    build_terminal: Option<Terminal>,
    focused_panel: FocusedPanel,
    root_path: PathBuf,
}

impl EditorApp {
    pub fn new(root_path: PathBuf) -> Self {
        let mut file_tree = FileTree::new();
        file_tree.load(&root_path);
        let project_watcher = ProjectWatcher::new(&root_path);

        Self {
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

    fn open_file(&mut self, path: PathBuf) {
        self.editor.open_file(&path);
        if let Some(parent) = path.parent() {
            self.watcher.watch(parent);
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Lazy init terminálů
        if self.claude_terminal.is_none() {
            self.claude_terminal = Some(Terminal::new(0, ctx, &self.root_path));
        }
        if self.build_terminal.is_none() {
            self.build_terminal = Some(Terminal::new(1, ctx, &self.root_path));
        }

        // Check for file changes from watcher
        if let Some(changed_path) = self.watcher.try_recv() {
            if let Some(editor_path) = &self.editor.path {
                if let (Ok(a), Ok(b)) = (changed_path.canonicalize(), editor_path.canonicalize()) {
                    if a == b && !self.editor.modified {
                        self.editor.reload_from_disk();
                    }
                }
            }
        }

        // Check for project-wide filesystem changes
        let fs_changes = self.project_watcher.poll();
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
                        if let Some(editor_path) = &self.editor.path {
                            if editor_path == path || editor_path.starts_with(path) {
                                self.editor.clear();
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
                    self.file_tree.request_reload_and_expand(path);
                } else {
                    self.file_tree.request_reload();
                }
            }
            if let Some(path) = open_file {
                self.open_file(path);
            }
        }

        // Autosave
        self.editor.try_autosave();

        // Request repaint for autosave and watcher checks
        ctx.request_repaint_after(std::time::Duration::from_millis(250));

        // Right panel — Claude terminál
        let dialog_open = self.file_tree.has_open_dialog();
        let focused = self.focused_panel;
        egui::SidePanel::right("claude_panel")
            .default_width(400.0)
            .width_range(200.0..=600.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Claude");
                ui.separator();

                if !dialog_open {
                    if let Some(terminal) = &mut self.claude_terminal {
                        if terminal.ui(ui, focused == FocusedPanel::Claude) {
                            self.focused_panel = FocusedPanel::Claude;
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
                            let result = self.file_tree.ui(ui);
                            if let Some(path) = result.selected {
                                self.open_file(path);
                            }
                            if let Some(path) = result.created_file {
                                self.open_file(path);
                            }
                            if let Some(deleted) = result.deleted {
                                if let Some(editor_path) = &self.editor.path {
                                    if editor_path.starts_with(&deleted) || *editor_path == deleted {
                                        self.editor.clear();
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
                    if let Some(terminal) = &mut self.build_terminal {
                        if terminal.ui(ui, focused == FocusedPanel::Build) {
                            self.focused_panel = FocusedPanel::Build;
                        }
                    }
                }
            });

        // Central panel — editor
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.editor.ui(ui, dialog_open) {
                self.focused_panel = FocusedPanel::Editor;
            }
        });
    }
}
