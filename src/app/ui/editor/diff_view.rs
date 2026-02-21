use eframe::egui;
use similar::{ChangeTag, TextDiff};

pub struct DiffResult {
    pub accepted: bool,
    pub rejected: bool,
}

pub fn render_diff_modal(
    ctx: &egui::Context,
    i18n: &crate::i18n::I18n,
    file_path: &str,
    original_text: &str,
    new_text: &str,
    font_size: f32,
) -> DiffResult {
    let mut result = DiffResult {
        accepted: false,
        rejected: false,
    };

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

        // Diff calculation
        let diff = TextDiff::from_lines(original_text, new_text);

        egui::ScrollArea::vertical()
            .id_salt("ai_diff_scroll")
            .max_height(500.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let font_id = egui::FontId::monospace(font_size);

                // Set up visual style for diffs
                let bg_added = egui::Color32::from_rgba_unmultiplied(40, 100, 40, 100);
                let bg_removed = egui::Color32::from_rgba_unmultiplied(120, 30, 30, 100);

                let fg_added = egui::Color32::from_rgb(150, 255, 150);
                let fg_removed = egui::Color32::from_rgb(255, 150, 150);
                let fg_normal = ui.visuals().text_color();

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

                    // To make background span full width, we wrap the label in a horizontal layout
                    // or just paint the rect. Here we rely on LayoutJob background.
                    ui.add(egui::Label::new(layout_job).wrap_mode(egui::TextWrapMode::Extend));
                }
            });

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(i18n.get("btn-confirm")).clicked() {
                result.accepted = true;
            }
            if ui.button(i18n.get("btn-cancel")).clicked() {
                result.rejected = true;
            }
        });
    });

    result
}
