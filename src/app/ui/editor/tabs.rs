use super::*;
use std::path::PathBuf;

impl Editor {
    // --- Tab management ---

    pub fn open_file(&mut self, path: &PathBuf) {
        if let Some(idx) = self.tabs.iter().position(|t| t.path == *path) {
            self.active_tab = Some(idx);
            self.focus_editor_requested = true;
            self.update_search();
            return;
        }

        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());

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
                        last_autosave_attempt: None,
                        save_status: SaveStatus::None,
                        scroll_offset: 0.0,
                        md_scroll_offset: 0.0,
                        last_cursor_range: None,
                        is_binary: true,
                        image_texture: None,
                        binary_data: Some(bytes),
                        svg_modal_shown: false,
                        lsp_version: 0,
                        lsp_synced_version: 0,
                        read_error: false,
                        canonical_path: canonical_path.clone(),
                        md_cache: egui_commonmark::CommonMarkCache::default(),
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
                        last_autosave_attempt: None,
                        save_status: SaveStatus::None,
                        scroll_offset: 0.0,
                        md_scroll_offset: 0.0,
                        last_cursor_range: None,
                        is_binary: false,
                        image_texture: None,
                        binary_data: None,
                        svg_modal_shown: false,
                        lsp_version: 0,
                        lsp_synced_version: 0,
                        read_error: true,
                        canonical_path: canonical_path.clone(),
                        md_cache: egui_commonmark::CommonMarkCache::default(),
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
                        last_autosave_attempt: None,
                        save_status: SaveStatus::None,
                        scroll_offset: 0.0,
                        md_scroll_offset: 0.0,
                        last_cursor_range: None,
                        is_binary: false,
                        image_texture: None,
                        binary_data: None,
                        svg_modal_shown: false,
                        lsp_version: 0,
                        lsp_synced_version: 0,
                        read_error: false,
                        canonical_path: canonical_path.clone(),
                        md_cache: egui_commonmark::CommonMarkCache::default(),
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
                        last_autosave_attempt: None,
                        save_status: SaveStatus::None,
                        scroll_offset: 0.0,
                        md_scroll_offset: 0.0,
                        last_cursor_range: None,
                        is_binary: false,
                        image_texture: None,
                        binary_data: None,
                        svg_modal_shown: false,
                        lsp_version: 0,
                        lsp_synced_version: 0,
                        read_error: true,
                        canonical_path: canonical_path.clone(),
                        md_cache: egui_commonmark::CommonMarkCache::default(),
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
        self.pending_jump = Some((line.max(1), 1));
    }

    pub fn jump_to_location(&mut self, line: usize, column: usize) {
        self.pending_jump = Some((line.max(1), column.max(1)));
    }

    pub fn request_editor_focus(&mut self) {
        self.focus_editor_requested = true;
    }
}
