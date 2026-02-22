use crate::app::types::AppShared;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn show(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
) {
    // We only show the modal if the user has triggered the gemini command
    // or if there is an active session (prompt or response present).
    if ws.gemini_prompt.is_empty() && ws.gemini_response.is_none() && !ws.gemini_loading {
        return;
    }

    let mut close_requested = false;
    let font_size = {
        let sh = shared.lock().expect("lock");
        sh.settings.editor_font_size
    };

    egui::Window::new(i18n.get("gemini-title"))
        .id(egui::Id::new("gemini_modal_win"))
        .collapsible(false)
        .resizable(true)
        .default_size([700.0, 500.0])
        .min_size([300.0, 250.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui: &mut egui::Ui| {
            ui.vertical(|ui| {
                // 1. Output Area (Markdown)
                if let Some(response) = &ws.gemini_response {
                    ui.label(egui::RichText::new(i18n.get("gemini-label-response")).strong());
                    egui::ScrollArea::vertical()
                        .id_salt("gemini_resp_scroll")
                        .max_height(ui.available_height() - 150.0)
                        .show(ui, |ui| {
                            egui_commonmark::CommonMarkViewer::new().show(
                                ui,
                                &mut ws.markdown_cache,
                                response,
                            );
                        });
                    ui.separator();
                } else if ws.gemini_loading {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(i18n.get("gemini-loading"));
                    });
                    ui.add_space(20.0);
                }

                // 2. Input Area
                ui.label(egui::RichText::new(i18n.get("gemini-label-prompt")).strong());
                let resp = ui.add(
                    egui::TextEdit::multiline(&mut ws.gemini_prompt)
                        .hint_text(i18n.get("gemini-placeholder-prompt"))
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::monospace(font_size))
                        .desired_rows(3),
                );

                if ws.gemini_response.is_none() && !ws.gemini_loading {
                    resp.request_focus();
                }

                ui.add_space(8.0);

                // 3. Buttons
                ui.horizontal(|ui| {
                    let can_send = !ws.gemini_prompt.trim().is_empty() && !ws.gemini_loading;
                    if ui
                        .add_enabled(
                            can_send,
                            egui::Button::new(format!("  {}  ", i18n.get("gemini-btn-send")))
                                .fill(egui::Color32::from_rgb(40, 80, 150)),
                        )
                        .clicked()
                    {
                        ws.gemini_loading = true;
                        ws.gemini_response = None;

                        let prompt = ws.gemini_prompt.clone();
                        let shared_arc = Arc::clone(shared);
                        let plugin_manager = {
                            let sh = shared_arc.lock().expect("lock");
                            Arc::clone(&sh.registry.plugins)
                        };

                        // SET CONTEXT: Tell the plugin about the active file
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
                            .map(|tab| tab.content.clone());

                        plugin_manager.set_context(crate::app::registry::plugins::HostContext {
                            active_file_path: active_path,
                            active_file_content: active_content,
                        });

                        std::thread::spawn(move || {
                            let config = {
                                let shared_lock = shared_arc.lock().expect("lock");
                                shared_lock
                                    .settings
                                    .plugins
                                    .get("gemini")
                                    .map(|s| s.config.clone())
                                    .unwrap_or_default()
                            };

                            let result =
                                plugin_manager.call("gemini", "ask_gemini", &prompt, &config);

                            let mut shared_lock = shared_arc.lock().expect("lock");
                            shared_lock
                                .actions
                                .push(crate::app::types::AppAction::PluginResponse(
                                    "gemini".to_string(),
                                    result.map_err(|e| e.to_string()),
                                ));
                        });
                    }

                    if ui.button(i18n.get("btn-close")).clicked() {
                        close_requested = true;
                    }

                    if ws.gemini_response.is_some()
                        && ui.button(i18n.get("gemini-btn-new")).clicked()
                    {
                        ws.gemini_response = None;
                        ws.gemini_prompt.clear();
                    }
                });
            });
        });

    if close_requested {
        ws.gemini_prompt.clear();
        ws.gemini_response = None;
        ws.gemini_loading = false;
    }
}
