use crate::app::ui::editor::Editor;
use eframe::egui;

impl Editor {
    /// Renders a modal-like picker for multiple LSP references.
    pub fn render_references_picker(
        &mut self,
        ctx: &egui::Context,
        i18n: &crate::i18n::I18n,
    ) -> Option<(std::path::PathBuf, usize, usize)> {
        let picker = self.lsp_references.as_mut()?;

        let key_up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
        let key_down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
        let key_enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
        let key_esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));

        if key_up && picker.selected > 0 {
            picker.selected -= 1;
            picker.scroll_to_selected = true;
        }
        if key_down && picker.selected + 1 < picker.items.len() {
            picker.selected += 1;
            picker.scroll_to_selected = true;
        }

        let mut result = None;
        let mut show_flag = true;
        let mut close_requested = false;

        if key_enter && !picker.items.is_empty() {
            let item = &picker.items[picker.selected];
            result = Some((item.path.clone(), item.line, item.character));
            close_requested = true;
        }

        use crate::app::ui::widgets::modal::StandardModal;
        let modal = StandardModal::new(i18n.get("lsp-references-heading"), "lsp_references_modal")
            .with_size(700.0, 500.0);

        let mut should_close = false;
        modal.show(ctx, &mut show_flag, |ui| {
            if let Some(close) = modal.ui_footer_actions(ui, i18n, |f| {
                if f.close() || f.cancel() {
                    return Some(true);
                }
                None
            }) && close
            {
                should_close = true;
            }

            modal.ui_body(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("lsp_ref_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for (i, item) in picker.items.iter().enumerate() {
                            let is_sel = i == picker.selected;
                            let filename = item
                                .path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "???".to_string());

                            let display_text = if item.text.is_empty() {
                                format!("{}:{}:{}", filename, item.line, item.character)
                            } else {
                                let mut truncated = item.text.clone();
                                if truncated.len() > 100 {
                                    truncated.truncate(97);
                                    truncated.push_str("...");
                                }
                                format!(
                                    "{}:{}:{}  {}",
                                    filename, item.line, item.character, truncated
                                )
                            };

                            let text = egui::RichText::new(display_text).monospace().size(12.0);

                            let r = ui.selectable_label(is_sel, text);
                            if is_sel && picker.scroll_to_selected {
                                r.scroll_to_me(None);
                            }
                            if r.clicked() {
                                result = Some((item.path.clone(), item.line, item.character));
                                close_requested = true;
                            }
                            if ui.is_rect_visible(r.rect) {
                                ui.label(
                                    egui::RichText::new(item.path.to_string_lossy())
                                        .weak()
                                        .size(10.0),
                                );
                            }
                            ui.separator();
                        }
                    });
            });
        });

        if should_close {
            show_flag = false;
        }

        if let Some(p) = self.lsp_references.as_mut()
            && p.scroll_to_selected
        {
            p.scroll_to_selected = false;
        }

        if close_requested || key_esc || !show_flag {
            self.lsp_references = None;
        }
        result
    }
}
