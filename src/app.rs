use std::path::PathBuf;

use eframe::egui;

use crate::editor::Editor;
use crate::file_tree::FileTree;
use crate::terminal::Terminal;
use crate::watcher::FileWatcher;

#[derive(PartialEq, Clone, Copy)]
enum FocusedTerminal {
    Build,
    Claude,
}

pub struct EditorApp {
    file_tree: FileTree,
    editor: Editor,
    watcher: FileWatcher,
    claude_terminal: Option<Terminal>,
    build_terminal: Option<Terminal>,
    focused_terminal: FocusedTerminal,
    root_path: PathBuf,
}

impl EditorApp {
    pub fn new(root_path: PathBuf) -> Self {
        let mut file_tree = FileTree::new();
        file_tree.load(&root_path);

        Self {
            file_tree,
            editor: Editor::new(),
            watcher: FileWatcher::new(),
            claude_terminal: None,
            build_terminal: None,
            focused_terminal: FocusedTerminal::Claude,
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

        // Autosave
        self.editor.try_autosave();

        // Request repaint for autosave and watcher checks
        ctx.request_repaint_after(std::time::Duration::from_millis(250));

        // Right panel — Claude terminál
        let focused = self.focused_terminal;
        egui::SidePanel::right("claude_panel")
            .default_width(400.0)
            .width_range(200.0..=600.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Claude");
                ui.separator();

                if let Some(terminal) = &mut self.claude_terminal {
                    if terminal.ui(ui, focused == FocusedTerminal::Claude) {
                        self.focused_terminal = FocusedTerminal::Claude;
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
                            if let Some(selected) = self.file_tree.ui(ui) {
                                self.open_file(selected);
                            }
                        });
                });

                ui.separator();

                // Dolní část — build terminál
                ui.heading("Build");
                ui.separator();

                if let Some(terminal) = &mut self.build_terminal {
                    if terminal.ui(ui, focused == FocusedTerminal::Build) {
                        self.focused_terminal = FocusedTerminal::Build;
                    }
                }
            });

        // Central panel — editor
        egui::CentralPanel::default().show(ctx, |ui| {
            self.editor.ui(ui);
        });
    }
}
