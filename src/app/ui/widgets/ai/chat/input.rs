use eframe::egui;

/// State for slash command autocomplete popup.
#[derive(Default)]
pub struct SlashAutocomplete {
    /// Whether the popup is currently visible.
    pub active: bool,
    /// Currently selected index in the filtered list (0-based).
    pub selected: usize,
    /// True when user dismissed with Escape — prevents re-activation until text changes.
    pub dismissed: bool,
    /// Previous frame's text — used to detect changes and reset dismissed state.
    pub prev_text: String,
}

pub fn ui_input(
    ui: &mut egui::Ui,
    text: &mut String,
    font_size: f32,
    hint: &str,
    history: &[String],
    history_index: &mut Option<usize>,
    autocomplete: &mut SlashAutocomplete,
) -> (bool, egui::Response) {
    let mut send = false;
    let mut refocus = false;

    let (
        enter_pressed,
        shift,
        ctrl,
        j_pressed,
        up_pressed,
        down_pressed,
        tab_pressed,
        escape_pressed,
    ) = ui.input(|i| {
        (
            i.key_pressed(egui::Key::Enter),
            i.modifiers.shift,
            i.modifiers.ctrl,
            i.key_pressed(egui::Key::J),
            i.key_pressed(egui::Key::ArrowUp),
            i.key_pressed(egui::Key::ArrowDown),
            i.key_pressed(egui::Key::Tab),
            i.key_pressed(egui::Key::Escape),
        )
    });

    // Reset dismissed state when text changes
    if *text != autocomplete.prev_text {
        autocomplete.dismissed = false;
        autocomplete.prev_text = text.clone();
    }

    // Detect autocomplete activation:
    // 1. Top-level: prompt starts with `/` and no whitespace after it
    // 2. GSD sub-level: prompt starts with `/gsd ` (show subcommand autocomplete)
    let is_gsd_sub = text.starts_with("/gsd ") && !text[5..].contains(char::is_whitespace);
    let is_top_level = text.starts_with('/') && !text[1..].contains(char::is_whitespace);
    let should_show = is_top_level || is_gsd_sub;
    if should_show && !autocomplete.dismissed {
        autocomplete.active = true;
    } else if !should_show {
        autocomplete.active = false;
        autocomplete.selected = 0;
    }

    // Get matching commands when autocomplete is active
    let matches = if autocomplete.active {
        if is_gsd_sub {
            let filter = &text[5..]; // after "/gsd "
            crate::app::ui::terminal::ai_chat::gsd::matching_subcommands(filter)
        } else {
            let filter = &text[1..]; // strip leading `/`
            crate::app::ui::terminal::ai_chat::slash::matching_commands(filter)
        }
    } else {
        Vec::new()
    };

    // Deactivate if no matches
    if autocomplete.active && matches.is_empty() {
        autocomplete.active = false;
        autocomplete.selected = 0;
    }

    // Clamp selected index
    if autocomplete.active && autocomplete.selected >= matches.len() {
        autocomplete.selected = matches.len().saturating_sub(1);
    }

    // Keyboard handling when autocomplete is active
    if autocomplete.active && !matches.is_empty() {
        if escape_pressed {
            ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            autocomplete.active = false;
            autocomplete.dismissed = true;
        } else if tab_pressed {
            ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));
            let selected_cmd = matches[autocomplete.selected].0;
            if is_gsd_sub {
                *text = format!("/gsd {} ", selected_cmd);
            } else {
                *text = format!("/{} ", selected_cmd);
            }
            autocomplete.active = false;
            autocomplete.prev_text = text.clone();
            refocus = true;
        } else if enter_pressed && !shift && !ctrl {
            ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            let selected_cmd = matches[autocomplete.selected].0;
            if is_gsd_sub {
                *text = format!("/gsd {}", selected_cmd);
            } else {
                *text = format!("/{}", selected_cmd);
            }
            autocomplete.active = false;
            autocomplete.prev_text = text.clone();
            send = true;
            *history_index = None;
        } else {
            if up_pressed {
                ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
                autocomplete.selected = autocomplete.selected.saturating_sub(1);
            }
            if down_pressed {
                ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
                if autocomplete.selected + 1 < matches.len() {
                    autocomplete.selected += 1;
                }
            }
        }
    } else {
        // Normal (non-autocomplete) key handling
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

    // After Tab autocomplete, keep focus on the input (prevent Tab focus traversal)
    if refocus {
        response.request_focus();
    }

    (send, response)
}
