use eframe::egui;

pub fn ui_input(
    ui: &mut egui::Ui,
    text: &mut String,
    font_size: f32,
    hint: &str,
    history: &[String],
    history_index: &mut Option<usize>,
) -> (bool, egui::Response) {
    let mut send = false;

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

    if enter_pressed && !shift && !ctrl {
        ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
        if !text.trim().is_empty() {
            send = true;
            *history_index = None;
        }
    }

    if ctrl && j_pressed {
        ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::J));
        text.push('\n');
    }

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

    let response = ui.add(
        egui::TextEdit::multiline(text)
            .hint_text(hint)
            .desired_width(f32::INFINITY)
            .font(egui::FontId::monospace(font_size))
            .margin(egui::vec2(4.0, font_size * 0.5))
            .desired_rows(1)
            .frame(false),
    );

    (send, response)
}
