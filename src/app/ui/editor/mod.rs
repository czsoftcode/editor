mod files;
mod markdown;
mod search;
mod tabs;
mod ui;

pub mod diff_view;
pub mod render;

use std::path::PathBuf;
use std::time::Instant;

use eframe::egui;

use crate::app::ui::widgets::tab_bar::TabBarAction;
use crate::highlighter::Highlighter;

const AUTOSAVE_DELAY_MS: u128 = 500;
/// Hover debounce: wait this long after mouse stops before sending request.
pub const LSP_HOVER_DEBOUNCE_MS: u128 = 600;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Content of an active LSP hover popup.
pub struct LspHoverPopup {
    pub content: String,
    pub screen_pos: egui::Pos2,
}

/// State of the active LSP completion dropdown.
pub struct LspCompletionState {
    pub items: Vec<async_lsp::lsp_types::CompletionItem>,
    pub selected: usize,
    /// Anchor position in screen coords (below the cursor).
    pub screen_pos: egui::Pos2,
}

/// A single reference location for the references picker.
pub struct LspReferenceItem {
    pub path: PathBuf,
    pub line: usize,
    pub character: usize,
    /// Line content (if fetched or available).
    pub text: String,
}

/// State for the find-references result picker.
pub struct LspReferencesState {
    pub items: Vec<LspReferenceItem>,
    pub selected: usize,
    pub focus_requested: bool,
    /// Flag to force scroll to the selected item (only set on keyboard nav).
    pub scroll_to_selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffAction {
    Accepted,
    Rejected,
}

pub struct EditorUiResult {
    pub clicked: bool,
    pub diff_action: Option<(String, DiffAction, String)>,
    pub tab_action: Option<TabBarAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownLayoutMode {
    PodSebou,
    VedleSebe,
    JenomKod,
    JenomNahled,
}

impl MarkdownLayoutMode {
    pub fn next(self) -> Self {
        match self {
            Self::PodSebou => Self::VedleSebe,
            Self::VedleSebe => Self::JenomKod,
            Self::JenomKod => Self::JenomNahled,
            Self::JenomNahled => Self::PodSebou,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SaveStatus {
    None,
    Modified,
    Saving,
    Saved,
}

pub struct Tab {
    pub content: String,
    pub path: PathBuf,
    pub modified: bool,
    pub(crate) deleted: bool,
    pub(crate) last_edit: Option<Instant>,
    /// Timestamp of the last *autosave* attempt (successful or failed).
    /// Used to prevent infinite retry loops on save errors.
    pub(crate) last_autosave_attempt: Option<Instant>,
    pub save_status: SaveStatus,
    pub(crate) last_saved_content: String,
    pub(crate) scroll_offset: f32,
    pub(crate) md_scroll_offset: f32,
    pub(crate) last_cursor_range: Option<egui::text::CursorRange>,
    /// Flag indicating if the file is binary (not text).
    pub is_binary: bool,
    /// For images: generated texture handle for egui.
    pub image_texture: Option<egui::TextureHandle>,
    /// Raw data for binary files (if kept in memory).
    pub binary_data: Option<Vec<u8>>,
    /// Whether the SVG modal has already been shown (true = user made a choice, don't show again).
    pub svg_modal_shown: bool,
    /// LSP document version — 0 = didOpen not yet sent, >0 = open with this version.
    pub lsp_version: i32,
    /// Last version successfully sent to the LSP server.
    pub lsp_synced_version: i32,
    /// Flag indicating that there was an error reading the file (Audit Task 1.2).
    /// If true, saving should be disabled or redirected.
    pub read_error: bool,
    /// Pre-calculated canonical path to avoid repeated filesystem calls (Audit S-5).
    pub canonical_path: PathBuf,
    /// Per-tab markdown cache to avoid re-parsing on every frame/tab switch (Audit S-1).
    pub md_cache: egui_commonmark::CommonMarkCache,
}

// ---------------------------------------------------------------------------
// Editor — main structure
// ---------------------------------------------------------------------------

pub struct Editor {
    pub tabs: Vec<Tab>,
    pub active_tab: Option<usize>,
    pub highlighter: Highlighter,
    pub(crate) show_search: bool,
    pub(crate) search_query: String,
    pub(crate) replace_query: String,
    pub(crate) show_replace: bool,
    pub(crate) search_matches: Vec<(usize, usize)>,
    pub(crate) current_match: Option<usize>,
    pub(crate) search_focus_requested: bool,
    pub(crate) md_split_ratio: f32,
    pub(crate) md_layout_mode: MarkdownLayoutMode,
    pub(crate) tab_scroll_x: f32,
    pub(crate) scroll_to_active: bool,
    /// Pending jump to (line, column) — 1-based
    pub pending_jump: Option<(usize, usize)>,
    pub(crate) show_goto_line: bool,
    pub(crate) goto_line_input: String,
    pub(crate) goto_line_focus_requested: bool,
    pub(crate) focus_editor_requested: bool,
    // --- LSP interaction state ---
    /// Pending hover request channel.
    pub lsp_hover_rx: Option<std::sync::mpsc::Receiver<Option<async_lsp::lsp_types::Hover>>>,
    /// Active hover popup (Some = visible).
    pub lsp_hover_popup: Option<LspHoverPopup>,
    /// Screen position where the hover popup should appear.
    pub lsp_hover_screen_pos: Option<egui::Pos2>,
    /// LSP position that was last hovered (for debounce comparison).
    pub lsp_hover_last_pos: Option<async_lsp::lsp_types::Position>,
    /// Time when mouse stopped moving (for debounce).
    pub lsp_hover_timer: Option<std::time::Instant>,
    /// Pending go-to-definition request channel.
    pub lsp_definition_rx:
        Option<std::sync::mpsc::Receiver<Option<async_lsp::lsp_types::GotoDefinitionResponse>>>,
    /// Pending navigation from LSP result: (file path, 1-based line, 1-based column).
    pub pending_lsp_navigate: Option<(std::path::PathBuf, usize, usize)>,
    /// Pending completion request channel.
    pub lsp_completion_rx:
        Option<std::sync::mpsc::Receiver<Option<async_lsp::lsp_types::CompletionResponse>>>,
    /// Active completion popup state.
    pub lsp_completion: Option<LspCompletionState>,
    /// Cursor screen position at the time completion was triggered.
    pub lsp_completion_cursor_pos: Option<egui::Pos2>,
    /// Pending find-references request channel.
    pub lsp_references_rx:
        Option<std::sync::mpsc::Receiver<Option<Vec<async_lsp::lsp_types::Location>>>>,
    /// Active references picker state.
    pub lsp_references: Option<LspReferencesState>,
    /// Temporary status message (text, timestamp).
    pub lsp_status: Option<(String, std::time::Instant)>,
    /// Pending AI diff for approval: (file_path, original_content, new_content).
    pub pending_ai_diff: Option<(String, String, String)>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            highlighter: Highlighter::new(),
            show_search: false,
            search_query: String::new(),
            replace_query: String::new(),
            show_replace: false,
            search_matches: Vec::new(),
            current_match: None,
            search_focus_requested: false,
            md_split_ratio: 0.5,
            md_layout_mode: MarkdownLayoutMode::PodSebou,
            tab_scroll_x: 0.0,
            scroll_to_active: false,
            pending_jump: None,
            show_goto_line: false,
            goto_line_input: String::new(),
            goto_line_focus_requested: false,
            focus_editor_requested: false,
            lsp_hover_rx: None,
            lsp_hover_popup: None,
            lsp_hover_screen_pos: None,
            lsp_hover_last_pos: None,
            lsp_hover_timer: None,
            lsp_definition_rx: None,
            pending_lsp_navigate: None,
            lsp_completion_rx: None,
            lsp_completion: None,
            lsp_completion_cursor_pos: None,
            lsp_references_rx: None,
            lsp_references: None,
            lsp_status: None,
            pending_ai_diff: None,
        }
    }

    // --- Helpers ---

    pub(crate) fn active(&self) -> Option<&Tab> {
        self.active_tab.and_then(|i| self.tabs.get(i))
    }

    pub(crate) fn active_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab.and_then(|i| self.tabs.get_mut(i))
    }

