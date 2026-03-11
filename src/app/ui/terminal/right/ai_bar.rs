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
    });
}
