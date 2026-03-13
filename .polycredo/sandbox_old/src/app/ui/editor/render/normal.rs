use super::*;
use crate::app::ui::editor::search::apply_search_highlights;
use crate::app::ui::editor::*;
use eframe::egui;
use std::time::Instant;

impl Editor {
    // --- Normal editor ---

    #[allow(clippy::too_many_arguments)]
    pub fn ui_normal(
        &mut self,
        ui: &mut egui::Ui,
        dialog_open: bool,
        i18n: &crate::i18n::I18n,
        diagnostics_for_file: Option<&Vec<async_lsp::lsp_types::Diagnostic>>,
        lsp_client: Option<&crate::app::lsp::LspClient>,
        is_readonly: bool,
        theme_name: &str,
    ) -> bool {
        let idx = match self.active_tab {
            Some(i) => i,
            None => return false,
        };

        let bg = self.highlighter.background_color(theme_name);
        let ext = self.extension();
        let fname = self.filename();
        let current_match = self.current_match;
        let tab_path = self.tabs[idx].path.clone();
        let current_scroll_y = self.tabs[idx].scroll_offset;

        let edit_id = egui::Id::new("editor_text").with(&tab_path);
        let line_count_for_jump = self
            .tabs
            .get(idx)
            .map(|tab| editor_line_count(&tab.content))
            .unwrap_or(1);

        let jump_line_col = self.pending_jump.take();
        let jump_char_idx: Option<usize> = jump_line_col.and_then(|(line, col)| {
            self.tabs.get(idx).map(|tab| {
                let mut char_count = 0;
                for (i, l) in tab.content.lines().enumerate() {
                    if i + 1 == line {
                        // Add column offset (1-based to 0-based)
                        char_count += col.saturating_sub(1);
                        break;
                    }
                    char_count += l.chars().count() + 1; // +1 for newline
                }
                char_count
            })
        });
        let has_jump = jump_char_idx.is_some();

        let font_size = Self::current_editor_font_size(ui);
        let font_id = egui::FontId::monospace(font_size);
        let row_height = ui.fonts(|f| f.row_height(&font_id));
        let viewport_height = (ui.available_height() - 16.0).max(row_height);
        let desired_scroll_y: Option<f32> = jump_line_col.map(|(line, _)| {
            goto_centered_scroll_offset(line, line_count_for_jump, row_height, viewport_height)
        });

        let should_request_editor_focus = self.focus_editor_requested
            && !dialog_open
            && !self.show_search
            && !self.show_goto_line;

        let mut clicked = false;
        let mut saved_response: Option<egui::text_edit::TextEditOutput> = None;
        let mut content_changed = false;
        let mut updated_scroll_y: Option<f32> = None;

        let frame = egui::Frame::new()
            .fill(bg)
            .inner_margin(egui::Margin::same(8));

        frame.show(ui, |ui| {
            if !is_readonly {
                self.handle_smart_typing(ui, edit_id, idx);
            }
            let scroll_y = desired_scroll_y.unwrap_or(current_scroll_y);
            let scroll_output = egui::ScrollArea::both()
                .id_salt(("editor_scroll", &tab_path))
                .auto_shrink([false, false])
                .vertical_scroll_offset(scroll_y)
                .show(ui, |ui| {
                    let highlighter = &self.highlighter;
                    let search_matches = &self.search_matches;
                    let tab = &mut self.tabs[idx];

                    // TRICK: If readonly, we pass a temporary copy of the string to TextEdit.
                    // This way it remains interactive (cursor, selection, shortcuts for movement),
                    // but changes are never saved back to the tab.
                    let mut temp_content;
                    let content_to_edit = if is_readonly {
                        temp_content = tab.content.clone();
                        &mut temp_content
                    } else {
                        &mut tab.content
                    };

                    let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                        let job_arc = highlighter.highlight(
                            text,
                            &ext,
                            &fname,
                            Self::current_editor_font_size(ui),
                            theme_name,
                        );
                        // We clone the job from the Arc to apply dynamic overlays like wrap width and search.
                        // This is still much faster than re-parsing the whole file.
                        let mut job = (*job_arc).clone();
                        job.wrap.max_width = wrap_width;
                        apply_search_highlights(&mut job, search_matches, current_match);
                        ui.fonts(|f| f.layout_job(job))
                    };

                    let line_count = editor_line_count(content_to_edit);
                    let gutter_width = Self::gutter_width(ui, line_count);

                    ui.horizontal_top(|ui| {
                        let (gutter_rect, _) = ui.allocate_exact_size(
                            egui::vec2(gutter_width, ui.available_height()),
                            egui::Sense::hover(),
                        );

                        let response = egui::TextEdit::multiline(content_to_edit)
                            .id(edit_id)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .interactive(true)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter)
                            .show(ui);

                        // --- FORCED JUMP AND FOCUS ---
                        if let Some(char_idx) = jump_char_idx {
                            let mut state = egui::text_edit::TextEditState::load(ui.ctx(), edit_id)
                                .unwrap_or_default();
                            state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::one(
                                    egui::text::CCursor::new(char_idx),
                                )));
                            state.store(ui.ctx(), edit_id);
                            response.response.request_focus();
                        }

                        Self::paint_line_numbers(ui, &response, gutter_rect, diagnostics_for_file);
                        Self::paint_squiggles(ui, &response, diagnostics_for_file);

                        if dialog_open {
                            if response.response.clicked() {
                                clicked = true;
                            }
                        } else if response.response.clicked() || response.response.has_focus() {
                            clicked = true;
                        }
                        if response.response.changed() && !is_readonly {
                            tab.modified = true;
                            tab.last_edit = Some(Instant::now());
                            tab.save_status = SaveStatus::Modified;
                            tab.lsp_version += 1;
                            content_changed = true;
                        }
                        saved_response = Some(response);
                    });
                });
            updated_scroll_y = Some(scroll_output.state.offset.y);
        });
        if let Some(scroll_y) = updated_scroll_y {
            self.tabs[idx].scroll_offset = scroll_y;
        }

        if should_request_editor_focus {
            if !has_jump {
                restore_saved_cursor(ui.ctx(), edit_id, self.tabs[idx].last_cursor_range);
            }
            ui.memory_mut(|m| m.request_focus(edit_id));
            self.focus_editor_requested = false;
        }

        if let Some(lsp) = lsp_client {
            let tab = &mut self.tabs[idx];
            // Only send didChange if didOpen was already sent (lsp_version > 0),
            // and if there are unsynced changes that have aged enough (debounce).
            let needs_sync = tab.lsp_version > tab.lsp_synced_version;
            let debounce_passed = tab.last_edit.is_none_or(|e| e.elapsed().as_millis() >= 500);

            if tab.lsp_version > 0
                && needs_sync
                && debounce_passed
                && let Ok(uri) = async_lsp::lsp_types::Url::from_file_path(&tab.path)
            {
                tab.lsp_synced_version = tab.lsp_version;
                lsp.notify_did_change(uri, tab.lsp_version, tab.content.clone());
            }
        }

        if let Some(response) = &saved_response {
            self.show_editor_context_menu(response, i18n);
        }
        if content_changed && self.show_search {
            self.update_search();
        }

        // -----------------------------------------------------------------------
        // LSP interactions — hover, go-to-definition, completion
        // (runs after frame.show so we can re-borrow self freely)
        // -----------------------------------------------------------------------
        self.process_lsp_interactions(ui, idx, &tab_path, lsp_client, &saved_response, i18n);

        clicked
    }
}
