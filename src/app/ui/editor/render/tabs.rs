use crate::app::ui::editor::*;
use crate::app::ui::widgets::tab_bar::TabBarAction;
use crate::config;
use eframe::egui;

fn tab_mode_marker(is_active: bool, save_mode: &crate::settings::SaveMode) -> &'static str {
    if !is_active {
        return "";
    }
    match save_mode {
        crate::settings::SaveMode::Automatic => "·A",
        crate::settings::SaveMode::Manual => "·M",
    }
}

fn tab_label_with_mode_indicator(
    name: &str,
    modified: bool,
    deleted: bool,
    is_active: bool,
    save_mode: &crate::settings::SaveMode,
) -> String {
    if deleted {
        return format!("{} \u{26A0}", name);
    }

    let mut label = String::from(name);
    if modified {
        label.push_str(" \u{25CF}");
    }
    let marker = tab_mode_marker(is_active, save_mode);
    if !marker.is_empty() {
        label.push(' ');
        label.push_str(marker);
    }
    label
}

#[cfg(test)]
pub(crate) fn tab_label_with_mode_indicator_for_tests(
    name: &str,
    modified: bool,
    deleted: bool,
    is_active: bool,
    save_mode: &crate::settings::SaveMode,
) -> String {
    tab_label_with_mode_indicator(name, modified, deleted, is_active, save_mode)
}

impl Editor {
    pub fn tab_bar(
        &mut self,
        ui: &mut egui::Ui,
        action: &mut Option<TabBarAction>,
        settings: &crate::settings::Settings,
        i18n: &crate::i18n::I18n,
    ) {
        let btn_w = config::TAB_BTN_WIDTH;
        let initial_scroll = self.tab_scroll_x;
        let active_tab = self.active_tab;
        let tab_count = self.tabs.len();
        let need_scroll = self.scroll_to_active;

        let tab_data: Vec<(String, bool)> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(idx, t)| {
                let name = t
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "???".to_string());
                let label = tab_label_with_mode_indicator(
                    &name,
                    t.modified,
                    t.deleted,
                    active_tab == Some(idx),
                    &settings.save_mode,
                );
                (label, t.deleted)
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
                        for (idx, (label, deleted)) in tab_data.iter().enumerate() {
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

                            // Context menu (pravý klik na tab)
                            r.context_menu(|ui| {
                                // "Historie souboru" — jen pro ne-binární taby
                                let is_binary = self.tabs.get(idx).is_none_or(|t| t.is_binary);
                                if !is_binary
                                    && ui.button(i18n.get("tab-context-history")).clicked()
                                {
                                    tab_action = Some(TabBarAction::ShowHistory(idx));
                                    ui.close_menu();
                                }
                                // "Zavřít tab"
                                if ui.button(i18n.get("tab-context-close")).clicked() {
                                    tab_action = Some(TabBarAction::Close(idx));
                                    ui.close_menu();
                                }
                            });

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

    pub fn goto_line_bar(&mut self, ui: &mut egui::Ui, i18n: &crate::i18n::I18n) {
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
                self.pending_jump = Some((n, 1));
            }
            self.show_goto_line = false;
            self.goto_line_focus_requested = false;
        }
        if do_close {
            self.show_goto_line = false;
            self.goto_line_focus_requested = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::tab_label_with_mode_indicator;
    use crate::settings::SaveMode;

    #[test]
    fn tab_save_mode_indicator_is_visible_for_active_tab_only() {
        let active_label =
            tab_label_with_mode_indicator("main.rs", false, false, true, &SaveMode::Manual);
        let inactive_label =
            tab_label_with_mode_indicator("lib.rs", false, false, false, &SaveMode::Manual);

        assert!(active_label.contains("·M"));
        assert!(!inactive_label.contains("·M"));
    }

    #[test]
    fn tab_save_mode_indicator_keeps_dirty_symbol_primary() {
        let label = tab_label_with_mode_indicator("main.rs", true, false, true, &SaveMode::Manual);
        assert!(label.contains("● ·M"));
    }

    #[test]
    fn tab_save_mode_indicator_uses_active_mode_symbol() {
        let auto_label =
            tab_label_with_mode_indicator("main.rs", false, false, true, &SaveMode::Automatic);
        assert!(auto_label.contains("·A"));
    }
}
