use crate::app::ai_core::{AiExpertiseRole, AiReasoningDepth};
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
        ui.label(egui::RichText::new(i18n.get("cli-chat-settings-title")).strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(i18n.get("cli-chat-label-rank"));
            let expertise_text = match expertise {
                AiExpertiseRole::Junior => i18n.get("cli-chat-rank-junior"),
                AiExpertiseRole::Senior => i18n.get("cli-chat-rank-senior"),
                AiExpertiseRole::Master => i18n.get("cli-chat-rank-master"),
            };
            egui::ComboBox::from_id_salt("ai_expertise")
                .selected_text(expertise_text)
                .show_ui(ui, |ui| {
                    changed |= ui
                        .selectable_value(
                            expertise,
                            AiExpertiseRole::Junior,
                            i18n.get("cli-chat-rank-junior"),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            expertise,
                            AiExpertiseRole::Senior,
                            i18n.get("cli-chat-rank-senior"),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            expertise,
                            AiExpertiseRole::Master,
                            i18n.get("cli-chat-rank-master"),
                        )
                        .changed();
                });

            ui.add_space(8.0);
            ui.label(i18n.get("cli-chat-label-depth"));
            let depth_text = match depth {
                AiReasoningDepth::Fast => i18n.get("cli-chat-depth-fast"),
                AiReasoningDepth::Balanced => i18n.get("cli-chat-depth-balanced"),
                AiReasoningDepth::Deep => i18n.get("cli-chat-depth-deep"),
            };
            egui::ComboBox::from_id_salt("ai_depth")
                .selected_text(depth_text)
                .show_ui(ui, |ui| {
                    changed |= ui
                        .selectable_value(
                            depth,
                            AiReasoningDepth::Fast,
                            i18n.get("cli-chat-depth-fast"),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            depth,
                            AiReasoningDepth::Balanced,
                            i18n.get("cli-chat-depth-balanced"),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            depth,
                            AiReasoningDepth::Deep,
                            i18n.get("cli-chat-depth-deep"),
                        )
                        .changed();
                });

            ui.add_space(16.0);
            ui.label(i18n.get("cli-chat-label-language"));
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
                .button(i18n.get("cli-chat-btn-reset"))
                .on_hover_text("Factory Reset")
                .clicked()
            {
                *system_prompt = i18n.get("cli-chat-default-prompt");
                *language = i18n.lang().to_string();
                changed = true;
            }
        });
        ui.add_space(4.0);
        ui.label(i18n.get("cli-chat-label-system-prompt"));
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
