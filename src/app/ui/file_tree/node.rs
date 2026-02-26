use std::path::PathBuf;

pub const IGNORED_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".idea",
    ".vscode",
    "__pycache__",
    ".polycredo",
];

pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub expanded: bool,
    pub children_loaded: bool,
    pub line_count: Option<usize>,
}

impl FileNode {
    pub fn new(path: PathBuf, is_dir: bool) -> Self {
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
            line_count: None,
        }
    }

    pub fn load_children(&mut self) {
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
