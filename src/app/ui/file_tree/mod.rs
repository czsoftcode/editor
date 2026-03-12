pub mod dialogs;
pub mod node;
pub mod ops;
pub mod render;

use self::dialogs::format_delete_toast_error;
use crate::app::ui::git_status::GitVisualStatus;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub use self::node::FileNode;

#[derive(Default)]
pub struct FileTreeResult {
    pub selected: Option<PathBuf>,
    pub created_file: Option<PathBuf>,
    pub deleted: Option<PathBuf>,
}

pub(crate) enum DeleteJobResult {
    Deleted(PathBuf),
    Error(String),
}

pub struct FileTree {
    pub root: Option<FileNode>,
    pub(crate) root_path: PathBuf,
    pub(crate) clipboard: Option<PathBuf>,
    pub(crate) rename_target: Option<PathBuf>,
    pub(crate) rename_buffer: String,
    pub(crate) new_item_parent: Option<PathBuf>,
    pub(crate) new_item_buffer: String,
    pub(crate) new_item_is_dir: bool,
    pub(crate) delete_confirm: Option<PathBuf>,
    pub(crate) needs_reload: bool,
    pub(crate) pending_created_file: Option<PathBuf>,
    pub(crate) pending_deleted: Option<PathBuf>,
    pub(crate) expand_to: Option<PathBuf>,
    pub(crate) pending_error: Option<String>,
    pub(crate) delete_rx: Option<mpsc::Receiver<DeleteJobResult>>,
    /// File statuses from git porcelain (absolute path -> semantic status)
    pub(crate) git_statuses: HashMap<PathBuf, GitVisualStatus>,
}

impl FileTree {
    pub fn has_open_dialog(&self) -> bool {
        self.new_item_parent.is_some()
            || self.rename_target.is_some()
            || self.delete_confirm.is_some()
    }

    pub fn new() -> Self {
        Self {
            root: None,
            root_path: PathBuf::new(),
            clipboard: None,
            rename_target: None,
            rename_buffer: String::new(),
            new_item_parent: None,
            new_item_buffer: String::new(),
            new_item_is_dir: false,
            delete_confirm: None,
            needs_reload: false,
            pending_created_file: None,
            pending_deleted: None,
            expand_to: None,
            pending_error: None,
            delete_rx: None,
            git_statuses: HashMap::new(),
        }
    }

    /// Sets the mapping of absolute paths to semantic git statuses.
    pub fn set_git_statuses(&mut self, statuses: HashMap<PathBuf, GitVisualStatus>) {
        self.git_statuses = statuses;
    }

    /// Fetches a potential I/O operation error (to be displayed in a toast notification).
    pub fn take_error(&mut self) -> Option<String> {
        self.pending_error.take()
    }

    pub fn request_reload(&mut self) {
        self.needs_reload = true;
    }

    pub fn request_reload_and_expand(&mut self, target: &Path) {
        self.needs_reload = true;
        self.expand_to = Some(target.to_path_buf());
    }

    pub fn load(&mut self, path: &Path) {
        self.root_path = path.to_path_buf();
        let mut root = FileNode::new(path.to_path_buf(), true);
        root.expanded = true;
        root.load_children();
        self.root = Some(root);
    }

    pub fn ui(&mut self, ui: &mut eframe::egui::Ui, i18n: &crate::i18n::I18n) -> FileTreeResult {
        let mut result = FileTreeResult::default();

        if let Some(rx) = &self.delete_rx
            && let Ok(job) = rx.try_recv()
        {
            match job {
                DeleteJobResult::Deleted(path) => {
                    self.pending_deleted = Some(path);
                    self.needs_reload = true;
                }
                DeleteJobResult::Error(err) => {
                    self.pending_error = Some(format_delete_toast_error(i18n, &err));
                }
            }
            self.delete_rx = None;
        }

        if self.needs_reload {
            self.needs_reload = false;
            let path = self.root_path.clone();
            self.load(&path);
        }

        // Collecting pending results from the previous frame
        result.created_file = self.pending_created_file.take();
        result.deleted = self.pending_deleted.take();

        let mut selected = None;
        let mut action = None;

        let expand_to = self.expand_to.take();
        if let Some(root) = &mut self.root {
            let has_clipboard = self.clipboard.is_some();
            Self::show_node(
                ui,
                root,
                &mut selected,
                &mut action,
                has_clipboard,
                &expand_to,
                &self.git_statuses,
                i18n,
            );
        }

        if let Some(act) = action {
            self.handle_action(act, i18n);
        }

        self.show_dialogs(ui, i18n);

        result.selected = selected;
        result
    }
}
