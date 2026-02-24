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
                                if a.contains("____        __") && a.contains("CLI") {
                                    // Special rendering for the logo with per-line split points
                                    let mut logo_line_idx = 0;
                                    for line in a.lines() {
                                        if line.contains("____")
                                            || line.contains(" / ")
                                            || line.contains("/_/")
                                            || line.contains("/____/")
                                        {
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 0.0;

                                                // Calculate split point based on line index to follow the slant
                                                // These points mark the exact gap between Poly and Credo
                                                let split_point = match logo_line_idx {
                                                    0 => 25, // Top line
                                                    1 => 24,
                                                    2 => 23,
                                                    3 => 22,
                                                    4 => 21,
                                                    5 => 20, // Tail line (/____/)
                                                    _ => 22,
                                                };
                                                logo_line_idx += 1;

                                                let actual_split = split_point.min(line.len());
                                                let poly = &line[..actual_split];
                                                let credo_full = &line[actual_split..];

                                                ui.label(
                                                    egui::RichText::new(poly)
                                                        .color(egui::Color32::from_rgb(
                                                            100, 180, 255,
                                                        ))
                                                        .monospace()
                                                        .size(font_size),
                                                );

                                                if line.contains("CLI") {
                                                    // Robustly split "CLI" and color it gold
                                                    let parts: Vec<&str> =
                                                        credo_full.splitn(2, "CLI").collect();
                                                    ui.label(
                                                        egui::RichText::new(parts[0])
                                                            .color(egui::Color32::from_rgb(
                                                                100, 255, 180,
                                                            ))
                                                            .monospace()
                                                            .size(font_size),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new("CLI")
                                                            .color(egui::Color32::from_rgb(
                                                                255, 215, 0,
                                                            ))
                                                            .monospace()
                                                            .size(font_size),
                                                    );
                                                    if parts.len() > 1 {
                                                        ui.label(
                                                            egui::RichText::new(parts[1])
                                                                .color(egui::Color32::from_rgb(
                                                                    100, 255, 180,
                                                                ))
                                                                .monospace()
                                                                .size(font_size),
                                                        );
                                                    }
                                                } else {
                                                    ui.label(
                                                        egui::RichText::new(credo_full)
                                                            .color(egui::Color32::from_rgb(
                                                                100, 255, 180,
                                                            ))
                                                            .monospace()
                                                            .size(font_size),
                                                    );
                                                }
                                            });
                                        } else if line.trim().starts_with("Version:")
                                            || line.trim().starts_with("Model:")
                                            || line.trim().starts_with("Plan:")
                                        {
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 0.0;
                                                let parts: Vec<&str> =
                                                    line.splitn(2, ':').collect();
                                                if parts.len() == 2 {
                                                    ui.label(
                                                        egui::RichText::new(parts[0])
                                                            .color(egui::Color32::from_rgb(
                                                                130, 130, 130,
                                                            ))
                                                            .monospace()
                                                            .size(font_size),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(":")
                                                            .color(egui::Color32::from_gray(100))
                                                            .monospace()
                                                            .size(font_size),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(parts[1])
                                                            .color(egui::Color32::from_rgb(
                                                                200, 200, 200,
                                                            ))
                                                            .monospace()
                                                            .size(font_size),
                                                    ); // Soft silver for values
                                                } else {
                                                    ui.label(
                                                        egui::RichText::new(line)
                                                            .color(terminal_text)
                                                            .monospace()
                                                            .size(font_size),
                                                    );
                                                }
                                            });
                                        } else {
                                            ui.label(
                                                egui::RichText::new(line)
                                                    .color(terminal_text)
                                                    .monospace()
                                                    .size(font_size),
                                            );
                                        }
                                    }
                                } else {
                                    ui.scope(|ui| {
                                        // Respect the current font size settings for all markdown elements
                                        // We increase the base size to 120% for better emphasis and readability
                                        let md_font_size = font_size * 1.2;

                                        let style = ui.style_mut();
                                        let text_styles = &mut style.text_styles;
                                        text_styles.insert(
                                            egui::TextStyle::Body,
                                            egui::FontId::proportional(md_font_size),
                                        );
                                        text_styles.insert(
                                            egui::TextStyle::Monospace,
                                            egui::FontId::monospace(md_font_size),
                                        );
                                        text_styles.insert(
                                            egui::TextStyle::Button,
                                            egui::FontId::proportional(md_font_size),
                                        );
                                        text_styles.insert(
                                            egui::TextStyle::Heading,
                                            egui::FontId::proportional(md_font_size * 1.25),
                                        );

                                        // Increase spacing between blocks for better readability
                                        style.spacing.item_spacing.y = 12.0;

                                        egui_commonmark::CommonMarkViewer::new()
                                            .max_image_width(Some(512))
                                            .show(ui, _cache, a);
                                    });
                                }
                            }

                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(12.0);
                        }
                    });
            });
    }
}
