use eframe::egui;

pub fn render_markdown(
    ui: &mut egui::Ui,
    text: &str,
    font_size: f32,
    terminal_text: egui::Color32,
    path_purple: egui::Color32,
    path_re: &Option<regex::Regex>,
    cache: &mut egui_commonmark::CommonMarkCache,
) {
    ui.scope(|ui| {
        let avail_width = ui.available_width();
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

        style.spacing.item_spacing.y = 4.0; // Tighter paragraph spacing

        let mut current_block = String::new();
        let mut is_monologue_mode = false;

        for line in text.lines() {
            let trimmed = line.trim();
            let is_step = trimmed.contains("Step")
                && (trimmed.starts_with("Step")
                    || trimmed.starts_with('>')
                    || trimmed.starts_with('_'));
            let is_mono_line = trimmed.starts_with('>') || trimmed.starts_with("Step");

            // If switching between mono/normal or encountering a Step, flush the block
            if (is_mono_line != is_monologue_mode || is_step) && !current_block.is_empty() {
                flush_block(
                    ui,
                    &mut current_block,
                    is_monologue_mode,
                    avail_width,
                    terminal_text,
                    path_re,
                    cache,
                );
            }

            if is_step {
                let clean_step = trimmed.replace(['>', '_'], "").trim().to_string();
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(clean_step)
                        .size(font_size * 0.8)
                        .italics()
                        .color(terminal_text.gamma_multiply(0.7)),
                );
                ui.add_space(2.0);
                is_monologue_mode = true; // Steps are always within the monologue flow
            } else {
                is_monologue_mode = is_mono_line;
                if is_mono_line {
                    let clean = line.replace('>', "").trim().to_string();
                    if clean.is_empty() {
                        current_block.push('\n');
                    } else {
                        current_block.push_str(&clean);
                        current_block.push('\n');
                    }
                } else {
                    current_block.push_str(line);
                    current_block.push('\n');
                }
            }
        }
        flush_block(
            ui,
            &mut current_block,
            is_monologue_mode,
            avail_width,
            terminal_text,
            path_re,
            cache,
        );
    });
}

fn flush_block(
    ui: &mut egui::Ui,
    block: &mut String,
    mono: bool,
    width: f32,
    terminal_text: egui::Color32,
    path_re: &Option<regex::Regex>,
    cache: &mut egui_commonmark::CommonMarkCache,
) {
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
            let (rect, _) = ui.allocate_at_least(egui::vec2(2.0, 0.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 0.0, terminal_text);
            egui::Frame::new()
                .fill(egui::Color32::TRANSPARENT)
                .inner_margin(egui::Margin::symmetric(12, 8))
                .corner_radius(egui::CornerRadius {
                    nw: 0,
                    ne: 4,
                    sw: 0,
                    se: 4,
                })
                .show(ui, |ui| {
                    ui.set_width(width - 2.0);
                    egui_commonmark::CommonMarkViewer::new().show(ui, cache, &text);
                });
        });
    } else {
        ui.set_width(width);
        egui_commonmark::CommonMarkViewer::new().show(ui, cache, &text);
    }
    block.clear();
}

pub fn ui_monologue(
    ui: &mut egui::Ui,
    monologue: &[String],
    cache: &mut egui_commonmark::CommonMarkCache,
) {
    let path_purple = egui::Color32::from_rgb(120, 80, 170);
    let terminal_text = egui::Color32::from_rgb(175, 175, 175);

    if monologue.is_empty() {
        return;
    }

    ui.scope(|ui| {
        let base_font_size = ui.style().text_styles[&egui::TextStyle::Body].size;
        let style = ui.style_mut();
        style.visuals.widgets.noninteractive.fg_stroke.color = terminal_text;
        style.visuals.widgets.inactive.fg_stroke.color = terminal_text;
        style.visuals.widgets.active.fg_stroke.color = path_purple;
        style.visuals.hyperlink_color = path_purple;
        style.visuals.code_bg_color = egui::Color32::TRANSPARENT;

        let path_re = regex::Regex::new(r"(?P<link>\[[^\]]+\]\([^\)]+\))|`(?P<code_inner>[^`]+)`|(?P<path>\b(?:src|locales|docs|app|ui|workspace|packaging|privacy|vendor|target)/[a-zA-Z0-9_\-./]+\.[a-z0-9]+\b|\b[a-zA-Z0-9_\-./]+\.(?:rs|toml|md|ftl|sh|json)\b)").ok();

        let avail_width = ui.available_width();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            // 1. Sidebar line
            let (rect, _) = ui.allocate_at_least(egui::vec2(2.0, 0.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 0.0, terminal_text);

            // 2. Content
            egui::Frame::new()
                .fill(egui::Color32::TRANSPARENT)
                .inner_margin(egui::Margin::symmetric(12, 10))
                .corner_radius(egui::CornerRadius {
                    nw: 0,
                    ne: 4,
                    sw: 0,
                    se: 4,
                })
                .show(ui, |ui| {
                    ui.set_width(avail_width - 10.0);
                    ui.vertical(|ui| {
                        let mut current_thought = String::new();

                        let mut flush_thought = |ui: &mut egui::Ui, text: &mut String| {
                            if !text.is_empty() {
                                egui_commonmark::CommonMarkViewer::new().show(
                                    ui,
                                    cache,
                                    text.trim(),
                                );
                                text.clear();
                            }
                        };

                                                    for entry in monologue {
                                                        let mut processed = entry.clone();
                                                        if let Some(re) = &path_re {
                                                            processed = re
                                                                .replace_all(&processed, |caps: &regex::Captures| {
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

                                                        for line in processed.lines() {
                                                            let trimmed = line.trim();
                                                            if trimmed.is_empty() {
                                                                current_thought.push('\n');
                                                                continue;
                                                            }

                                                            if trimmed.contains("Step")
                                                                && (trimmed.starts_with("Step")
                                                                    || trimmed.starts_with('>')
                                                                    || trimmed.starts_with('_'))
                                                            {
                                                                flush_thought(ui, &mut current_thought);
                                                                let clean_step =
                                                                    trimmed.replace(['>', '_'], "").trim().to_string();
                                                                ui.add_space(2.0);
                                                                ui.label(
                                                                    egui::RichText::new(clean_step)
                                                                        .size(base_font_size * 0.8)
                                                                        .italics()
                                                                        .color(terminal_text.gamma_multiply(0.7)),
                                                                );
                                                                                                                                ui.add_space(2.0);
                                                                                                                            } else {
                                                                                                                                let content =

                                                                    trimmed.strip_prefix('>').unwrap_or(trimmed).trim();
                                                                if !content.is_empty() {
                                                                    current_thought.push_str(content);
                                                                    current_thought.push('\n');
                                                                }
                                                            }
                                                        }
                                                    }

                        flush_thought(ui, &mut current_thought);
                    });
                });
        });
    });
}
