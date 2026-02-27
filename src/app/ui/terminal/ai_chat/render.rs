use super::approval::render_approval_ui;
use super::inspector::render_inspector;
use super::AiChatAction;
use crate::app::types::AppShared;
use crate::app::ui::widgets::ai::AiChatWidget;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

// ── HEAD ─────────────────────────────────────────────────────────────────────

/// Checks whether the selected AI plugin still needs authorization and triggers
/// it automatically. Shows nothing visible when no authorization is pending.
pub fn render_head(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
) {
    let pending_auth = {
        let sh = shared.lock().expect("lock");
        sh.registry
            .plugins
            .get_pending_authorizations()
            .into_iter()
            .find(|(id, _)| id == &ws.ai_selected_provider)
    };

    if let Some((id, _meta)) = pending_auth {
        let (plugin_manager, config) = {
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

// ── BODY ─────────────────────────────────────────────────────────────────────

/// Renders the main chat area. When the inspector is open the body splits
/// horizontally: chat on the left, inspector on the right.
pub fn render_body(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    i18n: &I18n,
    prompt: &mut String,
    font_size: f32,
    loading: bool,
    body_h: f32,
) -> Option<AiChatAction> {
    if ws.ai_inspector_open {
        let mut action = None;
        ui.horizontal_top(|ui| {
            let left_w = (ui.available_width() * 0.55).max(400.0);
            ui.allocate_ui_with_layout(
                egui::vec2(left_w, body_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    action = render_chat_content(ui, ws, i18n, prompt, font_size, loading, body_h);
                },
            );
            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), body_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    render_inspector(ui, ws, font_size);
                },
            );
        });
        action
    } else {
        render_chat_content(ui, ws, i18n, prompt, font_size, loading, body_h)
    }
}

