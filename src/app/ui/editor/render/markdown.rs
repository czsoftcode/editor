use super::*;
use crate::app::ui::editor::search::apply_search_highlights;
use crate::app::ui::editor::*;
use eframe::egui;
use std::time::Instant;

impl Editor {
    // --- Markdown split view ---

    pub fn ui_markdown_split(
        &mut self,
        ui: &mut egui::Ui,
        dialog_open: bool,
        i18n: &crate::i18n::I18n,
        diagnostics_for_file: Option<&Vec<async_lsp::lsp_types::Diagnostic>>,
        lsp_client: Option<&crate::app::lsp::LspClient>,
        is_readonly: bool,
    ) -> bool {
        let idx = match self.active_tab {
            Some(i) => i,
            None => return false,
        };

        let bg = self.highlighter.background_color();
        let ext = self.extension();
        let fname = self.filename();
        let current_match = self.current_match;
        let tab_path = self.tabs[idx].path.clone();
        let edit_id = egui::Id::new("editor_text").with(&tab_path);

        let prev_editor_scroll = self.tabs[idx].scroll_offset;
        let prev_preview_scroll = self.tabs[idx].md_scroll_offset;

        let should_request_editor_focus = self.focus_editor_requested
            && !dialog_open
            && !self.show_search
            && !self.show_goto_line;
        if should_request_editor_focus {
            restore_saved_cursor(ui.ctx(), edit_id, self.tabs[idx].last_cursor_range);
            ui.memory_mut(|m| m.request_focus(edit_id));
            self.focus_editor_requested = false;
        }

        let mut clicked = false;
        let mut saved_response: Option<egui::text_edit::TextEditOutput> = None;
        let mut content_changed = false;

        // Button to open in an external viewer
        if let Some(path) = self.active_path().cloned() {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(i18n.get("md-open-external")).clicked() {
                        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                    }
                });
            });
            ui.separator();
        }

        let available = ui.available_size();
        let handle_h = 6.0_f32;
        let top_h = (available.y * self.md_split_ratio)
            .max(50.0)
            .min(available.y - handle_h - 50.0);
        let bottom_h = (available.y - top_h - handle_h).max(50.0);

        let mut editor_scroll_pct = 0.0;
        let mut editor_max_scroll = 0.0;

        // Top half: Editor
        ui.allocate_ui_with_layout(
            egui::vec2(available.x, top_h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.label(egui::RichText::new("Editor").strong());
                ui.separator();

                let frame = egui::Frame::new()
                    .fill(bg)
                    .inner_margin(egui::Margin::same(8));

                frame.show(ui, |ui| {
                    if !is_readonly {
                        self.handle_smart_typing(ui, edit_id, idx);
                    }
                    let scroll_y = self.tabs[idx].scroll_offset;

                    let scroll_output = egui::ScrollArea::both()
                        .id_salt(("md_editor_scroll", &tab_path))
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
                                );
                                // Cloned for dynamic overlays (wrap, search).
                                let mut job = (*job_arc).clone();
                                job.wrap.max_width = wrap_width;
                                apply_search_highlights(&mut job, search_matches, current_match);
                                ui.fonts(|f| f.layout_job(job))
                            };

                            let line_count = content_to_edit.lines().count().max(1)
                                + if content_to_edit.ends_with('\n') {
                                    1
                                } else {
                                    0
                                };
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
                                    .interactive(!dialog_open)
                                    .desired_width(f32::INFINITY)
                                    .layouter(&mut layouter)
                                    .show(ui);

                                Self::paint_line_numbers(
                                    ui,
                                    &response,
                                    gutter_rect,
                                    diagnostics_for_file,
                                );
                                Self::paint_squiggles(ui, &response, diagnostics_for_file);

                                if response.response.clicked() || response.response.has_focus() {
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

                    self.tabs[idx].scroll_offset = scroll_output.state.offset.y;
                    editor_max_scroll =
                        (scroll_output.content_size.y - scroll_output.inner_rect.height()).max(0.0);
                    if editor_max_scroll > 0.0 {
                        editor_scroll_pct = self.tabs[idx].scroll_offset / editor_max_scroll;
                    }
                });
            },
        );

        // Resizing handle
        let (handle_rect, handle_response) =
            ui.allocate_exact_size(egui::vec2(available.x, handle_h), egui::Sense::drag());
        let handle_color = if handle_response.hovered() || handle_response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
            egui::Color32::from_rgb(100, 140, 200)
        } else {
            egui::Color32::from_rgb(55, 60, 70)
        };
        ui.painter().rect_filled(handle_rect, 0.0, handle_color);
        let dot_y = handle_rect.center().y;
        let dot_r = 1.5_f32;
        for dx in [-6.0_f32, 0.0, 6.0] {
            ui.painter().circle_filled(
                egui::pos2(handle_rect.center().x + dx, dot_y),
                dot_r,
                egui::Color32::from_rgb(160, 170, 190),
            );
        }
        if handle_response.dragged() {
            let delta = handle_response.drag_delta().y;
            self.md_split_ratio =
                ((self.md_split_ratio * available.y + delta) / available.y).clamp(0.1, 0.9);
        }

        // Bottom half: Preview
        let mut preview_scroll_pct = 0.0;
        let mut preview_max_scroll = 0.0;

        ui.allocate_ui_with_layout(
            egui::vec2(available.x, bottom_h),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.label(egui::RichText::new(i18n.get("editor-preview-label")).strong());
                ui.separator();

                // Return to dark theme with dynamic font
                let preview_frame = egui::Frame::new()
                    .fill(egui::Color32::from_rgb(33, 37, 43)) // Dark gray background
                    .inner_margin(egui::Margin::same(24));

                preview_frame.show(ui, |ui| {
                    // Dynamic font adaptation based on editor settings
                    let font_size = Self::current_editor_font_size(ui);

                    if let Some(body) = ui.style_mut().text_styles.get_mut(&egui::TextStyle::Body) {
                        body.size = font_size;
                    }
                    if let Some(heading) = ui
                        .style_mut()
                        .text_styles
                        .get_mut(&egui::TextStyle::Heading)
                    {
                        heading.size = font_size * 1.4;
                    }

                    let md_scroll_offset = self.tabs[idx].md_scroll_offset;

                    let preview_scroll_output = egui::ScrollArea::vertical()
                        .id_salt("md_preview_scroll")
                        .auto_shrink([false, false])
                        .vertical_scroll_offset(md_scroll_offset)
                        .show(ui, |ui| {
                            let tab = &mut self.tabs[idx];
                            let content = &tab.content;
                            Self::render_markdown_preview(ui, &mut tab.md_cache, content);
                        });

                    let tab = &mut self.tabs[idx];
                    tab.md_scroll_offset = preview_scroll_output.state.offset.y;
                    preview_max_scroll = (preview_scroll_output.content_size.y
                        - preview_scroll_output.inner_rect.height())
                    .max(0.0);
                    if preview_max_scroll > 0.0 {
                        preview_scroll_pct = tab.md_scroll_offset / preview_max_scroll;
                    }
                });
            },
        );

        // Proportional Sync
        let tab = &mut self.tabs[idx];
        if tab.scroll_offset != prev_editor_scroll {
            // Editor scrolled -> update preview offset proportionally
            tab.md_scroll_offset =
                (editor_scroll_pct * preview_max_scroll).clamp(0.0, preview_max_scroll);
        } else if tab.md_scroll_offset != prev_preview_scroll {
            // Preview scrolled -> update editor offset proportionally
            tab.scroll_offset =
                (preview_scroll_pct * editor_max_scroll).clamp(0.0, editor_max_scroll);
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

        self.process_lsp_interactions(ui, idx, &tab_path, lsp_client, &saved_response, i18n);

        clicked
    }
}
