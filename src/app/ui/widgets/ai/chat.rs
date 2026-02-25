use crate::app::ai::{AiExpertiseRole, AiReasoningDepth};
use eframe::egui;

/// A unified UI widget for AI chat interfaces.
pub struct AiChatWidget;

impl AiChatWidget {
    /// Renders a multiline text edit with CLI-like behavior.
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
                .desired_rows(4),
        );

        (send, response)
    }

    /// Renders AI conversation history in a terminal-like style.
    pub fn ui_conversation(
        ui: &mut egui::Ui,
        conversation: &[(String, String)],
        font_size: f32,
        cache: &mut egui_commonmark::CommonMarkCache,
    ) {
        let terminal_bg = egui::Color32::from_rgb(20, 20, 25);
        let terminal_text = egui::Color32::from_rgb(175, 175, 175);
        let question_text = egui::Color32::from_rgb(70, 110, 160);
        let poly_color = egui::Color32::from_rgb(70, 110, 160);
        let credo_color = egui::Color32::from_rgb(70, 160, 110);
        let path_purple = egui::Color32::from_rgb(120, 80, 170);

        egui::Frame::new()
            .fill(terminal_bg)
            .inner_margin(8.0)
            .corner_radius(4.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(50)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("ai_chat_scroll")
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let path_re = regex::Regex::new(r"(?P<link>\[[^\]]+\]\([^\)]+\))|`(?P<code_inner>[^`]+)`|(?P<path>\b(?:src|locales|docs|app|ui|workspace|packaging|privacy|vendor|target)/[a-zA-Z0-9_\-./]+\.[a-z0-9]+\b|\b[a-zA-Z0-9_\-./]+\.(?:rs|toml|md|ftl|sh|json)\b)").ok();

                        for (q, a) in conversation {
                            // User Question
                            if !q.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(">>>").color(question_text).monospace().size(font_size));
                                    let mut q_mut = q.clone();
                                    ui.add(egui::TextEdit::multiline(&mut q_mut)
                                        .font(egui::FontId::monospace(font_size))
                                        .text_color(question_text)
                                        .code_editor()
                                        .lock_focus(false)
                                        .interactive(true)
                                        .desired_width(f32::INFINITY));
                                });
                                ui.add_space(4.0);
                            }

                            // Agent Answer
                            if !a.is_empty() {
                                if a.contains("____        __") && a.contains("CLI") {
                                    Self::render_logo(ui, a, font_size, poly_color, credo_color);
                                } else {
                                    Self::render_markdown(ui, a, font_size, terminal_text, path_purple, &path_re, cache);
                                }
                            }

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("📋 Copy Thread").clicked() {
                                        let full_text = if q.is_empty() { a.clone() } else { format!(">>> {}

{}", q, a) };
                                        ui.ctx().copy_text(full_text);
                                    }
                                });
                            });
                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);
                        }
                    });
            });
    }

    fn render_logo(
        ui: &mut egui::Ui,
        text: &str,
        font_size: f32,
        poly_color: egui::Color32,
        credo_color: egui::Color32,
    ) {
        let mut logo_line_idx = 0;
        for line in text.lines() {
            if line.contains("____")
                || line.contains(" / ")
                || line.contains("/_/")
                || line.contains("/____/")
            {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let split_point = match logo_line_idx {
                        0 => 25,
                        1 => 24,
                        2 => 23,
                        3 => 22,
                        4 => 21,
                        5 => 20,
                        _ => 22,
                    };
                    logo_line_idx += 1;
                    let actual_split = split_point.min(line.len());
                    let poly = &line[..actual_split];
                    let credo_full = &line[actual_split..];

                    ui.label(
                        egui::RichText::new(poly)
                            .color(poly_color)
                            .monospace()
                            .size(font_size),
                    );
                    if line.contains("CLI") {
                        let parts: Vec<&str> = credo_full.splitn(2, "CLI").collect();
                        ui.label(
                            egui::RichText::new(parts[0])
                                .color(credo_color)
                                .monospace()
                                .size(font_size),
                        );
                        ui.label(
                            egui::RichText::new("CLI")
                                .color(egui::Color32::from_rgb(110, 90, 0))
                                .monospace()
                                .size(font_size),
                        );
                        if parts.len() > 1 {
                            ui.label(
                                egui::RichText::new(parts[1])
                                    .color(credo_color)
                                    .monospace()
                                    .size(font_size),
                            );
                        }
                    } else {
                        ui.label(
                            egui::RichText::new(credo_full)
                                .color(credo_color)
                                .monospace()
                                .size(font_size),
                        );
                    }
                });
            } else if line.trim().starts_with("Version:")
                || line.trim().starts_with("Model:")
                || line.trim().starts_with("Rank:")
            {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        ui.label(
                            egui::RichText::new(parts[0])
                                .color(egui::Color32::from_gray(90))
                                .monospace()
                                .size(font_size),
                        );
                        ui.label(
                            egui::RichText::new(":")
                                .color(egui::Color32::from_gray(50))
                                .monospace()
                                .size(font_size),
                        );
                        ui.label(
                            egui::RichText::new(parts[1])
                                .color(egui::Color32::from_rgb(175, 175, 175))
                                .monospace()
                                .size(font_size),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(line)
                                .color(egui::Color32::from_rgb(175, 175, 175))
                                .monospace()
                                .size(font_size),
                        );
                    }
                });
            } else {
                ui.label(
                    egui::RichText::new(line)
                        .color(egui::Color32::from_rgb(175, 175, 175))
                        .monospace()
                        .size(font_size),
                );
            }
        }
    }

    fn render_markdown(
        ui: &mut egui::Ui,
        text: &str,
        font_size: f32,
        terminal_text: egui::Color32,
        path_purple: egui::Color32,
        path_re: &Option<regex::Regex>,
        cache: &mut egui_commonmark::CommonMarkCache,
    ) {
        ui.scope(|ui| {
            let md_font_size = font_size * 1.2;
            let style = ui.style_mut();
            style.visuals.widgets.noninteractive.fg_stroke.color = terminal_text;
            style.visuals.widgets.inactive.fg_stroke.color = terminal_text;
            style.visuals.widgets.active.fg_stroke.color = path_purple;
            style.visuals.hyperlink_color = path_purple;
            style.visuals.code_bg_color = egui::Color32::TRANSPARENT;

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

            style.spacing.item_spacing.y = 8.0;

            let mut current_block = String::new();
            let mut is_monologue_mode = false;

            let flush_block =
                |ui: &mut egui::Ui,
                 block: &mut String,
                 mono: bool,
                 cache: &mut egui_commonmark::CommonMarkCache| {
                    if block.is_empty() {
                        return;
                    }
                    let mut text = block.clone();
                    if let Some(re) = &path_re {
                        text = re
                            .replace_all(&text, |caps: &regex::Captures| {
                                if caps.name("link").is_some() {
                                    caps[0].to_string()
                                } else if let Some(c) = caps.name("code_inner") {
                                    format!("[{}](code)", c.as_str())
                                } else {
                                    format!("[{}](path)", &caps[0])
                                }
                            })
                            .to_string();
                    }

                    if mono {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            let (rect, _) =
                                ui.allocate_at_least(egui::vec2(2.0, 0.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 0.0, terminal_text);
                            egui::Frame::new()
                                .fill(egui::Color32::from_gray(35))
                                .inner_margin(egui::Margin::symmetric(12, 8))
                                .corner_radius(egui::CornerRadius {
                                    nw: 0,
                                    ne: 4,
                                    sw: 0,
                                    se: 4,
                                })
                                .show(ui, |ui| {
                                    egui_commonmark::CommonMarkViewer::new().show(ui, cache, &text);
                                });
                        });
                    } else {
                        egui_commonmark::CommonMarkViewer::new().show(ui, cache, &text);
                    }
                    block.clear();
                };

            for line in text.lines() {
                let trimmed = line.trim();
                let is_mono_line = trimmed.starts_with('>') || trimmed.starts_with("Step");

                if is_mono_line != is_monologue_mode && !current_block.is_empty() {
                    flush_block(ui, &mut current_block, is_monologue_mode, cache);
                }

                is_monologue_mode = is_mono_line;
                if is_mono_line {
                    let clean = line.replace('>', "").trim().to_string();
                    if clean.starts_with("Step") {
                        current_block.push_str(&format!(
                            "_{}_
",
                            clean
                        ));
                    } else {
                        current_block.push_str(&format!(
                            "{}
",
                            clean
                        ));
                    }
                } else {
                    current_block.push_str(line);
                    current_block.push('\n');
                }
            }
            flush_block(ui, &mut current_block, is_monologue_mode, cache);
        });
    }

    /// Renders the real-time "thinking" monologue.
    pub fn ui_monologue(
        ui: &mut egui::Ui,
        monologue: &[String],
        cache: &mut egui_commonmark::CommonMarkCache,
    ) {
        let path_purple = egui::Color32::from_rgb(120, 80, 170);
        let terminal_text = egui::Color32::from_rgb(175, 175, 175);

        ui.scope(|ui| {
            let style = ui.style_mut();
            style.visuals.widgets.noninteractive.fg_stroke.color = terminal_text;
            style.visuals.widgets.inactive.fg_stroke.color = terminal_text;
            style.visuals.widgets.active.fg_stroke.color = path_purple;
            style.visuals.hyperlink_color = path_purple;
            style.visuals.code_bg_color = egui::Color32::TRANSPARENT;

            let path_re = regex::Regex::new(r"(?P<link>\[[^\]]+\]\([^\)]+\))|`(?P<code_inner>[^`]+)`|(?P<path>\b(?:src|locales|docs|app|ui|workspace|packaging|privacy|vendor|target)/[a-zA-Z0-9_\-./]+\.[a-z0-9]+\b|\b[a-zA-Z0-9_\-./]+\.(?:rs|toml|md|ftl|sh|json)\b)").ok();

            let mut full_text = String::new();
            for line in monologue {
                let mut processed = line.clone();
                if let Some(re) = &path_re {
                    processed = re.replace_all(&processed, |caps: &regex::Captures| {
                        if caps.name("link").is_some() { caps[0].to_string() }
                        else if let Some(c) = caps.name("code_inner") { format!("[{}](code)", c.as_str()) }
                        else { format!("[{}](path)", &caps[0]) }
                    }).to_string();
                }

                let trimmed = processed.trim();
                if trimmed.starts_with("Step") {
                    full_text.push_str(&format!("_{}_
", trimmed.replace('>', "").trim()));
                } else {
                    full_text.push_str(&format!("│ {}
", trimmed.replace('>', "").trim()));
                }
            }

            if !full_text.is_empty() {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let (rect, _) = ui.allocate_at_least(egui::vec2(2.0, 0.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 0.0, terminal_text);

                    egui::Frame::new()
                        .fill(egui::Color32::from_gray(35))
                        .inner_margin(egui::Margin::symmetric(12, 12))
                        .corner_radius(egui::CornerRadius { nw: 0, ne: 4, sw: 0, se: 4 })
                        .show(ui, |ui| {
                            egui_commonmark::CommonMarkViewer::new().show(ui, cache, &full_text);
                        });
                });
            }
        });
    }

    /// Renders settings for an AI agent (Rank, Depth, Language).
    pub fn ui_settings(
        ui: &mut egui::Ui,
        expertise: &mut AiExpertiseRole,
        depth: &mut AiReasoningDepth,
        language: &mut String,
        system_prompt: &mut String,
        i18n: &crate::i18n::I18n,
    ) -> bool {
        let mut changed = false;
        ui.group(|ui| {
            ui.label(egui::RichText::new(i18n.get("gemini-settings-title")).strong());
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Rank:");
                egui::ComboBox::from_id_salt("ai_expertise")
                    .selected_text(expertise.as_str())
                    .show_ui(ui, |ui| {
                        changed |= ui
                            .selectable_value(expertise, AiExpertiseRole::Junior, "Junior")
                            .changed();
                        changed |= ui
                            .selectable_value(expertise, AiExpertiseRole::Senior, "Senior")
                            .changed();
                        changed |= ui
                            .selectable_value(expertise, AiExpertiseRole::Master, "Master")
                            .changed();
                    });

                ui.add_space(8.0);
                ui.label("Depth:");
                egui::ComboBox::from_id_salt("ai_depth")
                    .selected_text(depth.as_str())
                    .show_ui(ui, |ui| {
                        changed |= ui
                            .selectable_value(depth, AiReasoningDepth::Fast, "Fast")
                            .changed();
                        changed |= ui
                            .selectable_value(depth, AiReasoningDepth::Balanced, "Balanced")
                            .changed();
                        changed |= ui
                            .selectable_value(depth, AiReasoningDepth::Deep, "Deep")
                            .changed();
                    });

                ui.add_space(16.0);
                ui.label(i18n.get("gemini-label-language"));
                egui::ComboBox::from_id_salt("ai_lang")
                    .selected_text(crate::i18n::lang_display_name(language))
                    .show_ui(ui, |ui| {
                        for lang in crate::i18n::SUPPORTED_LANGS {
                            changed |= ui
                                .selectable_value(
                                    language,
                                    lang.to_string(),
                                    crate::i18n::lang_display_name(lang),
                                )
                                .changed();
                        }
                    });

                if ui
                    .button(i18n.get("gemini-btn-reset"))
                    .on_hover_text("Factory Reset")
                    .clicked()
                {
                    *system_prompt = i18n.get("gemini-default-prompt");
                    *language = i18n.lang().to_string();
                    changed = true;
                }
            });
            ui.add_space(4.0);
            ui.label(i18n.get("gemini-label-system-prompt"));
            changed |= ui
                .add(
                    egui::TextEdit::multiline(system_prompt)
                        .desired_width(f32::INFINITY)
                        .desired_rows(3),
                )
                .changed();
        });
        changed
    }
}
