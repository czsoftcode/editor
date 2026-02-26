pub mod approval;
pub mod inspector;
pub mod logic;
pub mod render;

use crate::app::ai::AiManager;
use crate::app::types::AppShared;
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
    if !ws.show_ai_chat {
        return;
    }

    let mut prompt = ws.ai_prompt.clone();
    let response_text = ws.ai_response.clone();
    let loading = ws.ai_loading;
    let mut show_flag = ws.show_ai_chat;
    let inspector_open = ws.ai_inspector_open;
    let font_size = {
        let sh = shared.lock().expect("lock");
        sh.settings.editor_font_size
    };

    let mut action = None;

    let modal_width = if inspector_open { 1200.0 } else { 900.0 };
    let modal = StandardModal::new(i18n.get("ai-chat-title"), (id_salt, "ai_chat_modal"))
        .with_size(modal_width, 700.0);

    let viewer_bg = egui::Color32::from_rgb(20, 20, 25);

    modal.show(ctx, &mut show_flag, |ui| {
        ui.visuals_mut().window_fill = viewer_bg;
        ui.visuals_mut().panel_fill = viewer_bg;
        ui.visuals_mut().widgets.noninteractive.bg_fill = viewer_bg;
        ui.visuals_mut().widgets.noninteractive.bg_stroke = egui::Stroke::NONE;

        if (ui.input(|i| i.pointer.any_click()) || ui.ui_contains_pointer())
            && ui.ui_contains_pointer()
            && ws.focused_panel != crate::app::types::FocusedPanel::AiChat
        {
            ws.focused_panel = crate::app::types::FocusedPanel::AiChat;
            ws.ai_focus_requested = true;
        }

        // SEPARATOR
        ui.scope(|ui| {
            ui.visuals_mut().widgets.noninteractive.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_gray(60));
            ui.separator();
        });

        // FOOTER
        action = modal.ui_footer(ui, |ui| {
            if ui
                .selectable_label(ws.ai_show_settings, i18n.get("ai-chat-settings-title"))
                .clicked()
            {
                ws.ai_show_settings = !ws.ai_show_settings;
            }

            if ui
                .selectable_label(inspector_open, "\u{1F50D} Inspector".to_string())
                .clicked()
            {
                return Some(AiModalAction::ToggleInspector);
            }

            if (response_text.is_some() || !ws.ai_conversation.is_empty())
                && ui.button(i18n.get("ai-chat-btn-new")).clicked()
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
            render::check_agent_authorization(shared, ws, ui);

            if inspector_open {
                egui::SidePanel::left("ai_chat_side")
                    .resizable(true)
                    .default_width(600.0)
                    .width_range(400.0..=800.0)
                    .frame(egui::Frame::NONE.fill(viewer_bg))
                    .show_inside(ui, |ui| {
                        render::render_chat_main_ui(
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
                        inspector::render_inspector(ui, ws, font_size);
                    });
            } else {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.fill(viewer_bg))
                    .show_inside(ui, |ui| {
                        render::render_chat_main_ui(
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

    ws.ai_prompt = prompt;
    ws.show_ai_chat = show_flag;

    if let Some(act) = action {
        handle_modal_action(act, ws, shared, ctx, i18n);
    }
}

pub fn handle_modal_action(
    act: AiModalAction,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    _ctx: &egui::Context,
    _i18n: &I18n,
) {
    match act {
        AiModalAction::Send => {
            logic::send_query_to_agent(ws, shared);
        }
        AiModalAction::NewQuery => {
            ws.ai_response = None;
            ws.ai_prompt.clear();
            ws.ai_conversation.clear();
            ws.ai_monologue.clear();
            ws.ai_in_tokens = 0;
            ws.ai_out_tokens = 0;

            let model = {
                let sh = shared.lock().expect("lock");
                sh.settings
                    .plugins
                    .get(&ws.ai_selected_provider)
                    .and_then(|s| s.config.get("MODEL").cloned())
                    .unwrap_or_else(|| {
                        if ws.ai_selected_provider == "ollama" {
                            "llama3.1".to_string()
                        } else {
                            "gemini-1.5-flash".to_string()
                        }
                    })
            };

            ws.ai_conversation.push((
                String::new(),
                AiManager::get_logo(
                    crate::config::CLI_VERSION,
                    &model,
                    ws.ai_expertise,
                    ws.ai_reasoning_depth,
                ),
            ));
        }
        AiModalAction::ToggleInspector => {
            ws.ai_inspector_open = !ws.ai_inspector_open;
        }
        AiModalAction::SaveSettings => {
            let mut sh = shared.lock().expect("lock");
            let mut settings = (*sh.settings).clone();
            let provider_settings = settings
                .plugins
                .entry(ws.ai_selected_provider.clone())
                .or_default();
            provider_settings.expertise = ws.ai_expertise;
            provider_settings.reasoning_depth = ws.ai_reasoning_depth;
            provider_settings
                .config
                .insert("SYSTEM_PROMPT".to_string(), ws.ai_system_prompt.clone());
            provider_settings
                .config
                .insert("LANGUAGE".to_string(), ws.ai_language.clone());
            settings.save();
            sh.settings = Arc::new(settings);
            ws.toasts
                .push(crate::app::types::Toast::info("AI settings saved."));
        }
        AiModalAction::Close => {
            ws.show_ai_chat = false;
        }
    }
}
