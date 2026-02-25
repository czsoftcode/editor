use crate::app::ai::AiManager;
use crate::app::types::AppShared;
use crate::app::ui::widgets::ai::AiChatWidget;
use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub enum AiModalAction {
    Send,
    NewQuery,
    ToggleInspector,
    SaveSettings,
    Close,
}

pub fn show(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
    id_salt: &impl std::hash::Hash,
) {
    if !ws.show_gemini {
        // Keeping show_gemini for now as it's the flag in state, but logic is generic
        return;
    }

    let mut prompt = ws.gemini_prompt.clone();
    let response_text = ws.gemini_response.clone();
    let loading = ws.gemini_loading;
    let mut show_flag = ws.show_gemini;
    let inspector_open = ws.gemini_inspector_open;
    let font_size = {
        let sh = shared.lock().expect("lock");
        sh.settings.editor_font_size
    };

    let mut action = None;

    let modal_width = if inspector_open { 1200.0 } else { 900.0 };
    let modal = StandardModal::new(i18n.get("gemini-title"), (id_salt, "ai_chat_modal"))
        .with_size(modal_width, 700.0);

    modal.show(ctx, &mut show_flag, |ui| {
        if (ui.input(|i| i.pointer.any_click()) || ui.ui_contains_pointer())
            && ui.ui_contains_pointer()
            && ws.focused_panel != crate::app::types::FocusedPanel::Gemini
        {
            ws.focused_panel = crate::app::types::FocusedPanel::Gemini;
            ws.gemini_focus_requested = true;
        }

        // FOOTER
        action = modal.ui_footer(ui, |ui| {
            if ui
                .selectable_label(ws.gemini_show_settings, i18n.get("gemini-settings-title"))
                .clicked()
            {
                ws.gemini_show_settings = !ws.gemini_show_settings;
            }

            if ui
                .selectable_label(inspector_open, "\u{1F50D} Inspector".to_string())
                .clicked()
            {
                return Some(AiModalAction::ToggleInspector);
            }

            if (response_text.is_some() || !ws.gemini_conversation.is_empty())
                && ui.button(i18n.get("gemini-btn-new")).clicked()
            {
                return Some(AiModalAction::NewQuery);
            }

            if ui.button(i18n.get("btn-close")).clicked() {
                return Some(AiModalAction::Close);
            }
            None
        });

        // BODY
        modal.ui_body(ui, |ui| {
            // Check if active agent needs authorization (e.g. Gemini plugin)
            check_agent_authorization(shared, ws, ui);

            if inspector_open {
                egui::SidePanel::left("ai_chat_side")
                    .resizable(true)
                    .default_width(600.0)
                    .width_range(400.0..=800.0)
                    .show_inside(ui, |ui| {
                        render_chat_main_ui(
                            ui,
                            ws,
                            i18n,
                            &mut prompt,
                            font_size,
                            &mut action,
                            loading,
                        );
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    render_inspector(ui, ws, font_size);
                });
            } else {
                render_chat_main_ui(ui, ws, i18n, &mut prompt, font_size, &mut action, loading);
            }
        });
    });

    // Sync back
    ws.gemini_prompt = prompt;
    ws.show_gemini = show_flag;

    if let Some(act) = action {
        handle_modal_action(act, ws, shared, ctx, i18n);
    }
}

fn check_agent_authorization(
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

fn render_chat_main_ui(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    i18n: &I18n,
    prompt: &mut String,
    font_size: f32,
    action: &mut Option<AiModalAction>,
    loading: bool,
) {
    if ws.gemini_show_settings {
        if AiChatWidget::ui_settings(
            ui,
            &mut ws.gemini_expertise,
            &mut ws.gemini_reasoning_depth,
            &mut ws.gemini_language,
            &mut ws.gemini_system_prompt,
            i18n,
        ) {
            // Settings changed, could trigger auto-save here if needed
        }
        if ui.button(format!("✔ {}", i18n.get("btn-save"))).clicked() {
            *action = Some(AiModalAction::SaveSettings);
        }
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);
    }

    // Input / Approval Area
    ui.add_space(4.0);
    if loading && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        ws.gemini_cancellation_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    if let Some((id, action_name, details, sender)) = ws.pending_plugin_approval.take() {
        render_approval_ui(ui, id, action_name, details, sender, ws);
    } else {
        ui.label(egui::RichText::new(i18n.get("gemini-label-prompt")).strong());
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
    }

    ui.add_space(8.0);
    render_info_bar(ui, ws, loading);
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(8.0);

    // Results / History Area
    ui.vertical(|ui| {
        if !ws.gemini_conversation.is_empty() {
            ui.label(egui::RichText::new(i18n.get("gemini-label-response")).strong());
            AiChatWidget::ui_conversation(
                ui,
                &ws.gemini_conversation,
                font_size,
                &mut ws.markdown_cache,
            );
        } else if loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(egui::RichText::new(i18n.get("gemini-loading")).strong());
            });
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .id_salt("monologue_scroll")
                .max_height(200.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    AiChatWidget::ui_monologue(ui, &ws.gemini_monologue, &mut ws.markdown_cache);
                });
        }
    });
}

