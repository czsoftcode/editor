use crate::app::ai::{AiContextPayload, AiManager};
use crate::app::types::AppShared;
use crate::app::ui::workspace::state::WorkspaceState;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn format_context_for_terminal(ctx: &AiContextPayload) -> String {
    let mut s = String::new();
    s.push_str(
        "Context info (paths are relative to current working directory):
",
    );

    if !ctx.memory_keys.is_empty() {
        s.push_str("Long-term memory keys: ");
        s.push_str(&ctx.memory_keys.join(", "));
        s.push('\n');
    }

    if !ctx.open_files.is_empty() {
        s.push_str(
            "Open files:
",
        );
        for file in &ctx.open_files {
            let active = if file.is_active { " (active)" } else { "" };
            s.push_str(&format!(
                "- {}{}
",
                file.path, active
            ));
        }
    }

    if !ctx.build_errors.is_empty() {
        s.push_str(
            "
Build errors:
",
        );
        for err in &ctx.build_errors {
            let level = if err.is_warning { "Warning" } else { "Error" };
            s.push_str(&format!(
                "[{}] {}:{}: {}
",
                level, err.file, err.line, err.message
            ));
        }
    } else {
        s.push_str(
            "
Build is clean.
",
        );
    }
    s
}

pub fn render_ai_bar(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    combo_id: &'static str,
    i18n: &crate::i18n::I18n,
) {
    ui.horizontal(|ui| {
        let agents = {
            let sh = shared.lock().expect("lock");
            sh.registry.agents.get_all().to_vec()
        };

        ui.label(i18n.get("ai-label-assistant"));

        // Picker
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
            let plan = ws.sandbox.get_sync_plan();
            if plan.is_empty() {
                let cmd = agent.command.clone();
                let active = ws.claude_active_tab;
                let context = format_context_for_terminal(&AiManager::generate_context(ws, shared));
                if let Some(terminal) = ws.claude_tabs.get_mut(active) {
                    terminal.send_command(&cmd);
                    if agent.context_aware {
                        terminal.send_command(&context);
                    }
                }
            } else {
                ws.sync_confirmation = Some(plan);
                ws.pending_agent_id = Some(agent.id.clone());
            }
        }

        // Sync button
        let active_tab = ws.claude_tabs.get(ws.claude_active_tab);
        let can_sync = active_tab.map(|t| !t.is_exited()).unwrap_or(false);
        if ui
            .add_enabled(can_sync, egui::Button::new(i18n.get("ai-btn-sync")))
            .on_hover_text(i18n.get("ai-hover-sync"))
            .clicked()
            && let Some(agent) = selected_agent
            && agent.context_aware
        {
            let context = format_context_for_terminal(&AiManager::generate_context(ws, shared));
            if let Some(terminal) = ws.claude_tabs.get_mut(ws.claude_active_tab) {
                terminal.send_command(&context);
            }
        }
    });
}
