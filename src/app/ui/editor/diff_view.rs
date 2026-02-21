use eframe::egui;
use similar::{ChangeTag, TextDiff};

pub fn render_diff_modal(
    ctx: &egui::Context,
    i18n: &crate::i18n::I18n,
    file_path: &str,
    original_text: &str,
    new_text: &str,
    font_size: f32,
    side_by_side: bool,
) -> Option<super::DiffAction> {
    let mut result = None;

    let modal = egui::Modal::new(egui::Id::new("ai_diff_modal"));

    modal.show(ctx, |ui| {
        ui.set_min_width(800.0);
        ui.set_min_height(600.0);

        ui.horizontal(|ui| {
            ui.heading(i18n.get("ai-diff-heading"));
            ui.add_space(8.0);
            ui.label(egui::RichText::new(file_path).strong());
        });
        ui.separator();

        let diff = TextDiff::from_lines(original_text, new_text);

        egui::ScrollArea::both()
            .id_salt("ai_diff_scroll")
            .max_height(500.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let font_id = egui::FontId::monospace(font_size);

                let bg_added = egui::Color32::from_rgba_unmultiplied(40, 100, 40, 100);
                let bg_removed = egui::Color32::from_rgba_unmultiplied(120, 30, 30, 100);
                let fg_added = egui::Color32::from_rgb(150, 255, 150);
                let fg_removed = egui::Color32::from_rgb(255, 150, 150);
                let fg_normal = ui.visuals().text_color();

                if side_by_side {
                    let available_width = ui.available_width();
                    let half_width = (available_width / 2.0) - 10.0; // padding

                    egui::Grid::new("side_by_side_diff_grid")
                        .num_columns(2)
                        .spacing([20.0, 0.0])
                        .show(ui, |ui| {
                            // Headers for columns
                            ui.label(egui::RichText::new("Original").strong().color(fg_removed));
                            ui.label(egui::RichText::new("Proposed").strong().color(fg_added));
                            ui.end_row();

                            for change in diff.iter_all_changes() {
                                let line_text = change.value();

                                let mut orig_job = egui::text::LayoutJob::default();
                                let mut new_job = egui::text::LayoutJob::default();

                                match change.tag() {
                                    ChangeTag::Delete => {
                                        orig_job.append(
                                            line_text,
                                            0.0,
                                            egui::text::TextFormat {
                                                font_id: font_id.clone(),
                                                color: fg_removed,
                                                background: bg_removed,
                                                ..Default::default()
                                            },
                                        );
                                        // Right side empty
                                    }
                                    ChangeTag::Insert => {
                                        // Left side empty
                                        new_job.append(
                                            line_text,
                                            0.0,
                                            egui::text::TextFormat {
                                                font_id: font_id.clone(),
                                                color: fg_added,
                                                background: bg_added,
                                                ..Default::default()
                                            },
                                        );
                                    }
                                    ChangeTag::Equal => {
                                        let fmt = egui::text::TextFormat {
                                            font_id: font_id.clone(),
                                            color: fg_normal,
                                            ..Default::default()
                                        };
                                        orig_job.append(line_text, 0.0, fmt.clone());
                                        new_job.append(line_text, 0.0, fmt);
                                    }
                                }

                                ui.allocate_ui(egui::vec2(half_width, 0.0), |ui| {
                                    ui.add(
                                        egui::Label::new(orig_job)
                                            .wrap_mode(egui::TextWrapMode::Extend),
                                    );
                                });
                                ui.allocate_ui(egui::vec2(half_width, 0.0), |ui| {
                                    ui.add(
                                        egui::Label::new(new_job)
                                            .wrap_mode(egui::TextWrapMode::Extend),
                                    );
                                });
                                ui.end_row();
                            }
                        });
                } else {
                    // Inline diff
                    for change in diff.iter_all_changes() {
                        let (bg_color, fg_color, prefix) = match change.tag() {
                            ChangeTag::Delete => (Some(bg_removed), fg_removed, "- "),
                            ChangeTag::Insert => (Some(bg_added), fg_added, "+ "),
                            ChangeTag::Equal => (None, fg_normal, "  "),
                        };

                        let line_text = change.value();

                        let mut layout_job = egui::text::LayoutJob::default();
                        layout_job.append(
                            &format!("{}{}", prefix, line_text),
                            0.0,
                            egui::text::TextFormat {
                                font_id: font_id.clone(),
                                color: fg_color,
                                background: bg_color.unwrap_or(egui::Color32::TRANSPARENT),
                                ..Default::default()
                            },
                        );

                        ui.add(egui::Label::new(layout_job).wrap_mode(egui::TextWrapMode::Extend));
                    }
                }
            });

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(i18n.get("btn-confirm")).clicked() {
                result = Some(super::DiffAction::Accepted);
            }
            if ui.button(i18n.get("btn-cancel")).clicked() {
                result = Some(super::DiffAction::Rejected);
            }
        });
    });

    result
}