fn render_approval_ui(
    ui: &mut egui::Ui,
    id: String,
    _action_name: String,
    details: String,
    sender: std::sync::mpsc::Sender<crate::app::types::PluginApprovalResponse>,
    ws: &mut WorkspaceState,
) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(60, 45, 10))
        .stroke(egui::Stroke::new(1.0, egui::Color32::YELLOW))
        .inner_margin(10.0)
        .corner_radius(4.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(format!("Agent '{}' vyžaduje schválení:", id))
                    .strong()
                    .color(egui::Color32::YELLOW),
            );
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    egui_commonmark::CommonMarkViewer::new().show(
                        ui,
                        &mut ws.markdown_cache,
                        &details,
                    );
                });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("1 - Provést").clicked()
                    || ui.input(|i| i.key_pressed(egui::Key::Num1))
                {
                    let _ = sender.send(crate::app::types::PluginApprovalResponse::Approve);
                }
                if ui.button("2 - Vždy").clicked() || ui.input(|i| i.key_pressed(egui::Key::Num2))
                {
                    let _ = sender.send(crate::app::types::PluginApprovalResponse::ApproveAlways);
                }
                if ui.button("3/Esc - Zamítnout").clicked()
                    || ui.input(|i| {
                        i.key_pressed(egui::Key::Num3) || i.key_pressed(egui::Key::Escape)
                    })
                {
                    let _ = sender.send(crate::app::types::PluginApprovalResponse::Deny);
                    ws.gemini_cancellation_token
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                } else {
                    ws.pending_plugin_approval = Some((id, _action_name, details, sender));
                }
            });
        });
}

fn render_info_bar(ui: &mut egui::Ui, ws: &WorkspaceState, loading: bool) {
    ui.horizontal(|ui| {
        if loading {
            ui.spinner();
        } else {
            ui.label("📁");
        }
        ui.label(ws.sandbox.root.to_string_lossy());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!(
                "In: {} | Out: {}",
                ws.gemini_in_tokens, ws.gemini_out_tokens
            ));
        });
    });
}

fn render_inspector(ui: &mut egui::Ui, ws: &mut WorkspaceState, font_size: f32) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("\u{1F50D} AI Inspector").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear").clicked() {
                    ws.gemini_last_payload.clear();
                }
                if ui.button("Copy").clicked() {
                    ui.ctx().copy_text(ws.gemini_last_payload.clone());
                }
            });
        });
        ui.add_space(4.0);
        egui::ScrollArea::both()
            .id_salt("inspector_scroll")
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut ws.gemini_last_payload)
                        .font(egui::FontId::monospace(font_size * 0.9))
                        .desired_width(f32::INFINITY)
                        .code_editor(),
                );
            });
    });
}