/// Renders the chat column: conversation history, prompt input and settings panel.
fn render_chat_content(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    i18n: &I18n,
    prompt: &mut String,
    font_size: f32,
    loading: bool,
    body_h: f32,
) -> Option<AiChatAction> {
    let viewer_bg = egui::Color32::from_rgb(20, 20, 25);
    let mut action = None;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        // Reserve height for info-bar + prompt (+ settings panel when open)
        let mut reserved_h = 110.0;
        if ws.ai_show_settings {
            reserved_h += 240.0;
        }
        let history_h = (body_h - reserved_h).max(100.0);

        // ── CONVERSATION HISTORY ──────────────────────────────────────────────
        egui::ScrollArea::vertical()
            .id_salt("ai_chat_terminal_history")
            .auto_shrink([false, false])
            .max_height(history_h)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;

                if !ws.ai_conversation.is_empty() {
                    AiChatWidget::ui_conversation(
                        ui,
                        &ws.ai_conversation,
                        font_size,
                        &mut ws.markdown_cache,
                    );
                    if !ws.ai_monologue.is_empty() {
                        ui.add_space(8.0);
                        AiChatWidget::ui_monologue(
                            ui,
                            &ws.ai_monologue,
                            font_size,
                            &mut ws.markdown_cache,
                        );
                    }
                } else if loading {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(egui::RichText::new(i18n.get("ai-chat-loading")).strong());
                    });
                    ui.add_space(4.0);
                    AiChatWidget::ui_monologue(
                        ui,
                        &ws.ai_monologue,
                        font_size,
                        &mut ws.markdown_cache,
                    );
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("PolyCredo AI").weak().size(24.0));
                    });
                }
            });

        // ── APPROVAL UI  OR  PROMPT ───────────────────────────────────────────
        if let Some((id, action_name, details, sender)) = ws.pending_plugin_approval.take() {
            render_approval_ui(ui, id, action_name, details, sender, ws);
        } else {
            ui.add_space(4.0);
            ui.scope(|ui| {
                ui.visuals_mut().widgets.noninteractive.bg_stroke =
                    egui::Stroke::new(1.0, egui::Color32::from_gray(60));
                ui.separator();
            });

            // Info bar
            ui.add_space(4.0);
            egui::Frame::new()
                .fill(viewer_bg)
                .inner_margin(egui::Margin::symmetric(8, 2))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    render_info_bar(ui, ws, loading);
                });

            ui.add_space(4.0);

            // Prompt input
            let prompt_bg = egui::Color32::from_rgb(45, 55, 65);
            egui::Frame::new()
                .fill(prompt_bg)
                .inner_margin(egui::Margin::symmetric(8, 2))
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let visuals = ui.visuals_mut();
                    visuals.override_text_color = Some(egui::Color32::from_rgb(200, 200, 200));
                    visuals.extreme_bg_color = prompt_bg;
                    visuals.selection.stroke = egui::Stroke::NONE;
                    visuals.widgets.hovered.expansion = 0.0;
                    visuals.widgets.active.expansion = 0.0;

                    if loading && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        ws.ai_cancellation_token
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                    }

                    let (send_via_kb, resp) = AiChatWidget::ui_input(
                        ui,
                        prompt,
                        font_size,
                        &i18n.get("ai-chat-placeholder-prompt"),
                        &ws.ai_history,
                        &mut ws.ai_history_index,
                    );

                    if ws.ai_focus_requested {
                        resp.request_focus();
                        ws.ai_focus_requested = false;
                    }

                    if send_via_kb {
                        action = Some(AiChatAction::Send);
                    }
                });
        }

        // ── SETTINGS PANEL ────────────────────────────────────────────────────
        if ws.ai_show_settings {
            ui.add_space(8.0);
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 35, 45))
                .inner_margin(12.0)
                .corner_radius(4.0)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(60)))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    AiChatWidget::ui_settings(
                        ui,
                        &mut ws.ai_expertise,
                        &mut ws.ai_reasoning_depth,
                        &mut ws.ai_language,
                        &mut ws.ai_system_prompt,
                        i18n,
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(format!("✔ {}", i18n.get("btn-save"))).clicked() {
                            action = Some(AiChatAction::SaveSettings);
                        }
                        if ui.button(i18n.get("btn-close")).clicked() {
                            ws.ai_show_settings = false;
                        }
                    });
                });
        }
    });

    action
}

// ── FOOTER ────────────────────────────────────────────────────────────────────

/// Renders the window footer bar: settings toggle, inspector toggle,
/// new-query and close buttons.
pub fn render_footer(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    i18n: &I18n,
) -> Option<AiChatAction> {
    let mut action = None;

    if ui
        .selectable_label(ws.ai_show_settings, i18n.get("ai-chat-settings-title"))
        .clicked()
    {
        ws.ai_show_settings = !ws.ai_show_settings;
    }

    let inspector_label = if ws.ai_inspector_open {
        "\u{1F50D} Hide Inspector"
    } else {
        "\u{1F50D} Show Inspector"
    };
    if ui
        .selectable_label(ws.ai_inspector_open, inspector_label)
        .clicked()
    {
        action = Some(AiChatAction::ToggleInspector);
    }

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button(i18n.get("btn-close")).clicked() {
            action = Some(AiChatAction::Close);
            return;
        }
        if (!ws.ai_conversation.is_empty() || ws.ai_response.is_some())
            && ui.button(i18n.get("ai-chat-btn-new")).clicked()
        {
            action = Some(AiChatAction::NewQuery);
        }
    });

    action
}

// ── HELPERS ───────────────────────────────────────────────────────────────────

fn render_info_bar(ui: &mut egui::Ui, ws: &WorkspaceState, loading: bool) {
    ui.horizontal(|ui| {
        if loading {
            ui.spinner();
        } else {
            ui.label("\u{1F4C1}");
        }
        ui.label(egui::RichText::new(ws.sandbox.root.to_string_lossy()).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "In: {} | Out: {}",
                    ws.ai_in_tokens, ws.ai_out_tokens
                ))
                .weak(),
            );
        });
    });
}
