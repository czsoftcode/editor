use crate::app::ui::search_picker::collect_project_files;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Shared index of all project files.
///
/// This structure provides a thread-safe way to access the list of all files
/// in the project, which is used by Ctrl+P, global search, and potentially the file tree.
pub(crate) struct ProjectIndex {
    root: PathBuf,
    files: Arc<Mutex<Arc<Vec<PathBuf>>>>,
}

impl ProjectIndex {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            files: Arc::new(Mutex::new(Arc::new(Vec::new()))),
        }
    }

    /// Returns a reference-counted handle to the current file list (relative paths).
    pub fn get_files(&self) -> Arc<Vec<PathBuf>> {
        Arc::clone(
            &self
                .files
                .lock()
                .expect("Failed to lock ProjectIndex files"),
        )
    }

    /// Triggers a full re-scan of the project in the background.
    pub fn full_rescan(&self) {
        let root = self.root.clone();
        let files_arc = Arc::clone(&self.files);

        std::thread::spawn(move || {
            let new_files = collect_project_files(&root);
            let mut lock = files_arc
                .lock()
                .expect("Failed to lock ProjectIndex files for full re-scan");
            *lock = Arc::new(new_files);
        });
    }

    /// Updates the index based on a file system change.
    /// This is more efficient than a full re-scan.
    pub fn handle_change(&self, change: crate::watcher::FsChange) {
        match change {
            crate::watcher::FsChange::Created(path) => {
                if path.is_file()
                    && let Ok(rel) = path.strip_prefix(&self.root)
                {
                    let mut lock = self
                        .files
                        .lock()
                        .expect("Failed to lock ProjectIndex files for creation");
                    let rel_path = rel.to_path_buf();
                    if !lock.contains(&rel_path) {
                        let mut new_vec = (**lock).clone();
                        new_vec.push(rel_path);
                        new_vec.sort();
                        *lock = Arc::new(new_vec);
                    }
                }
            }
            crate::watcher::FsChange::Removed(path) => {
                if let Ok(rel) = path.strip_prefix(&self.root) {
                    let mut lock = self
                        .files
                        .lock()
                        .expect("Failed to lock ProjectIndex files for removal");
                    let rel_path = rel.to_path_buf();
                    if lock.contains(&rel_path) {
                        let mut new_vec = (**lock).clone();
                        new_vec.retain(|p| p != &rel_path);
                        *lock = Arc::new(new_vec);
                    }
                }
            }
            crate::watcher::FsChange::Modified(_) => {
                // No need to update the file list itself on modification,
                // but we might want to trigger a content search cache update in the future.
            }
        }
    }
}
