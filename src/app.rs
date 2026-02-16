use std::path::PathBuf;

use eframe::egui;

use crate::editor::Editor;
use crate::file_tree::FileTree;
use crate::watcher::FileWatcher;

pub struct EditorApp {
    file_tree: FileTree,
    editor: Editor,
    watcher: FileWatcher,
    _root_path: PathBuf,
}

impl EditorApp {
    pub fn new(root_path: PathBuf) -> Self {
        let mut file_tree = FileTree::new();
        file_tree.load(&root_path);

        Self {
            file_tree,
            editor: Editor::new(),
            watcher: FileWatcher::new(),
            _root_path: root_path,
        }
    }

    fn open_file(&mut self, path: PathBuf) {
        self.editor.open_file(&path);
        // Watch the parent directory for changes
        if let Some(parent) = path.parent() {
            self.watcher.watch(parent);
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        // Left panel — file tree
        egui::SidePanel::left("file_tree_panel")
            .default_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
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

        // Central panel — editor
        egui::CentralPanel::default().show(ctx, |ui| {
            self.editor.ui(ui);
        });
    }
}
