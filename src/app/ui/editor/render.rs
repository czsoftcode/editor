use std::time::Instant;

use eframe::egui;

use crate::config;

use super::search::apply_search_highlights;
use super::{Editor, SaveStatus};
use crate::app::ui::widgets::tab_bar::TabBarAction;

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

                        Self::paint_line_numbers(ui, &response, gutter_rect);

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

        if let Some(response) = &saved_response {
            self.show_editor_context_menu(response, i18n);
        }
        if content_changed && self.show_search {
            self.update_search();
        }

        clicked
    }

    // --- Markdown split view ---

    pub(super) fn ui_markdown_split(
        &mut self,
        ui: &mut egui::Ui,
        dialog_open: bool,
        i18n: &crate::i18n::I18n,
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

                                Self::paint_line_numbers(ui, &response, gutter_rect);

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

    pub(super) fn paint_line_numbers(
        ui: &egui::Ui,
        output: &egui::text_edit::TextEditOutput,
        gutter_rect: egui::Rect,
    ) {
        let font_size = Self::current_editor_font_size(ui);
        let font_id = egui::FontId::monospace(font_size);
        let gutter_color = egui::Color32::from_rgb(130, 130, 130);
        let highlight_color = egui::Color32::from_rgba_unmultiplied(80, 65, 15, 50);
        let painter = ui.painter();

        let galley_pos = output.galley_pos;
        let galley = &output.galley;

        let cursor_row = output.cursor_range.map(|cr| cr.primary.rcursor.row);

        let mut line_num: usize = 1;
        let mut is_new_line = true;

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
                painter.rect_filled(highlight_rect, 0.0, highlight_color);
            }

            if is_new_line {
                painter.text(
                    egui::pos2(gutter_rect.right() - 4.0, y),
                    egui::Align2::RIGHT_TOP,
                    format!("{}", line_num),
                    font_id.clone(),
                    gutter_color,
                );
                line_num += 1;
            }
            is_new_line = row.ends_with_newline;
        }
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
