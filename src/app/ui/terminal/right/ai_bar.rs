use crate::app::types::AppShared;
use crate::app::ui::workspace::state::WorkspaceState;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn render_ai_bar(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    combo_id: &'static str,
    i18n: &crate::i18n::I18n,
) {
    ui.horizontal(|ui| {
        // --- Agent picker ---
        let agents = {
            let sh = shared.lock().expect("lock");
            sh.registry.agents.get_all().to_vec()
        };

        ui.label(i18n.get("ai-label-assistant"));

        let selected_agent = agents.iter().find(|a| a.id == ws.selected_agent_id);
        let label = if let Some(agent) = selected_agent {
            agent.label.clone()
        } else {
            "—".to_string()
        };

        egui::ComboBox::from_id_salt(combo_id)
            .selected_text(label)
            .width(190.0)
            .show_ui(ui, |ui| {
                for agent in &agents {
                    ui.selectable_value(
                        &mut ws.selected_agent_id,
                        agent.id.clone(),
                        agent.label.clone(),
                    );
                }
            });

        let can_start = selected_agent.is_some();

        // Start button
        let start_response = ui.add_enabled(can_start, egui::Button::new(i18n.get("ai-btn-start")));
        if start_response.clicked()
            && let Some(agent) = selected_agent
        {
            let cmd = agent.command.clone();
            let active = ws.claude_active_tab;
            if let Some(terminal) = ws.claude_tabs.get_mut(active) {
                terminal.send_command(&cmd);
            }
        }

        ui.separator();
        ui.label(i18n.get("cli-chat-placeholder-model"));

        let preferred_model = {
            let sh = shared.lock().expect("lock");
            sh.settings.ai_default_model.clone()
        };
        let resolved_model = crate::app::ui::workspace::state::resolve_runtime_model(
            ws.available_ai_models(),
            ws.active_ai_model(),
            &preferred_model,
        );
        if resolved_model != ws.active_ai_model() {
            ws.set_active_ai_model(resolved_model.clone());
        }

        let model_combo_id = format!("{combo_id}_model");
        let selected_label = if resolved_model.is_empty() {
            "—".to_string()
        } else {
            resolved_model
        };

        egui::ComboBox::from_id_salt(model_combo_id)
            .selected_text(selected_label)
            .width(180.0)
            .show_ui(ui, |ui| {
                if ws.available_ai_models().is_empty() {
                    ui.label("No models");
                } else {
                    for model in ws.available_ai_models().to_vec() {
                        if ui
                            .selectable_label(ws.active_ai_model() == model, &model)
                            .clicked()
                        {
                            ws.set_active_ai_model(model);
                        }
                    }
                }
            });
    });
}
