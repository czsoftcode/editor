use std::path::PathBuf;
use std::time::Instant;

use eframe::egui;

use crate::app::ui::widgets::tab_bar::TabBarAction;
use crate::highlighter::Highlighter;

mod markdown;
mod render;
mod search;

const AUTOSAVE_DELAY_MS: u128 = 500;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(PartialEq)]
pub enum SaveStatus {
    None,
    Modified,
    Saving,
    Saved,
}

pub(super) struct Tab {
    content: String,
    path: PathBuf,
    modified: bool,
    deleted: bool,
    last_edit: Option<Instant>,
    save_status: SaveStatus,
    last_saved_content: String,
    scroll_offset: f32,
    last_cursor_range: Option<egui::text::CursorRange>,
    /// Flag indicating if the file is binary (not text).
    pub(super) is_binary: bool,
    /// For images: generated texture handle for egui.
    pub(super) image_texture: Option<egui::TextureHandle>,
    /// Raw data for binary files (if kept in memory).
    pub(super) binary_data: Option<Vec<u8>>,
    /// Whether the SVG modal has already been shown (true = user made a choice, don't show again).
    pub(super) svg_modal_shown: bool,
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
    /// Pending jump to line (1-based)
    pub(super) pending_jump: Option<usize>,
    pub(super) show_goto_line: bool,
    pub(super) goto_line_input: String,
    pub(super) goto_line_focus_requested: bool,
    pub(super) focus_editor_requested: bool,
    pub(super) markdown_cache: egui_commonmark::CommonMarkCache,
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
            markdown_cache: egui_commonmark::CommonMarkCache::default(),
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

    // --- Tab management ---

