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

    let viewer_bg = egui::Color32::from_rgb(20, 20, 25);

    modal.show(ctx, &mut show_flag, |ui| {
        ui.visuals_mut().window_fill = viewer_bg;
        ui.visuals_mut().panel_fill = viewer_bg;
        ui.visuals_mut().widgets.noninteractive.bg_fill = viewer_bg;
        ui.visuals_mut().widgets.noninteractive.bg_stroke = egui::Stroke::NONE;

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
            check_agent_authorization(shared, ws, ui);

            if inspector_open {
                egui::SidePanel::left("ai_chat_side")
                    .resizable(true)
                    .default_width(600.0)
                    .width_range(400.0..=800.0)
                    .frame(egui::Frame::NONE.fill(viewer_bg))
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

                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.fill(viewer_bg))
                    .show_inside(ui, |ui| {
                        render_inspector(ui, ws, font_size);
                    });
            } else {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.fill(viewer_bg))
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
            }
        });
    });

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
            .auto_shrink([false, true]) // Rosteme s obsahem!
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

        // 2. SEPARATOR (Always below history)
        ui.add_space(4.0);
        ui.scope(|ui| {
            ui.visuals_mut().widgets.noninteractive.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_gray(60));
            ui.separator();
        });

        // 3. CONTROLS (Info Bar + Prompt)
        if let Some((id, action_name, details, sender)) = ws.pending_plugin_approval.take() {
            render_approval_ui(ui, id, action_name, details, sender, ws);
        } else {
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

        // 4. SETTINGS (At the very bottom, pushing things up if needed)
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
        .stroke(egui::Stroke::new(1.5, egui::Color32::YELLOW))
        .inner_margin(16.0)
        .corner_radius(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚠️").size(24.0));
                    ui.label(
                        egui::RichText::new(format!("Agent '{}' vyžaduje schválení akce", id))
                            .strong()
                            .size(20.0)
                            .color(egui::Color32::YELLOW),
                    );
                });
                ui.add_space(12.0);

                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .id_salt("approval_details_scroll")
                    .show(ui, |ui| {
                        egui_commonmark::CommonMarkViewer::new().show(
                            ui,
                            &mut ws.markdown_cache,
                            &details,
                        );
                    });

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    let btn_size = egui::vec2(150.0, 32.0);
                    if ui
                        .add_sized(
                            btn_size,
                            egui::Button::new(egui::RichText::new("1 - Provést").strong()),
                        )
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Num1))
                    {
                        let _ = sender.send(crate::app::types::PluginApprovalResponse::Approve);
                    }
                    ui.add_space(12.0);
                    if ui
                        .add_sized(btn_size, egui::Button::new("2 - Schvalovat vždy"))
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Num2))
                    {
                        let _ =
                            sender.send(crate::app::types::PluginApprovalResponse::ApproveAlways);
                    }
                    ui.add_space(12.0);
                    if ui
                        .add_sized(btn_size, egui::Button::new("3/Esc - Zamítnout"))
                        .clicked()
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
        });
}

fn render_info_bar(ui: &mut egui::Ui, ws: &WorkspaceState, loading: bool) {
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