    pub fn active_path(&self) -> Option<&PathBuf> {
        self.active().map(|t| &t.path)
    }

    pub fn extension(&self) -> String {
        self.active()
            .and_then(|t| t.path.extension())
            .map(|e| e.to_string_lossy().to_string())
            .or_else(|| {
                let name = self.filename();
                if name.starts_with('.') && name.len() > 1 {
                    Some(name[1..].to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    pub fn filename(&self) -> String {
        self.active()
            .and_then(|t| t.path.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    pub fn is_markdown(&self) -> bool {
        let ext = self.extension();
        ext == "md" || ext == "markdown"
    }

    pub fn is_svg(&self) -> bool {
        self.extension() == "svg"
    }
}

// ---------------------------------------------------------------------------
// Helper free functions (keep here for internal use)
// ---------------------------------------------------------------------------

pub(crate) fn lang_id_from_path(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => "rust",
        Some("php") => "php",
        Some("js") | Some("mjs") => "javascript",
        Some("ts") => "typescript",
        Some("py") => "python",
        Some("html") | Some("htm") => "html",
        Some("css") => "css",
        Some("json") => "json",
        Some("md") => "markdown",
        _ => "plaintext",
    }
}

pub fn ext_to_file_type(ext: &str) -> &'static str {
    match ext {
        "rs" => "Rust",
        "php" => "PHP",
        "js" | "mjs" => "JavaScript",
        "ts" => "TypeScript",
        "tsx" => "TSX",
        "jsx" => "JSX",
        "md" | "markdown" => "Markdown",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" => "SCSS",
        "json" => "JSON",
        "toml" => "TOML",
        "yaml" | "yml" => "YAML",
        "sh" | "bash" => "Shell",
        "py" => "Python",
        "c" => "C",
        "cpp" | "h" | "hpp" => "C++",
        "txt" => "Text",
        _ => "File",
    }
}
