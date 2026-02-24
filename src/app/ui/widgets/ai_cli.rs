use eframe::egui;

/// StandardAI provides shared logic for AI CLI-like interfaces.
pub struct StandardAI;

impl StandardAI {
    /// Renders a multiline text edit with CLI-like behavior:
    /// - Enter: returns true (should send)
    /// - Shift+Enter, Ctrl+Enter, Ctrl+J: inserts a newline
    /// - Arrow Up/Down: cycles through history
    /// - Returns (should_send, egui_response)
    pub fn ui_input(
        ui: &mut egui::Ui,
        text: &mut String,
        font_size: f32,
        hint: &str,
        history: &[String],
        history_index: &mut Option<usize>,
    ) -> (bool, egui::Response) {
        let mut send = false;

        // Determine if we should intercept keys
        let (enter_pressed, shift, ctrl, j_pressed, up_pressed, down_pressed) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::Enter),
                i.modifiers.shift,
                i.modifiers.ctrl,
                i.key_pressed(egui::Key::J),
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
            )
        });

        // 1. Enter without modifiers = SEND
        if enter_pressed && !shift && !ctrl {
            ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            if !text.trim().is_empty() {
                send = true;
                *history_index = None;
            }
        }

        // 2. Ctrl+J = NEWLINE
        if ctrl && j_pressed {
            ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::J));
            text.push('\n');
        }

        // 3. History Navigation
        if !history.is_empty() {
            if up_pressed {
                ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
                let new_idx = match *history_index {
                    None => Some(history.len().saturating_sub(1)),
                    Some(idx) => Some(idx.saturating_sub(1)),
                };
                if let Some(idx) = new_idx {
                    *text = history[idx].clone();
                    *history_index = Some(idx);
                }
            } else if down_pressed {
                ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
                if let Some(idx) = *history_index {
                    if idx + 1 < history.len() {
                        let next_idx = idx + 1;
                        *text = history[next_idx].clone();
                        *history_index = Some(next_idx);
                    } else {
                        *text = String::new();
                        *history_index = None;
                    }
                }
            }
        }

        // Render the text edit
        let response = ui.add(
            egui::TextEdit::multiline(text)
                .hint_text(hint)
                .desired_width(f32::INFINITY)
                .font(egui::FontId::monospace(font_size))
                .desired_rows(4),
        );

        (send, response)
    }

    /// Renders AI conversation history in a terminal-like style.
    pub fn ui_response(
        ui: &mut egui::Ui,
        conversation: &[(String, String)],
        font_size: f32,
        _cache: &mut egui_commonmark::CommonMarkCache,
    ) {
        let terminal_bg = egui::Color32::from_rgb(20, 20, 25);
        let terminal_text = egui::Color32::from_rgb(220, 220, 220);
        let question_text = egui::Color32::from_rgb(100, 180, 255);

        egui::Frame::new()
            .fill(terminal_bg)
            .inner_margin(8.0)
            .corner_radius(4.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(50)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("terminal_response_scroll")
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for (q, a) in conversation {
                            // Question (only if not empty)
                            if !q.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(">>>")
                                            .color(question_text)
                                            .monospace()
                                            .size(font_size),
                                    );
                                    let mut q_mut = q.clone();
                                    ui.add(
                                        egui::TextEdit::multiline(&mut q_mut)
                                            .font(egui::FontId::monospace(font_size))
                                            .text_color(question_text)
                                            .code_editor()
                                            .lock_focus(false)
                                            .interactive(true)
                                            .desired_width(f32::INFINITY),
                                    );
                                });
                                ui.add_space(4.0);
                            }

                            // Answer
                            if !a.is_empty() {
                                let mut a_mut = a.clone();
                                ui.add(
                                    egui::TextEdit::multiline(&mut a_mut)
                                        .font(egui::FontId::monospace(font_size))
                                        .text_color(terminal_text)
                                        .code_editor()
                                        .lock_focus(false)
                                        .interactive(true)
                                        .desired_width(f32::INFINITY),
                                );
                            }

                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(12.0);
                        }
                    });
            });
    }
}
