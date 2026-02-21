use std::time::Instant;

use eframe::egui;

use crate::config;

use super::search::apply_search_highlights;
use super::{Editor, SaveStatus};
use crate::app::ui::widgets::tab_bar::TabBarAction;
use std::collections::HashMap;

fn editor_line_count(content: &str) -> usize {
    content.lines().count().max(1) + usize::from(content.ends_with('\n'))
}

fn goto_centered_scroll_offset(
    line: usize,
    total_lines: usize,
    row_height: f32,
    viewport_height: f32,
) -> f32 {
    if row_height <= 0.0 || viewport_height <= 0.0 || total_lines == 0 {
        return 0.0;
    }

    let max_line_index = total_lines.saturating_sub(1) as f32;
    let line_index = (line.saturating_sub(1) as f32).min(max_line_index);
    let line_y = line_index * row_height;
    let centered = line_y - (viewport_height - row_height) * 0.5;

    let doc_height = total_lines as f32 * row_height;
    let max_scroll = (doc_height - viewport_height).max(0.0);
    centered.clamp(0.0, max_scroll)
}

fn restore_saved_cursor(
    ctx: &egui::Context,
    edit_id: egui::Id,
    cursor_range: Option<egui::text::CursorRange>,
) {
    if let Some(saved) = cursor_range {
        let mut state = egui::text_edit::TextEditState::load(ctx, edit_id).unwrap_or_default();
        state.cursor.set_char_range(Some(saved.as_ccursor_range()));
        state.store(ctx, edit_id);
    }
}

impl Editor {
    // --- Tab bar ---

    pub(super) fn tab_bar(&mut self, ui: &mut egui::Ui, action: &mut Option<TabBarAction>) {
        let btn_w = config::TAB_BTN_WIDTH;
        let initial_scroll = self.tab_scroll_x;
        let active_tab = self.active_tab;
        let tab_count = self.tabs.len();
        let need_scroll = self.scroll_to_active;

        let tab_data: Vec<(String, bool, bool)> = self
            .tabs
            .iter()
            .map(|t| {
                let name = t
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "???".to_string());
                let label = if t.deleted {
                    format!("{} \u{26A0}", name)
                } else if t.modified {
                    format!("{} \u{25CF}", name)
                } else {
                    name
                };
                (label, t.modified, t.deleted)
            })
            .collect();

