use crate::app::ui::editor::Editor;
use eframe::egui;

impl Editor {
    /// Handles smart typing: auto-indent on Enter and smart un-indent on '}'.
    pub fn handle_smart_typing(&mut self, ui: &mut egui::Ui, edit_id: egui::Id, idx: usize) {
        if !ui.memory(|m| m.has_focus(edit_id))
            || self.lsp_completion.is_some()
            || self.lsp_references.is_some()
        {
            return;
        }

        let mut typed_brace = false;
        let enter = ui.input_mut(|i| {
            i.events.retain(|e| {
                if let egui::Event::Text(t) = e
                    && t == "}"
                {
                    typed_brace = true;
                    return false;
                }
                true
            });
            i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
        });

        if !enter && !typed_brace {
            return;
        }

        let tab = &mut self.tabs[idx];
        let mut state = egui::text_edit::TextEditState::load(ui.ctx(), edit_id).unwrap_or_default();

        if let Some(range) = state.cursor.char_range() {
            let cursor_char_idx = range.primary.index;
            let byte_idx = tab
                .content
                .char_indices()
                .nth(cursor_char_idx)
                .map(|(i, _)| i)
                .unwrap_or(tab.content.len());

            if enter {
                let prefix = &tab.content[..byte_idx];
                let line_start_byte_idx = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
                let current_line = &tab.content[line_start_byte_idx..byte_idx];

                let whitespace: String = current_line
                    .chars()
                    .take_while(|c| c.is_whitespace() && *c != '\n')
                    .collect();

                let mut to_insert = format!("\n{}", whitespace);
                if current_line.trim_end().ends_with('{') {
                    to_insert.push_str("    ");
                }

                tab.content.insert_str(byte_idx, &to_insert);
                let new_cursor_char_idx = cursor_char_idx + to_insert.chars().count();
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::one(
                        egui::text::CCursor::new(new_cursor_char_idx),
                    )));
            } else if typed_brace {
                let prefix = &tab.content[..byte_idx];
                let line_start_byte_idx = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_to_cursor = &tab.content[line_start_byte_idx..byte_idx];

                if line_to_cursor.chars().all(|c| c.is_whitespace())
                    && line_to_cursor.ends_with("    ")
                {
                    tab.content.replace_range(byte_idx - 4..byte_idx, "}");
                    let new_cursor_char_idx = cursor_char_idx - 3;
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(
                            egui::text::CCursor::new(new_cursor_char_idx),
                        )));
                } else {
                    tab.content.insert(byte_idx, '}');
                    let new_cursor_char_idx = cursor_char_idx + 1;
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(
                            egui::text::CCursor::new(new_cursor_char_idx),
                        )));
                }
            }

            tab.modified = true;
            tab.last_edit = Some(std::time::Instant::now());
            tab.save_status = crate::app::ui::editor::SaveStatus::Modified;
            state.store(ui.ctx(), edit_id);
        }
    }
}
