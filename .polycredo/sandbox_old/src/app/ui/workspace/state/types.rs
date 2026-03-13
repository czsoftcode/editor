use crate::app::ui::workspace::state::WorkspaceState;
use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::mpsc;

/// Result of an asynchronous folder selection.
pub type FolderPickResult = (Option<PathBuf>, bool);

// ---------------------------------------------------------------------------
// FilePicker — Ctrl+P quick file opening
// ---------------------------------------------------------------------------

pub struct FilePicker {
    pub query: String,
    pub files: Arc<Vec<PathBuf>>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub focus_requested: bool,
}

impl FilePicker {
    pub fn new(files: Arc<Vec<PathBuf>>) -> Self {
        let filtered: Vec<usize> = (0..files.len()).collect();
        Self {
            query: String::new(),
            files,
            filtered,
            selected: 0,
            focus_requested: true,
        }
    }

    pub fn update_filter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = self
            .files
            .iter()
            .enumerate()
            .filter(|(_, p)| crate::app::ui::search_picker::fuzzy_match(&q, &p.to_string_lossy()))
            .map(|(i, _)| i)
            .collect();
        self.selected = 0;
    }
}

// ---------------------------------------------------------------------------
// ProjectSearch — project-wide search
// ---------------------------------------------------------------------------

pub struct SearchResult {
    pub file: PathBuf,
    pub line: usize,
    pub text: String,
}

pub struct ProjectSearch {
    pub show_input: bool,
    pub query: String,
    pub results: Vec<SearchResult>,
    pub rx: Option<mpsc::Receiver<Vec<SearchResult>>>,
    pub focus_requested: bool,
    pub cancel_epoch: Arc<AtomicU64>,
}

impl Default for ProjectSearch {
    fn default() -> Self {
        Self {
            show_input: false,
            query: String::new(),
            results: Vec::new(),
            rx: None,
            focus_requested: false,
            cancel_epoch: Arc::new(AtomicU64::new(0)),
        }
    }
}

// ---------------------------------------------------------------------------
// Background I/O Results
// ---------------------------------------------------------------------------

pub enum FsChangeResult {
    AiDiff(String, String, String),
    LocalHistory(PathBuf, String),
}

pub struct SecondaryWorkspace {
    pub viewport_id: egui::ViewportId,
    pub state: Arc<Mutex<WorkspaceState>>,
    pub close_requested: Arc<AtomicBool>,
}