fn handle_modal_action(
    act: AiModalAction,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    _ctx: &egui::Context,
    _i18n: &I18n,
) {
    match act {
        AiModalAction::Send => {
            send_query_to_agent(ws, shared);
        }
        AiModalAction::NewQuery => {
            ws.gemini_response = None;
            ws.gemini_prompt.clear();
            ws.gemini_conversation.clear();
            ws.gemini_monologue.clear();
            ws.gemini_in_tokens = 0;
            ws.gemini_out_tokens = 0;

            let model = {
                let sh = shared.lock().expect("lock");
                sh.settings
                    .plugins
                    .get("gemini")
                    .and_then(|s| s.config.get("MODEL").cloned())
                    .unwrap_or_else(|| "gemini-1.5-flash".to_string())
            };

            ws.gemini_conversation.push((
                String::new(),
                AiManager::get_logo(
                    crate::config::CLI_VERSION,
                    &model,
                    ws.gemini_expertise,
                    ws.gemini_reasoning_depth,
                ),
            ));
        }
        AiModalAction::ToggleInspector => {
            ws.gemini_inspector_open = !ws.gemini_inspector_open;
        }
        AiModalAction::SaveSettings => {
            let mut sh = shared.lock().expect("lock");
            let mut settings = (*sh.settings).clone();
            let g_settings = settings.plugins.entry("gemini".to_string()).or_default();
            g_settings.expertise = ws.gemini_expertise;
            g_settings.reasoning_depth = ws.gemini_reasoning_depth;
            g_settings
                .config
                .insert("SYSTEM_PROMPT".to_string(), ws.gemini_system_prompt.clone());
            g_settings
                .config
                .insert("LANGUAGE".to_string(), ws.gemini_language.clone());
            settings.save();
            sh.settings = Arc::new(settings);
            ws.toasts
                .push(crate::app::types::Toast::info("AI settings saved."));
        }
        AiModalAction::Close => {
            ws.show_gemini = false;
        }
    }
}

fn send_query_to_agent(ws: &mut WorkspaceState, shared: &Arc<Mutex<AppShared>>) {
    if ws.gemini_prompt.trim().is_empty() {
        return;
    }

    let prompt = ws.gemini_prompt.clone();
    let context = AiManager::generate_context(ws);
    let tools = crate::app::ai::get_standard_tools();

    let input = serde_json::json!({
        "prompt": prompt,
        "history": ws.gemini_conversation,
        "context": context,
        "tools": tools
    });

    ws.gemini_conversation.push((prompt.clone(), String::new()));
    ws.gemini_prompt.clear();
    ws.gemini_loading = true;
    ws.gemini_monologue.clear();
    ws.gemini_cancellation_token = Arc::new(std::sync::atomic::AtomicBool::new(false));

    if ws.gemini_history.last() != Some(&prompt) {
        ws.gemini_history.push(prompt);
    }
    ws.gemini_history_index = None;

    let sh_arc = Arc::clone(shared);
    let (plugin_manager, config, expertise, depth, sys_prompt, lang) = {
        let sh = sh_arc.lock().expect("lock");
        let config = sh
            .settings
            .plugins
            .get("gemini")
            .map(|s| s.config.clone())
            .unwrap_or_default();
        (
            Arc::clone(&sh.registry.plugins),
            config,
            ws.gemini_expertise,
            ws.gemini_reasoning_depth,
            ws.gemini_system_prompt.clone(),
            ws.gemini_language.clone(),
        )
    };

    // Set host context for tools
    let active_path = ws.editor.active_path().map(|p| {
        p.strip_prefix(&ws.root_path)
            .unwrap_or(p)
            .to_string_lossy()
            .into_owned()
    });
    let active_content = ws
        .editor
        .active_tab
        .and_then(|idx| ws.editor.tabs.get(idx))
        .map(|t| t.content.clone());

    plugin_manager.set_context(crate::app::registry::plugins::HostContext {
        active_file_path: active_path,
        active_file_content: active_content,
        project_index: Some(Arc::clone(&ws.project_index)),
        semantic_index: Some(Arc::clone(&ws.semantic_index)),
        root_path: Some(ws.root_path.clone()),
        auto_approved_actions: std::collections::HashSet::new(),
        is_cancelled: Arc::clone(&ws.gemini_cancellation_token),
    });

    std::thread::spawn(move || {
        let intelligence = AiManager::get_system_mandates(expertise, depth);
        let mut final_config = config;
        final_config.insert(
            "SYSTEM_PROMPT".to_string(),
            format!("{}\n\n{}", intelligence, sys_prompt),
        );
        final_config.insert("LANGUAGE".to_string(), lang);

        let input_str = serde_json::to_string(&input).unwrap_or_default();
        let result = plugin_manager.call("gemini", "ask_gemini", &input_str, &final_config);

        let mut sh = sh_arc.lock().expect("lock");
        sh.actions
            .push(crate::app::types::AppAction::PluginResponse(
                "gemini".to_string(),
                result.map_err(|e| e.to_string()),
            ));
    });
}