        let mut scroll_left = false;
        let mut scroll_right = false;
        let mut new_scroll_x = initial_scroll;
        let mut active_rect: Option<egui::Rect> = None;

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    initial_scroll > 0.0,
                    egui::Button::new("◀").min_size(egui::vec2(btn_w, 0.0)),
                )
                .clicked()
            {
                scroll_left = true;
            }

            let avail_w = (ui.available_width() - btn_w - ui.spacing().item_spacing.x).max(50.0);
            let mut tab_action: Option<TabBarAction> = None;

            let out = egui::ScrollArea::horizontal()
                .id_salt("tab_scroll")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .scroll_offset(egui::vec2(initial_scroll, 0.0))
                .max_width(avail_w)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (idx, (label, _, deleted)) in tab_data.iter().enumerate() {
                            let is_active = active_tab == Some(idx);
                            let mut text = egui::RichText::new(label).size(config::UI_FONT_SIZE);
                            if is_active {
                                text = text.strong();
                            }
                            if *deleted {
                                text = text.color(egui::Color32::from_rgb(200, 100, 80));
                            }

                            let r = ui.selectable_label(is_active, text);
                            if is_active {
                                active_rect = Some(r.rect);
                            }
                            if r.clicked() {
                                tab_action = Some(TabBarAction::Switch(idx));
                            }
                            if r.clicked_by(egui::PointerButton::Middle) {
                                tab_action = Some(TabBarAction::Close(idx));
                            }
                            if ui.small_button("\u{00D7}").clicked() {
                                tab_action = Some(TabBarAction::Close(idx));
                            }
                            if idx + 1 < tab_count {
                                ui.separator();
                            }
                        }
                    });
                });

            if let Some(a) = tab_action {
                *action = Some(a);
            }
            new_scroll_x = out.state.offset.x;

            if need_scroll && let Some(tab_rect) = active_rect {
                let inner = out.inner_rect;
                if tab_rect.max.x > inner.max.x {
                    new_scroll_x += tab_rect.max.x - inner.max.x + 8.0;
                } else if tab_rect.min.x < inner.min.x {
                    new_scroll_x = (new_scroll_x - (inner.min.x - tab_rect.min.x) - 8.0).max(0.0);
                }
            }

            let content_w = out.content_size.x;
            let visible_w = out.inner_rect.width();
            let can_right = new_scroll_x + visible_w < content_w - 1.0;

            if ui
                .add_enabled(
                    can_right,
                    egui::Button::new("▶").min_size(egui::vec2(btn_w, 0.0)),
                )
                .clicked()
            {
                scroll_right = true;
            }
        });

        if scroll_left {
            new_scroll_x = (new_scroll_x - config::TAB_SCROLL_STEP).max(0.0);
        }
        if scroll_right {
            new_scroll_x += config::TAB_SCROLL_STEP;
        }

        self.tab_scroll_x = new_scroll_x;
        self.scroll_to_active = false;
        ui.separator();
    }

    // --- Goto-line bar ---

    pub(super) fn goto_line_bar(&mut self, ui: &mut egui::Ui, i18n: &crate::i18n::I18n) {
        let mut do_jump = false;
        let mut do_close = false;
        let mut input_has_focus = false;
        let mut jump_by_enter = false;

        ui.horizontal(|ui| {
            ui.label(i18n.get("goto-line-prompt"));
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.goto_line_input)
                    .desired_width(80.0)
                    .id(egui::Id::new("goto_line_input")),
            );
            if self.goto_line_focus_requested {
                response.request_focus();
            }
            input_has_focus = response.has_focus();
            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
            if enter_pressed
                && (response.lost_focus() || input_has_focus || self.goto_line_focus_requested)
            {
                do_jump = true;
                jump_by_enter = true;
            }
            if ui.button("OK").clicked() {
                do_jump = true;
            }
            if ui.small_button("\u{00D7}").clicked() {
                do_close = true;
            }
        });
        ui.separator();
        if jump_by_enter {
            ui.input_mut(|i| {
                let _ = i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
            });
        }
        if self.goto_line_focus_requested && input_has_focus {
            self.goto_line_focus_requested = false;
        }

        if do_jump {
            if let Ok(n) = self.goto_line_input.trim().parse::<usize>()
                && n >= 1
            {
                self.pending_jump = Some(n);
            }
            self.show_goto_line = false;
            self.goto_line_focus_requested = false;
        }
        if do_close {
            self.show_goto_line = false;
            self.goto_line_focus_requested = false;
        }
    }

    pub(super) fn ui_binary(&mut self, ui: &mut egui::Ui, _i18n: &crate::i18n::I18n) -> bool {
        let idx = match self.active_tab {
            Some(i) => i,
            None => return false,
        };
        let ext = self.extension();

        let is_image = matches!(
            ext.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "svg"
        );

        if is_image {
            self.render_image_preview(ui, idx);
        } else {
            ui.centered_and_justified(|ui| {
                ui.vertical(|ui| {
                    ui.heading(format!("{} File", ext.to_uppercase()));
                    if let Some(data) = &self.tabs[idx].binary_data {
                        ui.label(format!("Size: {} B", data.len()));
                    }
                });
            });
        }
        false
    }

    fn render_image_preview(&mut self, ui: &mut egui::Ui, idx: usize) {
        let tab = &mut self.tabs[idx];
        let bytes = match &tab.binary_data {
            Some(b) => b,
            None => {
                ui.label("No data to display.");
                return;
            }
        };

        egui::ScrollArea::both()
            .id_salt("image_preview_scroll")
            .show(ui, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.add(
                        egui::Image::from_bytes(
                            format!("bytes://{}", tab.path.display()),
                            bytes.clone(),
                        )
                        .shrink_to_fit(),
                    );
                });
            });
    }

    // --- Normal editor ---

    pub(super) fn ui_normal(
        &mut self,
        ui: &mut egui::Ui,
        dialog_open: bool,
        i18n: &crate::i18n::I18n,
        diagnostics_for_file: Option<&Vec<async_lsp::lsp_types::Diagnostic>>,
        lsp_client: Option<&crate::app::lsp::LspClient>,
    ) -> bool {
        let idx = match self.active_tab {
            Some(i) => i,
            None => return false,
        };

        let bg = self.highlighter.background_color();
        let ext = self.extension();
        let fname = self.filename();
        let search_matches = self.search_matches.clone();
        let current_match = self.current_match;
        let tab_path = self.tabs[idx].path.clone();
        let current_scroll_y = self.tabs[idx].scroll_offset;

        let edit_id = egui::Id::new("editor_text").with(&tab_path);
        let line_count_for_jump = self
            .tabs
            .get(idx)
            .map(|tab| editor_line_count(&tab.content))
            .unwrap_or(1);

        let jump_line = self.pending_jump.take();
        let jump_char_idx: Option<usize> = jump_line.and_then(|line| {
            self.tabs.get(idx).map(|tab| {
                tab.content
                    .lines()
                    .take(line.saturating_sub(1))
                    .map(|l| l.chars().count() + 1)
                    .sum::<usize>()
            })
        });
        let has_jump = jump_char_idx.is_some();

        let font_size = Self::current_editor_font_size(ui);
        let font_id = egui::FontId::monospace(font_size);
        let row_height = ui.fonts(|f| f.row_height(&font_id));
        let viewport_height = (ui.available_height() - 16.0).max(row_height);
        let desired_scroll_y: Option<f32> = jump_line.map(|line| {
            goto_centered_scroll_offset(line, line_count_for_jump, row_height, viewport_height)
        });

        if let Some(char_idx) = jump_char_idx {
            let mut state =
                egui::text_edit::TextEditState::load(ui.ctx(), edit_id).unwrap_or_default();
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(char_idx),
                )));
            state.store(ui.ctx(), edit_id);
            ui.memory_mut(|m| m.request_focus(edit_id));
        }

        let should_request_editor_focus = self.focus_editor_requested
            && !dialog_open
            && !self.show_search
            && !self.show_goto_line;
        if should_request_editor_focus {
            if !has_jump {
                restore_saved_cursor(ui.ctx(), edit_id, self.tabs[idx].last_cursor_range);
            }
            ui.memory_mut(|m| m.request_focus(edit_id));
            self.focus_editor_requested = false;
        }

        let mut clicked = false;
        let mut saved_response: Option<egui::text_edit::TextEditOutput> = None;
        let mut content_changed = false;
        let mut updated_scroll_y: Option<f32> = None;

        let frame = egui::Frame::new()
            .fill(bg)
            .inner_margin(egui::Margin::same(8));

        frame.show(ui, |ui| {
            let scroll_y = desired_scroll_y.unwrap_or(current_scroll_y);
            let scroll_output = egui::ScrollArea::both()
                .id_salt(("editor_scroll", &tab_path))
                .auto_shrink([false, false])
                .vertical_scroll_offset(scroll_y)
                .show(ui, |ui| {
                    let highlighter = &self.highlighter;
                    let tab = &mut self.tabs[idx];

                    let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                        let mut job = highlighter.highlight(
                            text,
                            &ext,
                            &fname,
                            Self::current_editor_font_size(ui),
                        );
                        job.wrap.max_width = wrap_width;
                        apply_search_highlights(&mut job, &search_matches, current_match);
                        ui.fonts(|f| f.layout_job(job))
                    };

                    let line_count = editor_line_count(&tab.content);
                    let gutter_width = Self::gutter_width(ui, line_count);

                    ui.horizontal_top(|ui| {
                        let (gutter_rect, _) = ui.allocate_exact_size(
                            egui::vec2(gutter_width, ui.available_height()),
                            egui::Sense::hover(),
                        );

                        let response = egui::TextEdit::multiline(&mut tab.content)
                            .id(edit_id)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .interactive(!dialog_open)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter)
                            .show(ui);

                        Self::paint_line_numbers(ui, &response, gutter_rect, diagnostics_for_file);
                        Self::paint_squiggles(ui, &response, diagnostics_for_file);

                        if response.response.clicked() || response.response.has_focus() {
                            clicked = true;
                        }
                        if response.response.changed() {
                            tab.modified = true;
                            tab.last_edit = Some(Instant::now());
                            tab.save_status = SaveStatus::Modified;
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

        if content_changed && let Some(lsp) = lsp_client {
            let tab = &mut self.tabs[idx];
            // Only send didChange if didOpen was already sent (lsp_version > 0).
            // If lsp_version == 0, didOpen will fire next frame with current content.
            if tab.lsp_version > 0
                && let Ok(uri) = async_lsp::lsp_types::Url::from_file_path(&tab.path)
            {
                tab.lsp_version += 1;
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
        self.process_lsp_interactions(ui, idx, &tab_path, lsp_client, &saved_response);

        clicked
    }

    // --- Markdown split view ---

    pub(super) fn ui_markdown_split(
        &mut self,
        ui: &mut egui::Ui,
        dialog_open: bool,
        i18n: &crate::i18n::I18n,
        diagnostics_for_file: Option<&Vec<async_lsp::lsp_types::Diagnostic>>,
        lsp_client: Option<&crate::app::lsp::LspClient>,
    ) -> bool {
        let idx = match self.active_tab {
            Some(i) => i,
            None => return false,
        };

        let bg = self.highlighter.background_color();
        let ext = self.extension();
        let fname = self.filename();
        let search_matches = self.search_matches.clone();
        let current_match = self.current_match;
        let tab_path = self.tabs[idx].path.clone();
        let edit_id = egui::Id::new("editor_text").with(&tab_path);
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
                    let highlighter = &self.highlighter;
                    let tab = &mut self.tabs[idx];

                    let scroll_output = egui::ScrollArea::both()
                        .id_salt(("md_editor_scroll", &tab_path))
                        .auto_shrink([false, false])
                        .vertical_scroll_offset(tab.scroll_offset)
                        .show(ui, |ui| {
                            let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                                let mut job = highlighter.highlight(
                                    text,
                                    &ext,
                                    &fname,
                                    Self::current_editor_font_size(ui),
                                );
                                job.wrap.max_width = wrap_width;
                                apply_search_highlights(&mut job, &search_matches, current_match);
                                ui.fonts(|f| f.layout_job(job))
                            };

                            let line_count = tab.content.lines().count().max(1)
                                + if tab.content.ends_with('\n') { 1 } else { 0 };
                            let gutter_width = Self::gutter_width(ui, line_count);

                            ui.horizontal_top(|ui| {
                                let (gutter_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(gutter_width, ui.available_height()),
                                    egui::Sense::hover(),
                                );

                                let response = egui::TextEdit::multiline(&mut tab.content)
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
                                if response.response.changed() {
                                    tab.modified = true;
                                    tab.last_edit = Some(Instant::now());
                                    tab.save_status = SaveStatus::Modified;
                                    content_changed = true;
                                }
                                saved_response = Some(response);
                            });
                        });

                    tab.scroll_offset = scroll_output.state.offset.y;
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
        let scroll_offset = self.tabs[idx].scroll_offset;

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

                    egui::ScrollArea::vertical()
                        .id_salt("md_preview_scroll")
                        .auto_shrink([false, false])
                        .vertical_scroll_offset(scroll_offset)
                        .show(ui, |ui| {
                            let content = self.tabs[idx].content.clone();
                            self.render_markdown_preview(ui, &content);
                        });
                });
            },
        );

        if content_changed && let Some(lsp) = lsp_client {
            let tab = &mut self.tabs[idx];
            // Only send didChange if didOpen was already sent (lsp_version > 0).
            // If lsp_version == 0, didOpen will fire next frame with current content.
            if tab.lsp_version > 0
                && let Ok(uri) = async_lsp::lsp_types::Url::from_file_path(&tab.path)
            {
                tab.lsp_version += 1;
                lsp.notify_did_change(uri, tab.lsp_version, tab.content.clone());
            }
        }

        if let Some(response) = &saved_response {
            self.show_editor_context_menu(response, i18n);
        }
        if content_changed && self.show_search {
            self.update_search();
        }

        clicked
    }

    // --- Context menu ---

    pub(super) fn show_editor_context_menu(
        &mut self,
        response: &egui::text_edit::TextEditOutput,
        i18n: &crate::i18n::I18n,
    ) {
        let idx = match self.active_tab {
            Some(i) => i,
            None => return,
        };

        if response.cursor_range.is_some() {
            self.tabs[idx].last_cursor_range = response.cursor_range;
        }

        let tab = &mut self.tabs[idx];
        let menu_size = 15.0;

        response.response.context_menu(|ui| {
            let selected_text = tab.last_cursor_range.and_then(|cr| {
                let start = cr.primary.ccursor.index.min(cr.secondary.ccursor.index);
                let end = cr.primary.ccursor.index.max(cr.secondary.ccursor.index);
                if start != end {
                    Some(
                        tab.content
                            .chars()
                            .skip(start)
                            .take(end - start)
                            .collect::<String>(),
                    )
                } else {
                    None
                }
            });
            let has_selection = selected_text.is_some();

            if ui
                .add_enabled(
                    has_selection,
                    egui::Button::new(egui::RichText::new(i18n.get("btn-copy")).size(menu_size)),
                )
                .clicked()
            {
                if let Some(text) = &selected_text {
                    ui.ctx().copy_text(text.to_string());
                }
                ui.close_menu();
            }

            if ui
                .button(egui::RichText::new(i18n.get("btn-paste")).size(menu_size))
                .clicked()
            {
                if let Ok(mut clipboard) = arboard::Clipboard::new()
                    && let Ok(text) = clipboard.get_text()
                {
                    let insert_pos = tab
                        .last_cursor_range
                        .map(|cr| cr.primary.ccursor.index.max(cr.secondary.ccursor.index))
                        .unwrap_or(tab.content.chars().count());
                    let (start, end) = if let Some(cr) = tab.last_cursor_range {
                        let s = cr.primary.ccursor.index.min(cr.secondary.ccursor.index);
                        let e = cr.primary.ccursor.index.max(cr.secondary.ccursor.index);
                        if s != e {
                            (s, e)
                        } else {
                            (insert_pos, insert_pos)
                        }
                    } else {
                        (insert_pos, insert_pos)
                    };
                    let byte_start = tab
                        .content
                        .char_indices()
                        .nth(start)
                        .map(|(i, _)| i)
                        .unwrap_or(tab.content.len());
                    let byte_end = tab
                        .content
                        .char_indices()
                        .nth(end)
                        .map(|(i, _)| i)
                        .unwrap_or(tab.content.len());
                    tab.content.replace_range(byte_start..byte_end, &text);
                    tab.modified = true;
                    tab.last_edit = Some(Instant::now());
                    tab.save_status = SaveStatus::Modified;
                }
                ui.close_menu();
            }
        });
    }

    // --- Font & gutter helpers ---

    pub(super) fn current_editor_font_size(ui: &egui::Ui) -> f32 {
        ui.style()
            .text_styles
            .get(&egui::TextStyle::Monospace)
            .map(|f| f.size)
            .unwrap_or(config::EDITOR_FONT_SIZE)
    }

    pub(super) fn gutter_width(ui: &egui::Ui, line_count: usize) -> f32 {
        let font_size = Self::current_editor_font_size(ui);
        let font_id = egui::FontId::monospace(font_size);
        let digits = ((line_count.max(1) as f64).log10().floor() as usize) + 1;
        let char_width = ui.fonts(|f| f.glyph_width(&font_id, '0'));
        (digits as f32) * char_width + 12.0
    }

    /// Paints colored underlines under lines that have LSP diagnostics.
    /// ERROR = červená, WARNING = oranžová, INFO = modrá, HINT = zelená.
    pub(super) fn paint_squiggles(
        ui: &mut egui::Ui,
        output: &egui::text_edit::TextEditOutput,
        diagnostics_for_file: Option<&Vec<async_lsp::lsp_types::Diagnostic>>,
    ) {
        let Some(diagnostics) = diagnostics_for_file else { return };
        if diagnostics.is_empty() {
            return;
        }

        // Build map: 0-based logical line → (priority, color)
        // Lower priority number = higher severity (ERROR=0 wins over WARNING=1, etc.)
        let mut diag_by_line: HashMap<usize, (u8, egui::Color32)> = HashMap::new();
        for diag in diagnostics {
            let line = diag.range.start.line as usize;
            let (color, priority) = match diag.severity {
                Some(async_lsp::lsp_types::DiagnosticSeverity::ERROR) => {
                    (egui::Color32::from_rgba_unmultiplied(220, 50, 50, 160), 0u8)
                }
                Some(async_lsp::lsp_types::DiagnosticSeverity::WARNING) => {
                    (egui::Color32::from_rgba_unmultiplied(220, 160, 0, 130), 1)
                }
                Some(async_lsp::lsp_types::DiagnosticSeverity::INFORMATION) => {
                    (egui::Color32::from_rgba_unmultiplied(30, 150, 220, 100), 2)
                }
                Some(async_lsp::lsp_types::DiagnosticSeverity::HINT) => {
                    (egui::Color32::from_rgba_unmultiplied(80, 200, 80, 80), 3)
                }
                _ => continue,
            };
            diag_by_line
                .entry(line)
                .and_modify(|(prev_prio, prev_color)| {
                    if priority < *prev_prio {
                        *prev_prio = priority;
                        *prev_color = color;
                    }
                })
                .or_insert((priority, color));
        }

        let galley_pos = output.galley_pos;
        let galley = &output.galley;

        let mut logical_line: usize = 0;
        let mut is_new_line = true;

        for row in galley.rows.iter() {
            if is_new_line {
                if let Some((_, color)) = diag_by_line.get(&logical_line) {
                    let y = galley_pos.y + row.rect.min.y;
                    let row_h = row.rect.height();
                    let x_start = galley_pos.x + row.rect.min.x;
                    // Extend at least a bit so empty lines are also visible
                    let x_end = (galley_pos.x + row.rect.max.x).max(x_start + 40.0);

                    // 2px underline at the bottom of the row
                    let underline_rect = egui::Rect::from_min_max(
                        egui::pos2(x_start, y + row_h - 2.0),
                        egui::pos2(x_end, y + row_h),
                    );
                    ui.painter().rect_filled(underline_rect, 0.0, *color);
                }
                logical_line += 1;
            }
            is_new_line = row.ends_with_newline;
        }
    }

    pub(super) fn paint_line_numbers(
        ui: &mut egui::Ui,
        output: &egui::text_edit::TextEditOutput,
        gutter_rect: egui::Rect,
        diagnostics_for_file: Option<&Vec<async_lsp::lsp_types::Diagnostic>>,
    ) {
        let font_size = Self::current_editor_font_size(ui);
        let font_id = egui::FontId::monospace(font_size);
        let gutter_color = egui::Color32::from_rgb(130, 130, 130);
        let highlight_color = egui::Color32::from_rgba_unmultiplied(80, 65, 15, 50);

        let galley_pos = output.galley_pos;
        let galley = &output.galley;

        let cursor_row = output.cursor_range.map(|cr| cr.primary.rcursor.row);

        let mut line_num: usize = 1;
        let mut is_new_line = true;

        let diagnostic_map: HashMap<usize, Vec<&async_lsp::lsp_types::Diagnostic>> =
            diagnostics_for_file
                .map(|diagnostics| {
                    let mut map = HashMap::new();
                    for diag in diagnostics {
                        // LSP lines are 0-indexed, UI is 1-indexed
                        map.entry(diag.range.start.line as usize + 1)
                            .or_insert_with(Vec::new)
                            .push(diag);
                    }
                    map
                })
                .unwrap_or_default();

        for (row_idx, row) in galley.rows.iter().enumerate() {
            let y = galley_pos.y + row.rect.min.y;
            let row_height = row.rect.height();

            if cursor_row == Some(row_idx) {
                let highlight_rect = egui::Rect::from_min_size(
                    egui::pos2(gutter_rect.left(), y),
                    egui::vec2(
                        output.response.rect.right() - gutter_rect.left(),
                        row_height,
                    ),
                );
                ui.painter()
                    .rect_filled(highlight_rect, 0.0, highlight_color);
            }

            if is_new_line {
                let current_line_num = line_num;
                let text_color = gutter_color;
                let mut dot_color = None;
                let mut tooltip_text = Vec::new();

                if let Some(diagnostics_on_line) = diagnostic_map.get(&current_line_num) {
                    for diag in diagnostics_on_line {
                        let severity_color = match diag.severity {
                            Some(async_lsp::lsp_types::DiagnosticSeverity::ERROR) => {
                                egui::Color32::from_rgb(255, 60, 60)
                            }
                            Some(async_lsp::lsp_types::DiagnosticSeverity::WARNING) => {
                                egui::Color32::from_rgb(255, 180, 0)
                            }
                            Some(async_lsp::lsp_types::DiagnosticSeverity::INFORMATION) => {
                                egui::Color32::from_rgb(0, 180, 255)
                            }
                            Some(async_lsp::lsp_types::DiagnosticSeverity::HINT) => {
                                egui::Color32::from_rgb(100, 255, 100)
                            }
                            _ => gutter_color,
                        };
                        // Use the highest severity color for the dot
                        if dot_color.is_none()
                            || diag.severity
                                == Some(async_lsp::lsp_types::DiagnosticSeverity::ERROR)
                        {
                            dot_color = Some(severity_color);
                        }
                        tooltip_text.push(diag.message.clone());
                    }
                }

                let line_number_text = format!("{}", current_line_num);
                let text_galley =
                    ui.fonts(|f| f.layout_no_wrap(line_number_text, font_id.clone(), text_color));
                let text_pos = egui::pos2(gutter_rect.right() - 4.0 - text_galley.rect.width(), y);

                // Hover area: full gutter row width for easy tooltip triggering
                let line_number_rect = egui::Rect::from_min_max(
                    egui::pos2(gutter_rect.left(), y),
                    egui::pos2(gutter_rect.right(), y + row_height),
                );
                let response = ui.allocate_rect(line_number_rect, egui::Sense::hover());

                ui.painter().galley(text_pos, text_galley, text_color);

                // Dot on the LEFT side of the gutter — no overlap with line numbers
                if let Some(color) = dot_color {
                    ui.painter().circle_filled(
                        egui::pos2(gutter_rect.left() + 6.0, y + row_height / 2.0),
                        3.5,
                        color,
                    );
                }

                if response.hovered() && !tooltip_text.is_empty() {
                    response.on_hover_ui_at_pointer(|ui| {
                        for msg in &tooltip_text {
                            ui.label(msg);
                        }
                    });
                }

                line_num += 1;
            }
            is_new_line = row.ends_with_newline;
        }
    }

    // -----------------------------------------------------------------------
    // LSP interaction logic
    // -----------------------------------------------------------------------

    /// Handles all LSP interactions: hover, go-to-definition (F12), completion (Ctrl+Space).
    /// Called after the TextEdit frame is rendered so self can be freely borrowed.
    #[allow(clippy::too_many_arguments)]
    fn process_lsp_interactions(
        &mut self,
        ui: &egui::Ui,
        idx: usize,
        tab_path: &std::path::Path,
        lsp_client: Option<&crate::app::lsp::LspClient>,
        saved_response: &Option<egui::text_edit::TextEditOutput>,
    ) {
        use super::LspCompletionState;
        use super::LspHoverPopup;
        use super::LSP_HOVER_DEBOUNCE_MS;

        let tab_lsp_version = self.tabs[idx].lsp_version;
        let tab_is_binary = self.tabs[idx].is_binary;

        // --- Process pending async results ---

        // Hover result
        if let Some(rx) = &self.lsp_hover_rx {
            if let Ok(result) = rx.try_recv() {
                self.lsp_hover_rx = None;
                if let Some(hover) = result {
                    let content = hover_content_to_string(&hover);
                    if !content.trim().is_empty() {
                        self.lsp_hover_popup = Some(LspHoverPopup {
                            content,
                            screen_pos: self.lsp_hover_screen_pos.unwrap_or_default(),
                        });
                    }
                }
            }
        }

        // Go-to-definition result
        if let Some(rx) = &self.lsp_definition_rx {
            if let Ok(result) = rx.try_recv() {
                self.lsp_definition_rx = None;
                if let Some(def_resp) = result {
                    let location = match def_resp {
                        async_lsp::lsp_types::GotoDefinitionResponse::Scalar(loc) => Some(loc),
                        async_lsp::lsp_types::GotoDefinitionResponse::Array(locs) => {
                            locs.into_iter().next()
                        }
                        async_lsp::lsp_types::GotoDefinitionResponse::Link(links) => {
                            links.into_iter().next().map(|l| async_lsp::lsp_types::Location {
                                uri: l.target_uri,
                                range: l.target_range,
                            })
                        }
                    };
                    if let Some(loc) = location {
                        if let Ok(path) = loc.uri.to_file_path() {
                            let line = loc.range.start.line as usize + 1; // → 1-based
                            self.pending_lsp_navigate = Some((path, line));
                        }
                    }
                }
            }
        }

        // Completion result
        if let Some(rx) = &self.lsp_completion_rx {
            if let Ok(result) = rx.try_recv() {
                self.lsp_completion_rx = None;
                let items: Vec<_> = match result {
                    Some(async_lsp::lsp_types::CompletionResponse::Array(items)) => items,
                    Some(async_lsp::lsp_types::CompletionResponse::List(list)) => list.items,
                    None => Vec::new(),
                };
                let items: Vec<_> = items.into_iter().take(25).collect();
                if !items.is_empty() {
                    self.lsp_completion = Some(LspCompletionState {
                        items,
                        selected: 0,
                        screen_pos: self.lsp_completion_cursor_pos.unwrap_or_default(),
                    });
                }
            }
        }

        // --- Skip LSP interactions for binary files or when LSP not ready ---
        if tab_is_binary || tab_lsp_version == 0 {
            return;
        }

        // --- Pre-compute cursor/galley data from saved response ---
        let cursor_lsp_pos = saved_response.as_ref().and_then(|r| {
            r.cursor_range.map(|cr| async_lsp::lsp_types::Position {
                line: cr.primary.rcursor.row as u32,
                character: cr.primary.rcursor.column as u32,
            })
        });
        let cursor_screen_pos = saved_response.as_ref().and_then(|r| {
            r.cursor_range.map(|cr| {
                r.galley_pos + r.galley.pos_from_cursor(&cr.primary).min.to_vec2()
            })
        });
        let editor_rect = saved_response.as_ref().map(|r| r.response.rect);
        let galley_info = saved_response
            .as_ref()
            .map(|r| (r.galley_pos, r.galley.clone()));

        // --- Keyboard: F12 — go to definition ---
        let f12 = ui.ctx().input(|i| i.key_pressed(egui::Key::F12));
        if f12 {
            if let (Some(lsp), Some(pos)) = (lsp_client, cursor_lsp_pos) {
                if let Ok(uri) = async_lsp::lsp_types::Url::from_file_path(tab_path) {
                    self.lsp_definition_rx = Some(lsp.request_goto_definition(uri, pos));
                }
            }
        }

        // --- Keyboard: Ctrl+Space — trigger completion ---
        let ctrl_space = ui
            .ctx()
            .input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Space));
        if ctrl_space {
            if let (Some(lsp), Some(pos), Some(screen)) = (lsp_client, cursor_lsp_pos, cursor_screen_pos) {
                if let Ok(uri) = async_lsp::lsp_types::Url::from_file_path(tab_path) {
                    self.lsp_completion_cursor_pos = Some(screen);
                    self.lsp_completion = None;
                    self.lsp_completion_rx = Some(lsp.request_completion(uri, pos));
                }
            }
        }

        // --- Completion keyboard navigation (when popup is open) ---
        let mut completion_accepted_idx: Option<usize> = None;
        let mut completion_close = false;
        if self.lsp_completion.is_some() {
            let (up, down, enter, tab, esc) = ui.ctx().input_mut(|i| {
                (
                    i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Tab),
                    i.consume_key(egui::Modifiers::NONE, egui::Key::Escape),
                )
            });
            if let Some(ref mut comp) = self.lsp_completion {
                if up && comp.selected > 0 {
                    comp.selected -= 1;
                }
                if down && comp.selected + 1 < comp.items.len() {
                    comp.selected += 1;
                }
                if enter || tab {
                    completion_accepted_idx = Some(comp.selected);
                }
                if esc {
                    completion_close = true;
                }
            }
        }

        // --- Accept completion item ---
        // --- Render completion popup (before acceptance so click can set accept_idx) ---
        if let Some(ref comp) = self.lsp_completion {
            let comp_screen_pos = comp.screen_pos;
            let comp_selected = comp.selected;
            let comp_items = comp.items.clone();
            let mock_comp = LspCompletionState {
                items: comp_items,
                selected: comp_selected,
                screen_pos: comp_screen_pos,
            };
            if let Some(clicked_idx) = Self::render_completion_popup(ui, &mock_comp) {
                // Click overrides keyboard selection
                completion_accepted_idx = Some(clicked_idx);
            }
        }

        // --- Accept completion item (keyboard Enter/Tab or mouse click) ---
        if let Some(accept_idx) = completion_accepted_idx {
            // Clone item data to avoid borrow conflict
            let insert_text = self
                .lsp_completion
                .as_ref()
                .and_then(|c| c.items.get(accept_idx))
                .map(|item| {
                    item.insert_text
                        .clone()
                        .unwrap_or_else(|| item.label.clone())
                });

            if let Some(insert_text) = insert_text {
                let cursor_char_idx = saved_response
                    .as_ref()
                    .and_then(|r| r.cursor_range)
                    .map(|cr| cr.primary.ccursor.index)
                    .unwrap_or(0);
                let content = &self.tabs[idx].content;
                // Find word start: walk back while alphanumeric or '_'
                let word_start = content
                    .char_indices()
                    .take(cursor_char_idx)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
                    .last()
                    .map(|(i, _)| content[..i].chars().count())
                    .unwrap_or(cursor_char_idx);
                let byte_start = content
                    .char_indices()
                    .nth(word_start)
                    .map(|(i, _)| i)
                    .unwrap_or(content.len());
                let byte_end = content
                    .char_indices()
                    .nth(cursor_char_idx)
                    .map(|(i, _)| i)
                    .unwrap_or(content.len());
                {
                    let tab = &mut self.tabs[idx];
                    tab.content.replace_range(byte_start..byte_end, &insert_text);
                    tab.modified = true;
                    tab.last_edit = Some(std::time::Instant::now());
                    tab.save_status = super::SaveStatus::Modified;
                }
                let new_cursor_char = word_start + insert_text.chars().count();
                let edit_id = egui::Id::new("editor_text").with(tab_path);
                let mut state =
                    egui::text_edit::TextEditState::load(ui.ctx(), edit_id).unwrap_or_default();
                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(new_cursor_char),
                )));
                state.store(ui.ctx(), edit_id);
            }
            self.lsp_completion = None;
        }
        if completion_close {
            self.lsp_completion = None;
        }

        // --- Mouse hover for documentation ---
        if let (Some((galley_pos, galley)), Some(rect)) = (galley_info, editor_rect) {
            let ptr_pos = ui.ctx().pointer_hover_pos();
            if let Some(pos) = ptr_pos {
                if rect.contains(pos) {
                    // Convert mouse pos to LSP position via galley
                    let rel = pos - galley_pos;
                    let cursor = galley.cursor_from_pos(rel);
                    let lsp_pos = async_lsp::lsp_types::Position {
                        line: cursor.rcursor.row as u32,
                        character: cursor.rcursor.column as u32,
                    };
                    let pos_changed = self.lsp_hover_last_pos != Some(lsp_pos);
                    if pos_changed {
                        self.lsp_hover_last_pos = Some(lsp_pos);
                        self.lsp_hover_timer = Some(std::time::Instant::now());
                        self.lsp_hover_popup = None;
                        self.lsp_hover_rx = None;
                        self.lsp_hover_screen_pos = Some(pos);
                    }
                    // Debounce: trigger after LSP_HOVER_DEBOUNCE_MS of no movement
                    if let Some(timer) = self.lsp_hover_timer {
                        if timer.elapsed().as_millis() >= LSP_HOVER_DEBOUNCE_MS
                            && self.lsp_hover_rx.is_none()
                            && self.lsp_hover_popup.is_none()
                        {
                            if let (Some(lsp), Ok(uri)) = (
                                lsp_client,
                                async_lsp::lsp_types::Url::from_file_path(tab_path),
                            ) {
                                self.lsp_hover_rx = Some(lsp.request_hover(uri, lsp_pos));
                            }
                        }
                    }
                } else {
                    // Mouse left the editor area — dismiss popup
                    self.lsp_hover_popup = None;
                    self.lsp_hover_last_pos = None;
                    self.lsp_hover_timer = None;
                    self.lsp_hover_rx = None;
                }
            }
        }

        // --- Render hover popup ---
        if let Some(ref popup) = self.lsp_hover_popup {
            let content = popup.content.clone();
            let screen_pos = popup.screen_pos;
            Self::render_hover_popup(ui, &content, screen_pos);
        }
    }

    // -----------------------------------------------------------------------
    // LSP popup rendering
    // -----------------------------------------------------------------------

    /// Renders the hover documentation popup as a floating Area.
    /// Properly parses markdown code fences — code blocks are rendered monospace,
    /// prose sections are rendered as regular text.
    pub(super) fn render_hover_popup(
        ui: &egui::Ui,
        content: &str,
        screen_pos: egui::Pos2,
    ) {
        let popup_pos = screen_pos + egui::vec2(8.0, 16.0);
        egui::Area::new(egui::Id::new("lsp_hover_popup"))
            .fixed_pos(popup_pos)
            .order(egui::Order::Tooltip)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(35, 38, 46))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(70, 80, 100),
                    ))
                    .inner_margin(egui::Margin::same(10))
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        ui.set_max_width(520.0);
                        egui::ScrollArea::vertical()
                            .id_salt("lsp_hover_scroll")
                            .max_height(260.0)
                            .show(ui, |ui| {
                                let segments = parse_hover_segments(content);
                                let mut first = true;
                                for seg in &segments {
                                    if !first {
                                        ui.add_space(6.0);
                                        ui.separator();
                                        ui.add_space(4.0);
                                    }
                                    first = false;
                                    match seg {
                                        HoverSegment::Code(code) => {
                                            let trimmed = code.trim();
                                            if !trimmed.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(trimmed)
                                                        .monospace()
                                                        .color(egui::Color32::from_rgb(
                                                            180, 210, 255,
                                                        )),
                                                );
                                            }
                                        }
                                        HoverSegment::Prose(text) => {
                                            let trimmed = text.trim();
                                            if !trimmed.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(trimmed).color(
                                                        egui::Color32::from_rgb(200, 200, 200),
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                }
                            });
                    });
            });
    }

    /// Renders the completion dropdown and returns true if an item was accepted.
    /// Also handles keyboard navigation (arrow keys consumed pre-frame, but selected
    /// index is already updated by the time this runs).
    pub(super) fn render_completion_popup(
        ui: &egui::Ui,
        state: &super::LspCompletionState,
    ) -> Option<usize> {
        let mut accepted = None;

        let font_size = Self::current_editor_font_size(ui);
        let popup_pos = state.screen_pos + egui::vec2(0.0, font_size + 4.0);

        egui::Area::new(egui::Id::new("lsp_completion_popup"))
            .fixed_pos(popup_pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(38, 41, 50))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(80, 100, 130),
                    ))
                    .inner_margin(egui::Margin::same(4))
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        ui.set_max_width(380.0);
                        egui::ScrollArea::vertical()
                            .id_salt("lsp_completion_scroll")
                            .max_height(220.0)
                            .show(ui, |ui| {
                                for (i, item) in state.items.iter().enumerate() {
                                    let is_selected = i == state.selected;
                                    let kind_label = completion_kind_label(item.kind);
                                    let label = format!(
                                        "{} {}",
                                        kind_label,
                                        item.detail
                                            .as_deref()
                                            .map(|d| format!("{} — {}", item.label, d))
                                            .unwrap_or_else(|| item.label.clone()),
                                    );
                                    let text = egui::RichText::new(&label)
                                        .monospace()
                                        .size(font_size - 1.0);
                                    let response = ui.selectable_label(is_selected, text);
                                    if is_selected {
                                        response.scroll_to_me(None);
                                    }
                                    if response.clicked() {
                                        accepted = Some(i);
                                    }
                                }
                            });
                    });
            });

        accepted
    }
}

