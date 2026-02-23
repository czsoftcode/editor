use crate::app::types::AppShared;
use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub enum GeminiModalAction {
    Send,
    NewQuery,
    Close,
}

pub fn show(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
) {
    if !ws.show_gemini {
        return;
    }

    // Dočasné proměnné pro rozbití mutable výpůjček
    let mut prompt = ws.gemini_prompt.clone();
    let response_text = ws.gemini_response.clone();
    let loading = ws.gemini_loading;
    let mut show_flag = ws.show_gemini;
    let font_size = {
        let sh = shared.lock().expect("lock");
        sh.settings.editor_font_size
    };

    let mut action = None;

    let modal =
        StandardModal::new(i18n.get("gemini-title"), "gemini_modal").with_size(900.0, 700.0);

    modal.show(ctx, &mut show_flag, |ui| {
        // FOOTER
        action = modal.ui_footer(ui, |ui| {
            if ui.button(i18n.get("btn-close")).clicked() {
                return Some(GeminiModalAction::Close);
            }

            if response_text.is_some() && ui.button(i18n.get("gemini-btn-new")).clicked() {
                return Some(GeminiModalAction::NewQuery);
            }

            let can_send = !loading;
            if ui
                .add_enabled(
                    can_send,
                    egui::Button::new(i18n.get("gemini-btn-send"))
                        .fill(egui::Color32::from_rgb(40, 80, 150)),
                )
                .clicked()
            {
                return Some(GeminiModalAction::Send);
            }

            None
        });

        // BODY
        modal.ui_body(ui, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                // 1. VSTUPNÍ POLE (Dole)
                ui.add_space(8.0);
                let edit_resp = ui.add(
                    egui::TextEdit::multiline(&mut prompt)
                        .hint_text(i18n.get("gemini-placeholder-prompt"))
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::monospace(font_size))
                        .desired_rows(4),
                );
                ui.label(egui::RichText::new(i18n.get("gemini-label-prompt")).strong());

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // 2. ODPOVĚĎ AI (Vyplní zbytek nahoře)
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    if let Some(resp) = &response_text {
                        ui.label(egui::RichText::new(i18n.get("gemini-label-response")).strong());
                        egui::ScrollArea::vertical()
                            .id_salt("gemini_resp_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                egui_commonmark::CommonMarkViewer::new().show(
                                    ui,
                                    &mut ws.markdown_cache,
                                    resp,
                                );
                            });
                    } else if loading {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(i18n.get("gemini-loading"));
                        });
                    } else {
                        // Prázdný stav - zobrazíme aspoň místo
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("PolyCredo Gemini").weak().size(20.0));
                        });
                    }
                });

                // Auto-fokus na začátku
                if response_text.is_none() && !loading {
                    edit_resp.request_focus();
                }
            });
        });
    });

    // Synchronizace zpět
    ws.gemini_prompt = prompt;
    ws.show_gemini = show_flag;

    if let Some(act) = action {
        match act {
            GeminiModalAction::Send => {
                if !ws.gemini_prompt.trim().is_empty() {
                    ws.gemini_loading = true;
                    ws.gemini_response = None;

                    let p = ws.gemini_prompt.clone();
                    let shared_arc = Arc::clone(shared);
                    let plugin_manager = {
                        let sh = shared_arc.lock().expect("lock");
                        Arc::clone(&sh.registry.plugins)
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
                        let result = plugin_manager.call("gemini", "ask_gemini", &p, &config);
                        let mut shared_lock = shared_arc.lock().expect("lock");
                        shared_lock
                            .actions
                            .push(crate::app::types::AppAction::PluginResponse(
                                "gemini".to_string(),
                                result.map_err(|e| e.to_string()),
                            ));
                    });
                }
            }
            GeminiModalAction::NewQuery => {
                ws.gemini_response = None;
                ws.gemini_prompt.clear();
            }
            GeminiModalAction::Close => {
                ws.show_gemini = false;
            }
        }
    }
}
