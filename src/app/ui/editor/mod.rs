mod markdown;
mod search;
use std::path::PathBuf;
use std::time::Instant;

use eframe::egui;

use crate::highlighter::Highlighter;

mod files;
mod render_binary;
mod render_context;
mod render_helpers;
use render_helpers::*;
pub mod diff_view;
mod render_lsp;
mod render_markdown;
mod render_normal;
mod render_tabs;
mod tabs;
mod ui;

const AUTOSAVE_DELAY_MS: u128 = 500;
/// Hover debounce: wait this long after mouse stops before sending request.
const LSP_HOVER_DEBOUNCE_MS: u128 = 600;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Content of an active LSP hover popup.
pub(super) struct LspHoverPopup {
    pub content: String,
    pub screen_pos: egui::Pos2,
}

/// State of the active LSP completion dropdown.
pub(super) struct LspCompletionState {
    pub items: Vec<async_lsp::lsp_types::CompletionItem>,
    pub selected: usize,
    /// Anchor position in screen coords (below the cursor).
    pub screen_pos: egui::Pos2,
}

/// A single reference location for the references picker.
pub(super) struct LspReferenceItem {
    pub path: PathBuf,
    pub line: usize,
    pub character: usize,
    /// Line content (if fetched or available).
    pub text: String,
}

/// State for the find-references result picker.
pub(super) struct LspReferencesState {
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
}

#[derive(PartialEq)]
pub enum SaveStatus {
    None,
    Modified,
    Saving,
    Saved,
}

pub(super) struct Tab {
    pub(crate) content: String,
    pub(super) path: PathBuf,
    pub(crate) modified: bool,
    pub(super) deleted: bool,
    pub(super) last_edit: Option<Instant>,
    /// Timestamp of the last *autosave* attempt (successful or failed).
    /// Used to prevent infinite retry loops on save errors.
    pub(super) last_autosave_attempt: Option<Instant>,
    pub(crate) save_status: SaveStatus,
    pub(super) last_saved_content: String,
    scroll_offset: f32,
    md_scroll_offset: f32,
    last_cursor_range: Option<egui::text::CursorRange>,
    /// Flag indicating if the file is binary (not text).
    pub(super) is_binary: bool,
    /// For images: generated texture handle for egui.
    pub(super) image_texture: Option<egui::TextureHandle>,
    /// Raw data for binary files (if kept in memory).
    pub(super) binary_data: Option<Vec<u8>>,
    /// Whether the SVG modal has already been shown (true = user made a choice, don't show again).
    pub(super) svg_modal_shown: bool,
    /// LSP document version — 0 = didOpen not yet sent, >0 = open with this version.
    pub(super) lsp_version: i32,
    /// Last version successfully sent to the LSP server.
    pub(super) lsp_synced_version: i32,
    /// Flag indicating that there was an error reading the file (Audit Task 1.2).
    /// If true, saving should be disabled or redirected.
    pub(super) read_error: bool,
    /// Pre-calculated canonical path to avoid repeated filesystem calls (Audit S-5).
    pub(super) canonical_path: PathBuf,
    /// Per-tab markdown cache to avoid re-parsing on every frame/tab switch (Audit S-1).
    pub(super) md_cache: egui_commonmark::CommonMarkCache,
}

// ---------------------------------------------------------------------------
// Editor — main structure
// ---------------------------------------------------------------------------

pub struct Editor {
    pub(super) tabs: Vec<Tab>,
    pub(super) active_tab: Option<usize>,
    pub(super) highlighter: Highlighter,
    pub(super) show_search: bool,
    pub(super) search_query: String,
    pub(super) replace_query: String,
    pub(super) show_replace: bool,
    pub(super) search_matches: Vec<(usize, usize)>,
    pub(super) current_match: Option<usize>,
    pub(super) search_focus_requested: bool,
    pub(super) md_split_ratio: f32,
    pub(super) tab_scroll_x: f32,
    pub(super) scroll_to_active: bool,
    /// Pending jump to (line, column) — 1-based
    pub(super) pending_jump: Option<(usize, usize)>,
    pub(super) show_goto_line: bool,
    pub(super) goto_line_input: String,
    pub(super) goto_line_focus_requested: bool,
    pub(super) focus_editor_requested: bool,
    // --- LSP interaction state ---
    /// Pending hover request channel.
    pub(super) lsp_hover_rx: Option<std::sync::mpsc::Receiver<Option<async_lsp::lsp_types::Hover>>>,
    /// Active hover popup (Some = visible).
    pub(super) lsp_hover_popup: Option<LspHoverPopup>,
    /// Screen position where the hover popup should appear.
    pub(super) lsp_hover_screen_pos: Option<egui::Pos2>,
    /// LSP position that was last hovered (for debounce comparison).
    pub(super) lsp_hover_last_pos: Option<async_lsp::lsp_types::Position>,
    /// Time when mouse stopped moving (for debounce).
    pub(super) lsp_hover_timer: Option<std::time::Instant>,
    /// Pending go-to-definition request channel.
    pub(super) lsp_definition_rx:
        Option<std::sync::mpsc::Receiver<Option<async_lsp::lsp_types::GotoDefinitionResponse>>>,
    /// Pending navigation from LSP result: (file path, 1-based line, 1-based column).
    pub pending_lsp_navigate: Option<(std::path::PathBuf, usize, usize)>,
    /// Pending completion request channel.
    pub(super) lsp_completion_rx:
        Option<std::sync::mpsc::Receiver<Option<async_lsp::lsp_types::CompletionResponse>>>,
    /// Active completion popup state.
    pub(super) lsp_completion: Option<LspCompletionState>,
    /// Cursor screen position at the time completion was triggered.
    pub(super) lsp_completion_cursor_pos: Option<egui::Pos2>,
    /// Pending find-references request channel.
    pub(super) lsp_references_rx:
        Option<std::sync::mpsc::Receiver<Option<Vec<async_lsp::lsp_types::Location>>>>,
    /// Active references picker state.
    pub(super) lsp_references: Option<LspReferencesState>,
    /// Temporary status message (text, timestamp).
    pub(super) lsp_status: Option<(String, std::time::Instant)>,
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

    fn active(&self) -> Option<&Tab> {
        self.active_tab.and_then(|i| self.tabs.get(i))
    }

    fn active_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab.and_then(|i| self.tabs.get_mut(i))
    }

    pub fn active_path(&self) -> Option<&PathBuf> {
        self.active().map(|t| &t.path)
    }

    pub(super) fn extension(&self) -> String {
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

    pub(super) fn filename(&self) -> String {
        self.active()
            .and_then(|t| t.path.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    pub(super) fn is_markdown(&self) -> bool {
        let ext = self.extension();
        ext == "md" || ext == "markdown"
    }

    pub(super) fn is_svg(&self) -> bool {
        self.extension() == "svg"
    }
}

// ---------------------------------------------------------------------------
// Helper free functions
// ---------------------------------------------------------------------------

fn lang_id_from_path(path: &std::path::Path) -> &'static str {
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

pub(super) fn ext_to_file_type(ext: &str) -> &'static str {
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