// ---------------------------------------------------------------------------
// Markdown hover parsing helpers
// ---------------------------------------------------------------------------

/// A segment of parsed markdown hover content.
enum HoverSegment {
    /// Content of a fenced code block (``` ... ```).
    Code(String),
    /// Plain prose text outside code blocks.
    Prose(String),
}

/// Parses markdown content into alternating Code/Prose segments.
/// Properly handles fenced code blocks (```lang ... ```).
fn parse_hover_segments(content: &str) -> Vec<HoverSegment> {
    let mut segments: Vec<HoverSegment> = Vec::new();
    let mut in_fence = false;
    let mut current: Vec<&str> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_fence {
                // Close fence — emit code block
                let code = current.join("\n");
                if !code.trim().is_empty() {
                    segments.push(HoverSegment::Code(code));
                }
                current.clear();
                in_fence = false;
            } else {
                // Open fence — flush accumulated prose first
                let prose = current.join("\n");
                if !prose.trim().is_empty() {
                    segments.push(HoverSegment::Prose(prose));
                }
                current.clear();
                in_fence = true;
                // Skip the fence-opening line (it contains the language tag, not code)
            }
        } else {
            current.push(line);
        }
    }

    // Flush remaining content
    if !current.is_empty() {
        let text = current.join("\n");
        if !text.trim().is_empty() {
            if in_fence {
                segments.push(HoverSegment::Code(text));
            } else {
                segments.push(HoverSegment::Prose(text));
            }
        }
    }

    segments
}

