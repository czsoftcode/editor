use super::render::render_markdown;
use eframe::egui;

pub fn ui_conversation(
    ui: &mut egui::Ui,
    conversation: &[(String, String)],
    font_size: f32,
    cache: &mut egui_commonmark::CommonMarkCache,
    model_name: &str,
    out_tokens: u32,
    is_streaming: bool,
) {
    let poly_color = egui::Color32::from_rgb(100, 160, 220);
    let credo_color = egui::Color32::from_rgb(100, 220, 160);

    let path_re = regex::Regex::new(r"(?P<link>\[[^\]]+\]\([^\)]+\))|`(?P<code_inner>[^`]+)`|(?P<path>\b(?:src|locales|docs|app|ui|workspace|packaging|privacy|vendor|target)/[a-zA-Z0-9_\-./]+\.[a-z0-9]+\b|\b[a-zA-Z0-9_\-./]+\.(?:rs|toml|md|ftl|sh|json)\b)").ok();

    // Format current local time as HH:MM
    let timestamp = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as libc::time_t;
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        unsafe { libc::localtime_r(&secs, &mut tm) };
        format!("{:02}:{:02}", tm.tm_hour, tm.tm_min)
    };

    let total = conversation.len();

    for (i, (q, a)) in conversation.iter().enumerate() {
        let is_last = i + 1 == total;

        // User Question
        if !q.is_empty() {
            let user_bg = ui.visuals().faint_bg_color;
            let text_color = ui.visuals().text_color();
            let weak_color = ui.visuals().weak_text_color();

            // Metadata bar
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("You")
                        .color(weak_color)
                        .small()
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(&timestamp)
                        .color(weak_color)
                        .small(),
                );
            });

            egui::Frame::new()
                .fill(user_bg)
                .inner_margin(egui::Margin::symmetric(8, 4))
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("->")
                                .color(text_color)
                                .monospace()
                                .size(font_size),
                        );
                        let mut q_mut = q.clone();
                        ui.add(
                            egui::TextEdit::multiline(&mut q_mut)
                                .font(egui::FontId::monospace(font_size))
                                .text_color(text_color)
                                .code_editor()
                                .frame(false)
                                .lock_focus(false)
                                .interactive(true)
                                .desired_width(ui.available_width()),
                        );
                    });
                });
            // Copy button after the message block
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Copy").clicked() {
                        ui.ctx().copy_text(q.clone());
                    }
                });
            });
            ui.add_space(4.0);
        }

        // Agent Answer
        if !a.is_empty() {
            let ai_bg = ui.visuals().faint_bg_color;
            let weak_color = ui.visuals().weak_text_color();

            // Metadata bar
            ui.horizontal(|ui| {
                let role_label = if model_name.is_empty() { "AI" } else { model_name };
                ui.label(
                    egui::RichText::new(role_label)
                        .color(weak_color)
                        .small()
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(&timestamp)
                        .color(weak_color)
                        .small(),
                );
                // Token count only for the last AI response
                if is_last && out_tokens > 0 {
                    ui.label(
                        egui::RichText::new(format!("{} tokens", out_tokens))
                            .color(weak_color)
                            .small(),
                    );
                }
                if is_last && is_streaming {
                    ui.spinner();
                }
            });

            if a.contains("____        __") && a.contains("CLI") {
                render_logo(ui, a, font_size, poly_color, credo_color);
            } else {
                egui::Frame::new()
                    .fill(ai_bg)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        render_markdown(
                            ui,
                            a,
                            font_size,
                            &path_re,
                            cache,
                        );
                    });
            }
            // Copy button after the AI message block
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Copy").clicked() {
                        let full_text = if q.is_empty() {
                            a.clone()
                        } else {
                            format!(">>> {}\n\n{}", q, a)
                        };
                        ui.ctx().copy_text(full_text);
                    }
                });
            });
        }

        ui.add_space(4.0);

        // Visible separator
        if i + 1 < conversation.len() {
            ui.scope(|ui| {
                ui.visuals_mut().widgets.noninteractive.bg_stroke =
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
                ui.separator();
            });
            ui.add_space(4.0);
        }
    }
}

pub fn render_logo(
    ui: &mut egui::Ui,
    text: &str,
    font_size: f32,
    poly_color: egui::Color32,
    credo_color: egui::Color32,
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0; // Tight vertical spacing for ASCII art
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
                                .color(egui::Color32::from_rgb(210, 180, 50))
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
    });
}
