use super::*;
use eframe::egui;
use std::time::Instant;

impl Editor {
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
}
