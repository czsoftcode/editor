use crate::app::ai::{AiExpertiseRole, AiReasoningDepth};
use eframe::egui;

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
        ui.label(egui::RichText::new(i18n.get("ai-chat-settings-title")).strong());
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
            ui.label(i18n.get("ai-chat-label-language"));
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
                .button(i18n.get("ai-chat-btn-reset"))
                .on_hover_text("Factory Reset")
                .clicked()
            {
                *system_prompt = i18n.get("ai-chat-default-prompt");
                *language = i18n.lang().to_string();
                changed = true;
            }
        });
        ui.add_space(4.0);
        ui.label(i18n.get("ai-chat-label-system-prompt"));
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