/// Converts LSP Hover content to a plain string for display.
pub(super) fn hover_content_to_string(hover: &async_lsp::lsp_types::Hover) -> String {
    match &hover.contents {
        async_lsp::lsp_types::HoverContents::Markup(mc) => mc.value.clone(),
        async_lsp::lsp_types::HoverContents::Scalar(ms) => marked_string_value(ms),
        async_lsp::lsp_types::HoverContents::Array(arr) => arr
            .iter()
            .map(marked_string_value)
            .collect::<Vec<_>>()
            .join("\n\n"),
    }
}

fn marked_string_value(ms: &async_lsp::lsp_types::MarkedString) -> String {
    match ms {
        async_lsp::lsp_types::MarkedString::String(s) => s.clone(),
        async_lsp::lsp_types::MarkedString::LanguageString(ls) => ls.value.clone(),
    }
}

/// Short kind label for a completion item (shown before the label).
fn completion_kind_label(kind: Option<async_lsp::lsp_types::CompletionItemKind>) -> &'static str {
    use async_lsp::lsp_types::CompletionItemKind;
    match kind {
        Some(CompletionItemKind::FUNCTION) | Some(CompletionItemKind::METHOD) => "fn",
        Some(CompletionItemKind::CONSTRUCTOR) => "fn",
        Some(CompletionItemKind::STRUCT) | Some(CompletionItemKind::CLASS) => "st",
        Some(CompletionItemKind::INTERFACE) => "if",
        Some(CompletionItemKind::ENUM) => "en",
        Some(CompletionItemKind::ENUM_MEMBER) => "ev",
        Some(CompletionItemKind::CONSTANT) => "co",
        Some(CompletionItemKind::VARIABLE)
        | Some(CompletionItemKind::FIELD)
        | Some(CompletionItemKind::PROPERTY) => "va",
        Some(CompletionItemKind::KEYWORD) => "kw",
        Some(CompletionItemKind::MODULE) => "mo",
        Some(CompletionItemKind::TYPE_PARAMETER) => "ty",
        Some(CompletionItemKind::SNIPPET) => "sn",
        _ => "  ",
    }
}

#[cfg(test)]
mod tests {
    use super::goto_centered_scroll_offset;

    #[test]
    fn goto_scroll_centers_when_possible() {
        let offset = goto_centered_scroll_offset(50, 200, 20.0, 200.0);
        assert!((offset - 890.0).abs() < f32::EPSILON);
    }

    #[test]
    fn goto_scroll_clamps_near_top() {
        let offset = goto_centered_scroll_offset(1, 200, 20.0, 200.0);
        assert!((offset - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn goto_scroll_clamps_near_bottom() {
        let offset = goto_centered_scroll_offset(200, 200, 20.0, 200.0);
        assert!((offset - 3800.0).abs() < f32::EPSILON);
    }
}
