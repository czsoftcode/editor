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
    /// Byte rozsahy matchů v rámci `text` (start, end).
    pub match_ranges: Vec<(usize, usize)>,
    /// Kontextové řádky před matchem.
    pub context_before: Vec<String>,
    /// Kontextové řádky za matchem.
    pub context_after: Vec<String>,
}

/// Volby pro vyhledávání — togglery v search dialogu.
#[derive(Clone, Debug, Default)]
pub struct SearchOptions {
    pub use_regex: bool,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub file_filter: String,
}

/// Dávka výsledků ze search threadu.
pub enum SearchBatch {
    /// Výsledky z jednoho souboru.
    Results(Vec<SearchResult>),
    /// Search dokončen.
    Done,
    /// Chyba při vyhledávání (I/O, regex, atd.).
    Error(String),
}

pub struct ProjectSearch {
    pub show_input: bool,
    pub query: String,
    pub results: Vec<SearchResult>,
    pub rx: Option<mpsc::Receiver<SearchBatch>>,
    pub focus_requested: bool,
    pub cancel_epoch: Arc<AtomicU64>,
    /// Volby vyhledávání (regex/case/word/filtr).
    pub options: SearchOptions,
    /// Chybová hláška z nevalidního regexu.
    pub regex_error: Option<String>,
    /// Text pro replace (připraveno pro S02).
    pub replace_text: String,
    /// Zobrazit replace UI.
    pub show_replace: bool,
    /// Indikátor probíhajícího vyhledávání.
    pub searching: bool,
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
            options: SearchOptions::default(),
            regex_error: None,
            replace_text: String::new(),
            show_replace: false,
            searching: false,
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