    pub fn open_file(&mut self, path: &PathBuf) {
        if let Some(idx) = self.tabs.iter().position(|t| t.path == *path) {
            self.active_tab = Some(idx);
            self.focus_editor_requested = true;
            self.update_search();
            return;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_lowercase();
        // Files opened in a system application — no tab
        let is_external = matches!(ext.as_str(), "pdf" | "odt" | "docx");
        if is_external {
            let _ = std::process::Command::new("xdg-open").arg(path).spawn();
            return;
        }

        let is_image = matches!(
            ext.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico"
        );

        if is_image {
            match std::fs::read(path) {
                Ok(bytes) => {
                    let tab = Tab {
                        content: String::new(),
                        last_saved_content: String::new(),
                        path: path.clone(),
                        modified: false,
                        deleted: false,
                        last_edit: None,
                        save_status: SaveStatus::None,
                        scroll_offset: 0.0,
                        last_cursor_range: None,
                        is_binary: true,
                        image_texture: None,
                        binary_data: Some(bytes),
                        svg_modal_shown: false,
                    };
                    self.tabs.push(tab);
                    self.active_tab = Some(self.tabs.len() - 1);
                    self.focus_editor_requested = true;
                }
                Err(e) => {
                    let tab = Tab {
                        content: format!("Error reading binary file: {}", e),
                        last_saved_content: String::new(),
                        path: path.clone(),
                        modified: false,
                        deleted: false,
                        last_edit: None,
                        save_status: SaveStatus::None,
                        scroll_offset: 0.0,
                        last_cursor_range: None,
                        is_binary: false,
                        image_texture: None,
                        binary_data: None,
                        svg_modal_shown: false,
                    };
                    self.tabs.push(tab);
                    self.active_tab = Some(self.tabs.len() - 1);
                    self.focus_editor_requested = true;
                }
            }
        } else {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let tab = Tab {
                        last_saved_content: content.clone(),
                        content,
                        path: path.clone(),
                        modified: false,
                        deleted: false,
                        last_edit: None,
                        save_status: SaveStatus::None,
                        scroll_offset: 0.0,
                        last_cursor_range: None,
                        is_binary: false,
                        image_texture: None,
                        binary_data: None,
                        svg_modal_shown: false,
                    };
                    self.tabs.push(tab);
                    self.active_tab = Some(self.tabs.len() - 1);
                    self.focus_editor_requested = true;
                }
                Err(e) => {
                    let tab = Tab {
                        content: format!("Error reading file: {}", e),
                        last_saved_content: String::new(),
                        path: path.clone(),
                        modified: false,
                        deleted: false,
                        last_edit: None,
                        save_status: SaveStatus::None,
                        scroll_offset: 0.0,
                        last_cursor_range: None,
                        is_binary: false,
                        image_texture: None,
                        binary_data: None,
                        svg_modal_shown: false,
                    };
                    self.tabs.push(tab);
                    self.active_tab = Some(self.tabs.len() - 1);
                    self.focus_editor_requested = true;
                }
            }
        }
        self.update_search();
        self.scroll_to_active = true;
    }

    pub fn close_tab(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active_tab = None;
        } else if let Some(active) = self.active_tab {
            if active == index {
                self.active_tab = Some(active.min(self.tabs.len() - 1));
            } else if active > index {
                self.active_tab = Some(active - 1);
            }
        }
        self.update_search();
    }

    pub fn clear(&mut self) {
        if let Some(idx) = self.active_tab {
            self.close_tab(idx);
        }
    }

    pub fn close_tabs_for_path(&mut self, path: &PathBuf) {
        let indices: Vec<usize> = self
            .tabs
            .iter()
            .enumerate()
            .filter(|(_, t)| t.path == *path || t.path.starts_with(path))
            .map(|(i, _)| i)
            .collect();
        for idx in indices.into_iter().rev() {
            self.close_tab(idx);
        }
    }

    pub fn notify_file_deleted(&mut self, path: &PathBuf) {
        for tab in &mut self.tabs {
            if tab.path == *path {
                tab.deleted = true;
                tab.modified = false;
                tab.last_edit = None;
                tab.save_status = SaveStatus::None;
            }
        }
    }

    pub fn jump_to_line(&mut self, line: usize) {
        self.pending_jump = Some(line.max(1));
    }

    pub fn request_editor_focus(&mut self) {
        self.focus_editor_requested = true;
    }

    // --- File operations ---

    /// Returns true if a tab for the given path exists and has unsaved changes.
    pub fn is_path_modified(&self, path: &PathBuf) -> bool {
        self.tabs.iter().any(|t| t.path == *path && t.modified)
    }

    /// Finds the tab path whose canonicalized path matches `canonical`.
    /// Returns the original (non-canonicalized) tab path if it exists.
    pub fn tab_path_for_canonical(&self, canonical: &PathBuf) -> Option<PathBuf> {
        self.tabs.iter().find_map(|t| {
            t.path
                .canonicalize()
                .ok()
                .filter(|c| c == canonical)
                .map(|_| t.path.clone())
        })
    }

    /// Reloads a specific tab (by path) from disk — regardless of the active tab.
    pub fn reload_path_from_disk(&mut self, path: &PathBuf) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.path == *path) {
            if tab.is_binary {
                if let Ok(bytes) = std::fs::read(&tab.path) {
                    tab.binary_data = Some(bytes);
                    tab.image_texture = None;
                    tab.modified = false;
                    tab.last_edit = None;
                    tab.save_status = SaveStatus::Saved;
                }
            } else {
                if let Ok(content) = std::fs::read_to_string(&tab.path) {
                    tab.content = content.clone();
                    tab.last_saved_content = content;
                    tab.modified = false;
                    tab.last_edit = None;
                    tab.save_status = SaveStatus::Saved;
                }
            }
        }
        self.update_search();
    }

    /// Attempts to autosave the active tab. Returns an error message if writing fails.
    pub fn try_autosave(&mut self, i18n: &crate::i18n::I18n) -> Option<String> {
        let should_save = self.active().is_some_and(|t| {
            !t.deleted
                && t.modified
                && t.last_edit
                    .is_some_and(|e| e.elapsed().as_millis() >= AUTOSAVE_DELAY_MS)
        });
        if should_save { self.save(i18n) } else { None }
    }

    /// Saves the active tab. Returns an error message if writing fails, otherwise None.
    pub fn save(&mut self, i18n: &crate::i18n::I18n) -> Option<String> {
        let tab = self.active_mut()?;
        tab.save_status = SaveStatus::Saving;
        let res = if tab.is_binary {
            if let Some(bytes) = &tab.binary_data {
                std::fs::write(&tab.path, bytes)
            } else {
                Ok(())
            }
        } else {
            std::fs::write(&tab.path, &tab.content)
        };

        match res {
            Ok(()) => {
                if !tab.is_binary {
                    tab.last_saved_content = tab.content.clone();
                }
                tab.modified = false;
                tab.last_edit = None;
                tab.save_status = SaveStatus::Saved;
                None
            }
            Err(e) => {
                tab.save_status = SaveStatus::Modified;
                let name = tab.path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("name", name);
                args.set("reason", e.to_string());
                Some(i18n.get_args("error-file-save", &args))
            }
        }
    }

    /// Saves a specific tab identified by path (regardless of the active tab).
    /// Returns an error message if writing fails, otherwise None.
    pub fn save_path(&mut self, path: &PathBuf, i18n: &crate::i18n::I18n) -> Option<String> {
        let tab = self.tabs.iter_mut().find(|t| t.path == *path)?;
        tab.save_status = SaveStatus::Saving;
        let res = if tab.is_binary {
            if let Some(bytes) = &tab.binary_data {
                std::fs::write(&tab.path, bytes)
            } else {
                Ok(())
            }
        } else {
            std::fs::write(&tab.path, &tab.content)
        };

        match res {
            Ok(()) => {
                if !tab.is_binary {
                    tab.last_saved_content = tab.content.clone();
                }
                tab.modified = false;
                tab.last_edit = None;
                tab.save_status = SaveStatus::Saved;
                None
            }
            Err(e) => {
                tab.save_status = SaveStatus::Modified;
                let name = tab.path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("name", name);
                args.set("reason", e.to_string());
                Some(i18n.get_args("error-file-save", &args))
            }
        }
    }

    // --- UI entry point ---

    /// Returns `true` if the user clicked in the editor.
    pub fn ui(&mut self, ui: &mut egui::Ui, dialog_open: bool, i18n: &crate::i18n::I18n) -> bool {
        if self.tabs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(i18n.get("editor-empty-hint"));
            });
            return false;
        }

        let mut tab_action = None;
        self.tab_bar(ui, &mut tab_action);
        match tab_action {
            Some(TabBarAction::Switch(idx)) => {
                self.active_tab = Some(idx);
                self.focus_editor_requested = true;
                self.update_search();
            }
            Some(TabBarAction::Close(idx)) => {
                self.close_tab(idx);
            }
            Some(TabBarAction::New) | None => {}
        }

        if self.tabs.is_empty() {
            return false;
        }

        let ctrl_f = ui
            .ctx()
            .input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::F));
        let ctrl_h = ui
            .ctx()
            .input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::H));
        let ctrl_g = ui
            .ctx()
            .input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::G));
        let escape = ui.ctx().input(|i| i.key_pressed(egui::Key::Escape));

        if ctrl_f {
            self.show_search = true;
            self.show_replace = false;
            self.search_focus_requested = true;
            self.show_goto_line = false;
            self.goto_line_focus_requested = false;
            self.update_search();
        }
        if ctrl_h {
            self.show_search = true;
            self.show_replace = true;
            self.search_focus_requested = true;
            self.show_goto_line = false;
            self.goto_line_focus_requested = false;
            self.update_search();
        }
        if ctrl_g {
            self.show_goto_line = !self.show_goto_line;
            if self.show_goto_line {
                self.goto_line_input.clear();
                self.show_search = false;
                self.goto_line_focus_requested = true;
            } else {
                self.goto_line_focus_requested = false;
            }
        }
        if escape {
            if self.show_search {
                self.show_search = false;
                self.show_replace = false;
                self.search_matches.clear();
                self.current_match = None;
            } else if self.show_goto_line {
                self.show_goto_line = false;
                self.goto_line_focus_requested = false;
            }
        }

        if self.show_search {
            self.search_bar(ui, i18n);
        }
        if self.show_goto_line {
            self.goto_line_bar(ui, i18n);
        }

        let is_binary = self.active().is_some_and(|t| t.is_binary);

        if is_binary {
            self.ui_binary(ui, i18n)
        } else if self.is_markdown() {
            self.ui_markdown_split(ui, dialog_open, i18n)
        } else {
            if self.is_svg() {
                if let Some(idx) = self.active_tab {
                    if !self.tabs[idx].svg_modal_shown {
                        let path = self.tabs[idx].path.clone();
                        let fname = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();

                        let mut open_external = false;
                        let mut edit_as_text = false;

                        let modal = egui::Modal::new(egui::Id::new(("svg_modal", &path)));
                        modal.show(ui.ctx(), |ui| {
                            ui.heading(i18n.get("svg-modal-title"));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(&fname).strong());
                            ui.add_space(8.0);
                            ui.label(i18n.get("svg-modal-body"));
                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if ui.button(i18n.get("svg-open-external")).clicked() {
                                    open_external = true;
                                }
                                if ui.button(i18n.get("svg-modal-edit")).clicked() {
                                    edit_as_text = true;
                                }
                            });
                        });

                        if open_external {
                            let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                            self.tabs[idx].svg_modal_shown = true;
                        }
                        if edit_as_text {
                            self.tabs[idx].svg_modal_shown = true;
                        }
                    }
                }

                // Button always visible — user can use it anytime during editing
                if let Some(path) = self.active_path().cloned() {
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(i18n.get("svg-open-external")).clicked() {
                                let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                            }
                        });
                    });
                    ui.separator();
                }
            }
            self.ui_normal(ui, dialog_open, i18n)
        }
    }

    pub fn status_bar(&self, ui: &mut egui::Ui, git_branch: Option<&str>, i18n: &crate::i18n::I18n) {
        let tab = match self.active() {
            Some(t) => t,
            None => return,
        };
        let primary_color = egui::Color32::from_rgb(235, 240, 248);
        let secondary_color = egui::Color32::from_rgb(195, 205, 220);
        let status_warn_color = egui::Color32::from_rgb(255, 200, 120);
        let status_ok_color = egui::Color32::from_rgb(170, 230, 185);

        let cursor_text = tab.last_cursor_range.map(|cr| {
            let rc = cr.primary.rcursor;
            format!("{}:{}", rc.row + 1, rc.column + 1)
        });

        let file_type = ext_to_file_type(&self.extension());

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(tab.path.to_string_lossy().to_string()).color(primary_color),
            );
            ui.separator();
            match tab.save_status {
                SaveStatus::None => {}
                SaveStatus::Modified => {
                    ui.label(egui::RichText::new(i18n.get("statusbar-unsaved")).color(status_warn_color));
                }
                SaveStatus::Saving => {
                    ui.label(egui::RichText::new(i18n.get("statusbar-saving")).color(secondary_color));
                }
                SaveStatus::Saved => {
                    ui.label(egui::RichText::new(i18n.get("statusbar-saved")).color(status_ok_color));
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(pos) = cursor_text {
                    ui.label(egui::RichText::new(pos).monospace().color(primary_color));
                    ui.separator();
                }
                if let Some(branch) = git_branch {
                    ui.label(
                        egui::RichText::new(format!("\u{2387} {}", branch)).color(status_ok_color),
                    );
                    ui.separator();
                }
                ui.label(egui::RichText::new(i18n.get("statusbar-encoding")).color(secondary_color));
                ui.separator();
                ui.label(egui::RichText::new(file_type).color(secondary_color));
            });
        });
        ui.separator();
    }
}

// ---------------------------------------------------------------------------
// Helper free functions
// ---------------------------------------------------------------------------

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
        "cpp" | "cc" | "cxx" => "C++",
        "h" | "hpp" => "C/C++ Header",
        "go" => "Go",
        "java" => "Java",
        "xml" => "XML",
        "sql" => "SQL",
        "txt" => "Text",
        "pdf" => "PDF Document",
        "odt" => "ODT Document",
        "docx" => "DOCX Document",
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "svg" => "Image",
        _ => "Plain text",
    }
}
