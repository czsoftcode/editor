use super::render::render_markdown;
use eframe::egui;

pub fn ui_conversation(
    ui: &mut egui::Ui,
    conversation: &[(String, String)],
    font_size: f32,
    cache: &mut egui_commonmark::CommonMarkCache,
) {
    let poly_color = egui::Color32::from_rgb(100, 160, 220);
    let credo_color = egui::Color32::from_rgb(100, 220, 160);
    let terminal_text = egui::Color32::from_rgb(175, 175, 175);
    let path_purple = egui::Color32::from_rgb(120, 80, 170);

    let path_re = regex::Regex::new(r"(?P<link>\[[^\]]+\]\([^\)]+\))|`(?P<code_inner>[^`]+)`|(?P<path>\b(?:src|locales|docs|app|ui|workspace|packaging|privacy|vendor|target)/[a-zA-Z0-9_\-./]+\.[a-z0-9]+\b|\b[a-zA-Z0-9_\-./]+\.(?:rs|toml|md|ftl|sh|json)\b)").ok();

    for (i, (q, a)) in conversation.iter().enumerate() {
        // User Question (Styled to match the input prompt)
        if !q.is_empty() {
            let prompt_bg = egui::Color32::from_rgb(50, 60, 75);
            let text_color = egui::Color32::from_rgb(200, 200, 200);

            egui::Frame::new()
                .fill(prompt_bg)
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
                                .frame(false) // No internal frame
                                .lock_focus(false)
                                .interactive(true)
                                .desired_width(f32::INFINITY),
                        );
                    });
                });
            ui.add_space(4.0);
        }

        // Agent Answer
        if !a.is_empty() {
            if a.contains("____        __") && a.contains("CLI") {
                render_logo(ui, a, font_size, poly_color, credo_color);
            } else {
                render_markdown(
                    ui,
                    a,
                    font_size,
                    terminal_text,
                    path_purple,
                    &path_re,
                    cache,
                );
            }
        }

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("📋 Copy Thread").clicked() {
                    let full_text = if q.is_empty() {
                        a.clone()
                    } else {
                        format!(
                            ">>> {}

{}",
                            q, a
                        )
                    };
                    ui.ctx().copy_text(full_text);
                }
            });
        });
        ui.add_space(4.0);

        // Visible separator
        if i + 1 < conversation.len() {
            ui.scope(|ui| {
                ui.visuals_mut().widgets.noninteractive.bg_stroke =
                    egui::Stroke::new(1.0, egui::Color32::from_gray(60));
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
