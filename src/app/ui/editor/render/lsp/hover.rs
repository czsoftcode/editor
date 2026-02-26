use eframe::egui;

pub enum HoverSegment {
    /// Content of a fenced code block (``` ... ```).
    Code(String),
    /// Plain prose text outside code blocks.
    Prose(String),
}

pub fn parse_hover_segments(content: &str) -> Vec<HoverSegment> {
    let mut segments: Vec<HoverSegment> = Vec::new();
    let mut in_fence = false;
    let mut current: Vec<&str> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_fence {
                let code = current.join(
                    "
",
                );
                if !code.trim().is_empty() {
                    segments.push(HoverSegment::Code(code));
                }
                current.clear();
                in_fence = false;
            } else {
                let prose = current.join(
                    "
",
                );
                if !prose.trim().is_empty() {
                    segments.push(HoverSegment::Prose(prose));
                }
                current.clear();
                in_fence = true;
            }
        } else {
            current.push(line);
        }
    }

    if !current.is_empty() {
        let text = current.join(
            "
",
        );
        if !text.trim().is_empty() {
            if in_fence {
                segments.push(HoverSegment::Code(text));
            } else {
                segments.push(HoverSegment::Prose(text));
            }
        }
    }
    segments
}

pub fn hover_content_to_string(hover: &async_lsp::lsp_types::Hover) -> String {
    match &hover.contents {
        async_lsp::lsp_types::HoverContents::Markup(mc) => mc.value.clone(),
        async_lsp::lsp_types::HoverContents::Scalar(ms) => marked_string_value(ms),
        async_lsp::lsp_types::HoverContents::Array(arr) => arr
            .iter()
            .map(marked_string_value)
            .collect::<Vec<_>>()
            .join(
                "

",
            ),
    }
}

fn marked_string_value(ms: &async_lsp::lsp_types::MarkedString) -> String {
    match ms {
        async_lsp::lsp_types::MarkedString::String(s) => s.clone(),
        async_lsp::lsp_types::MarkedString::LanguageString(ls) => ls.value.clone(),
    }
}

pub fn render_hover_popup(ui: &egui::Ui, content: &str, screen_pos: egui::Pos2) {
    let popup_pos = screen_pos + egui::vec2(8.0, 16.0);
    egui::Area::new(egui::Id::new("lsp_hover_popup"))
        .fixed_pos(popup_pos)
        .order(egui::Order::Tooltip)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(35, 38, 46))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 80, 100)))
                .inner_margin(egui::Margin::same(10))
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.set_max_width(520.0);
                    egui::ScrollArea::vertical()
                        .id_salt("lsp_hover_scroll")
                        .max_height(260.0)
                        .show(ui, |ui| {
                            let segments = parse_hover_segments(content);
                            let mut first = true;
                            for seg in &segments {
                                if !first {
                                    ui.add_space(6.0);
                                    ui.separator();
                                    ui.add_space(4.0);
                                }
                                first = false;
                                match seg {
                                    HoverSegment::Code(code) => {
                                        let trimmed = code.trim();
                                        if !trimmed.is_empty() {
                                            ui.label(
                                                egui::RichText::new(trimmed)
                                                    .monospace()
                                                    .color(egui::Color32::from_rgb(180, 210, 255)),
                                            );
                                        }
                                    }
                                    HoverSegment::Prose(text) => {
                                        let trimmed = text.trim();
                                        if !trimmed.is_empty() {
                                            ui.label(
                                                egui::RichText::new(trimmed)
                                                    .color(egui::Color32::from_rgb(200, 200, 200)),
                                            );
                                        }
                                    }
                                }
                            }
                        });
                });
        });
}
