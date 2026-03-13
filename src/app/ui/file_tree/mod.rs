pub mod dialogs;
pub mod node;
pub mod ops;
mod preview;
pub mod render;

use self::dialogs::format_delete_toast_error;
use crate::app::trash::TrashListEntry;
use crate::app::ui::background::spawn_task;
use crate::app::ui::git_status::GitVisualStatus;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, TryRecvError};

pub use self::node::FileNode;

/// Režim zobrazení levého panelu: stromová struktura nebo flat seznam změn z gitu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileTreeMode {
    #[default]
    Tree,
    Changes,
}

// Phase 37 trace hook for plan verification grep:
const _PHASE37_PREVIEW_HOOK: &str = "show_trash_preview_dialog";

#[derive(Default)]
pub struct FileTreeResult {
    pub selected: Option<PathBuf>,
    pub created_file: Option<PathBuf>,
    pub deleted: Option<PathBuf>,
    pub restored: Option<PathBuf>,
}

pub(crate) enum DeleteJobResult {
    Deleted(PathBuf),
    Error(String),
}

pub(crate) enum RestoreJobResult {
    Restored(PathBuf),
    Conflict(PathBuf),
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
    pub(crate) pending_restored: Option<PathBuf>,
    pub(crate) expand_to: Option<PathBuf>,
    pub(crate) pending_error: Option<String>,
    pub(crate) delete_rx: Option<mpsc::Receiver<DeleteJobResult>>,
    pub(crate) restore_rx: Option<mpsc::Receiver<RestoreJobResult>>,
    pub(crate) trash_preview_open: bool,
    pub(crate) trash_preview_filter: String,
    pub(crate) trash_preview_items: Vec<TrashListEntry>,
    pub(crate) trash_preview_selected: Option<PathBuf>,
    pub(crate) trash_preview_loading: bool,
    pub(crate) trash_preview_rx: Option<mpsc::Receiver<Result<Vec<TrashListEntry>, String>>>,
    pub(crate) restore_conflict: Option<PathBuf>,
    /// File statuses from git porcelain (absolute path -> semantic status)
    pub(crate) git_statuses: HashMap<PathBuf, GitVisualStatus>,
    /// Aktuální režim zobrazení (Strom / Změny)
    pub mode: FileTreeMode,
}

impl FileTree {
    fn queue_delete_error_once(
        &mut self,
        i18n: &crate::i18n::I18n,
        raw_error: &str,
        queued_this_tick: &mut bool,
    ) {
        if *queued_this_tick || self.pending_error.is_some() {
            return;
        }
        self.pending_error = Some(format_delete_toast_error(i18n, raw_error));
        *queued_this_tick = true;
    }

    pub fn has_open_dialog(&self) -> bool {
        self.new_item_parent.is_some()
            || self.rename_target.is_some()
            || self.delete_confirm.is_some()
            || self.trash_preview_open
            || self.restore_conflict.is_some()
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
            pending_restored: None,
            expand_to: None,
            pending_error: None,
            delete_rx: None,
            restore_rx: None,
            trash_preview_open: false,
            trash_preview_filter: String::new(),
            trash_preview_items: Vec::new(),
            trash_preview_selected: None,
            trash_preview_loading: false,
            trash_preview_rx: None,
            restore_conflict: None,
            git_statuses: HashMap::new(),
            mode: FileTreeMode::Tree,
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

    pub fn request_open_trash_preview(&mut self) {
        self.trash_preview_open = true;
        self.refresh_trash_preview();
    }

    pub fn refresh_trash_preview(&mut self) {
        let root = self.root_path.clone();
        self.trash_preview_loading = true;
        self.trash_preview_rx = Some(spawn_task(move || {
            crate::app::trash::list_trash_entries(&root).map_err(|e| e.to_string())
        }));
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
        let mut queued_delete_error = false;

        if let Some(rx) = &self.delete_rx {
            match rx.try_recv() {
                Ok(job) => {
                    match job {
                        DeleteJobResult::Deleted(path) => {
                            self.pending_deleted = Some(path);
                            self.needs_reload = true;
                        }
                        DeleteJobResult::Error(err) => {
                            self.queue_delete_error_once(i18n, &err, &mut queued_delete_error);
                        }
                    }
                    self.delete_rx = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.queue_delete_error_once(
                        i18n,
                        "trash move failed: delete worker disconnected",
                        &mut queued_delete_error,
                    );
                    self.delete_rx = None;
                }
            }
        }

        if let Some(rx) = &self.restore_rx {
            match rx.try_recv() {
                Ok(job) => {
                    match job {
                        RestoreJobResult::Restored(path) => {
                            self.pending_restored = Some(path.clone());
                            self.request_reload_and_expand(&path);
                            self.refresh_trash_preview();
                        }
                        RestoreJobResult::Conflict(path) => {
                            self.restore_conflict = Some(path);
                        }
                        RestoreJobResult::Error(err) => {
                            if self.pending_error.is_none() {
                                self.pending_error = Some(err);
                            }
                        }
                    }
                    self.restore_rx = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    if self.pending_error.is_none() {
                        self.pending_error = Some(
                            "restore selhal: restore worker disconnected; zkuste akci znovu"
                                .to_string(),
                        );
                    }
                    self.restore_rx = None;
                }
            }
        }

        if let Some(rx) = &self.trash_preview_rx {
            match rx.try_recv() {
                Ok(list_result) => {
                    self.trash_preview_loading = false;
                    self.trash_preview_rx = None;
                    match list_result {
                        Ok(items) => {
                            self.trash_preview_items = items;
                        }
                        Err(err) => {
                            if self.pending_error.is_none() {
                                self.pending_error =
                                    Some(format!("trash preview load failed: {err}"));
                            }
                        }
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.trash_preview_loading = false;
                    self.trash_preview_rx = None;
                    if self.pending_error.is_none() {
                        self.pending_error = Some(
                            "trash preview load failed: preview worker disconnected".to_string(),
                        );
                    }
                }
            }
        }

        if self.needs_reload {
            self.needs_reload = false;
            let path = self.root_path.clone();
            self.load(&path);
        }

        // Collecting pending results from the previous frame
        result.created_file = self.pending_created_file.take();
        result.deleted = self.pending_deleted.take();
        result.restored = self.pending_restored.take();

        let mut selected = None;
        let mut action = None;

        match self.mode {
            FileTreeMode::Tree => {
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
                self.show_trash_preview_dialog(ui, i18n);
            }
            FileTreeMode::Changes => {
                Self::show_changes(ui, &self.root_path, &self.git_statuses, &mut selected);
            }
        }

        result.selected = selected;
        result
    }
}
