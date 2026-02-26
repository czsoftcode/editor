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

        // SEPARATOR
        ui.scope(|ui| {
            ui.visuals_mut().widgets.noninteractive.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_gray(60));
            ui.separator();
        });

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

    ws.gemini_prompt = prompt;
    ws.show_gemini = show_flag;

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
