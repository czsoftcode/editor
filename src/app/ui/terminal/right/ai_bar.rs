use crate::app::types::AppShared;
use crate::app::ai::state::OllamaConnectionStatus;
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
        // --- Ollama status icon ---
        let (color, tooltip) = match ws.ai.ollama.status {
            OllamaConnectionStatus::Connected => (
                egui::Color32::from_rgb(0, 180, 0),
                "Ollama: connected",
            ),
            OllamaConnectionStatus::Disconnected => (
                egui::Color32::from_rgb(220, 50, 50),
                "Ollama: disconnected",
            ),
            OllamaConnectionStatus::Checking => (
                egui::Color32::GRAY,
                "Ollama: checking...",
            ),
        };
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(12.0, 12.0),
            egui::Sense::hover(),
        );
        ui.painter().circle_filled(rect.center(), 5.0, color);
        response.on_hover_text(tooltip);

        ui.add_space(4.0);

        // --- Ollama model ComboBox ---
        if ws.ai.ollama.models.is_empty() {
            let resp = ui.add_enabled(
                false,
                egui::Button::new("No models available"),
            );
            resp.on_disabled_hover_text("Run 'ollama pull <model>' to download a model");
        } else {
            egui::ComboBox::from_id_salt("ollama_model_picker")
                .selected_text(&ws.ai.ollama.selected_model)
                .width(160.0)
                .show_ui(ui, |ui| {
                    for model in &ws.ai.ollama.models {
                        ui.selectable_value(
                            &mut ws.ai.ollama.selected_model,
                            model.clone(),
                            model,
                        );
                    }
                });
        }

        ui.separator();

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
            i18n.get("plugins-unknown-agent")
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
    });
}
