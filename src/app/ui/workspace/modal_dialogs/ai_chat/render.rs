use super::AiModalAction;
use super::approval::render_approval_ui;
use crate::app::types::AppShared;
use crate::app::ui::widgets::ai::AiChatWidget;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn check_agent_authorization(
    shared: &Arc<Mutex<AppShared>>,
    ws: &mut WorkspaceState,
    ui: &mut egui::Ui,
) {
    let pending_auth = {
        let sh = shared.lock().expect("lock");
        sh.registry
            .plugins
            .get_pending_authorizations()
            .into_iter()
            .find(|(id, _)| id == "gemini")
    };

    if let Some((id, _meta)) = pending_auth {
        let (plugin_manager, config): (Arc<crate::app::registry::plugins::PluginManager>, _) = {
            let sh = shared.lock().expect("lock");
            (
                Arc::clone(&sh.registry.plugins),
                sh.settings
                    .plugins
                    .get(&id)
                    .map(|s| s.config.clone())
                    .unwrap_or_default(),
            )
        };

        if let Err(e) = plugin_manager.authorize(&id, &config) {
            ws.plugin_error = Some(format!("Auto-authorization failed: {}", e));
        }
        ui.ctx().request_repaint();
    }
}

pub fn render_chat_main_ui(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    i18n: &I18n,
    prompt: &mut String,
    font_size: f32,
    action: &mut Option<AiModalAction>,
    loading: bool,
) {
    let viewer_bg = egui::Color32::from_rgb(20, 20, 25);
    // Normal Top-Down layout
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        // Calculate reserved space for bottom elements to limit history growth
        let mut bottom_h = 110.0; // Base for Info + Prompt
        if ws.gemini_show_settings {
            bottom_h += 220.0;
        }
        if ws.pending_plugin_approval.is_some() {
            bottom_h += 300.0;
        }

        let max_history_h = ui.available_height() - bottom_h;

        // 1. RESULTS / HISTORY AREA (Grows with content)
        egui::ScrollArea::vertical()
            .id_salt("ai_history_main_scroll")
            .auto_shrink([false, true])
            .max_height(max_history_h.max(100.0))
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                if !ws.gemini_conversation.is_empty() {
                    AiChatWidget::ui_conversation(
                        ui,
                        &ws.gemini_conversation,
                        font_size,
                        &mut ws.markdown_cache,
                    );
                } else if loading {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(egui::RichText::new(i18n.get("gemini-loading")).strong());
                    });
                    ui.add_space(4.0);
                    AiChatWidget::ui_monologue(ui, &ws.gemini_monologue, &mut ws.markdown_cache);
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("PolyCredo AI").weak().size(24.0));
                    });
                }
            });

        // 2. CONTROLS (Info Bar + Prompt)
        if let Some((id, action_name, details, sender)) = ws.pending_plugin_approval.take() {
            render_approval_ui(ui, id, action_name, details, sender, ws);
        } else {
            // SEPARATOR
            ui.add_space(4.0);
            ui.scope(|ui| {
                ui.visuals_mut().widgets.noninteractive.bg_stroke =
                    egui::Stroke::new(1.0, egui::Color32::from_gray(60));
                ui.separator();
            });

            // INFO BAR
            egui::Frame::new()
                .fill(viewer_bg)
                .inner_margin(egui::Margin::symmetric(8, 2))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    render_info_bar(ui, ws, loading);
                });

            ui.add_space(4.0);

            // PROMPT
            let prompt_bg = egui::Color32::from_rgb(45, 55, 65);
            let text_color = egui::Color32::from_rgb(200, 200, 200);

            egui::Frame::new()
                .fill(prompt_bg)
                .inner_margin(egui::Margin::symmetric(8, 2))
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let visuals = ui.visuals_mut();
                    visuals.override_text_color = Some(text_color);
                    visuals.selection.stroke = egui::Stroke::NONE;
                    visuals.extreme_bg_color = prompt_bg;
                    visuals.widgets.hovered.expansion = 0.0;
                    visuals.widgets.active.expansion = 0.0;

                    if loading && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        ws.gemini_cancellation_token
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                    }

                    let (send_via_kb, resp) = AiChatWidget::ui_input(
                        ui,
                        prompt,
                        font_size,
                        &i18n.get("gemini-placeholder-prompt"),
                        &ws.gemini_history,
                        &mut ws.gemini_history_index,
                    );
                    if ws.gemini_focus_requested {
                        resp.request_focus();
                        ws.gemini_focus_requested = false;
                    }
                    if send_via_kb {
                        *action = Some(AiModalAction::Send);
                    }
                });
        }

        // 4. SETTINGS
        if ws.gemini_show_settings {
            ui.add_space(8.0);
            let settings_bg = egui::Color32::from_rgb(30, 35, 45);
            egui::Frame::new()
                .fill(settings_bg)
                .inner_margin(12.0)
                .corner_radius(4.0)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(60)))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if AiChatWidget::ui_settings(
                        ui,
                        &mut ws.gemini_expertise,
                        &mut ws.gemini_reasoning_depth,
                        &mut ws.gemini_language,
                        &mut ws.gemini_system_prompt,
                        i18n,
                    ) {
                        // Settings changed
                    }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(format!("✔ {}", i18n.get("btn-save"))).clicked() {
                            *action = Some(AiModalAction::SaveSettings);
                        }
                        if ui.button(i18n.get("btn-close")).clicked() {
                            ws.gemini_show_settings = false;
                        }
                    });
                });
        }
    });
}

pub fn render_info_bar(ui: &mut egui::Ui, ws: &WorkspaceState, loading: bool) {
    ui.horizontal(|ui| {
        if loading {
            ui.spinner();
        } else {
            ui.label("📁");
        }
        ui.label(egui::RichText::new(ws.sandbox.root.to_string_lossy()).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "In: {} | Out: {}",
                    ws.gemini_in_tokens, ws.gemini_out_tokens
                ))
                .weak(),
            );
        });
    });
}
