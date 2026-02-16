use std::path::{Path, PathBuf};

const IGNORED_DIRS: &[&str] = &[".git", "target", "node_modules", ".idea", ".vscode", "__pycache__"];

pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub expanded: bool,
    pub children_loaded: bool,
}

impl FileNode {
    fn new(path: PathBuf, is_dir: bool) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        Self {
            name,
            path,
            is_dir,
            children: Vec::new(),
            expanded: false,
            children_loaded: !is_dir,
        }
    }

    fn load_children(&mut self) {
        if self.children_loaded || !self.is_dir {
            return;
        }
        self.children_loaded = true;

        let entries = match std::fs::read_dir(&self.path) {
            Ok(e) => e,
            Err(_) => return,
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if IGNORED_DIRS.contains(&name.as_str()) {
                continue;
            }

            let is_dir = path.is_dir();
            let node = FileNode::new(path, is_dir);
            if is_dir {
                dirs.push(node);
            } else {
                files.push(node);
            }
        }

        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        self.children = dirs;
        self.children.append(&mut files);
    }
}

pub struct FileTree {
    pub root: Option<FileNode>,
}

impl FileTree {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn load(&mut self, path: &Path) {
        let mut root = FileNode::new(path.to_path_buf(), true);
        root.expanded = true;
        root.load_children();
        self.root = Some(root);
    }

    pub fn ui(&mut self, ui: &mut eframe::egui::Ui) -> Option<PathBuf> {
        let mut selected = None;
        if let Some(root) = &mut self.root {
            selected = Self::show_node(ui, root);
        }
        selected
    }

    fn show_node(ui: &mut eframe::egui::Ui, node: &mut FileNode) -> Option<PathBuf> {
        let mut selected = None;

        let text_color = eframe::egui::Color32::from_rgb(230, 230, 230);
        let font_size = 16.0;

        if node.is_dir {
            let header_text = eframe::egui::RichText::new(format!("\u{1F4C1} {}", &node.name))
                .size(font_size)
                .color(text_color);
            let _response = eframe::egui::CollapsingHeader::new(header_text)
                .default_open(node.expanded)
                .show(ui, |ui| {
                    node.load_children();
                    for child in &mut node.children {
                        if let Some(path) = Self::show_node(ui, child) {
                            selected = Some(path);
                        }
                    }
                });
        } else {
            let file_text = eframe::egui::RichText::new(format!("\u{1F4C4} {}", &node.name))
                .size(font_size)
                .color(text_color);
            let label = ui.selectable_label(false, file_text);
            if label.clicked() {
                selected = Some(node.path.clone());
            }
        }

        selected
    }
}
