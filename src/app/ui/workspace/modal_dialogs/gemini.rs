use crate::app::types::AppShared;
use crate::app::ui::widgets::ai_cli::StandardAI;
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
    id_salt: &impl std::hash::Hash,
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

    let modal = StandardModal::new(i18n.get("gemini-title"), (id_salt, "gemini_modal"))
        .with_size(900.0, 700.0);

    modal.show(ctx, &mut show_flag, |ui| {
        // FOOTER
        action = modal.ui_footer(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(ws.gemini_show_settings, i18n.get("gemini-settings-title"))
                    .clicked()
                {
                    ws.gemini_show_settings = !ws.gemini_show_settings;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
                })
                .inner
            })
            .inner
        });

        // BODY
        modal.ui_body(ui, |ui| {
            // Check if plugin needs authorization
            let pending_auth = {
                let sh = shared.lock().expect("lock");
                sh.registry
                    .plugins
                    .get_pending_authorizations()
                    .into_iter()
                    .find(|(id, _)| id == "gemini")
            };

            if let Some((id, meta)) = pending_auth {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(egui::RichText::new("🛡").size(64.0));
                    ui.add_space(16.0);

                    let mut args = fluent_bundle::FluentArgs::new();
                    args.set("name", meta.name.clone());
                    args.set("hosts", meta.allowed_hosts.join(", "));
                    ui.label(
                        egui::RichText::new(i18n.get_args("plugin-auth-bar-msg", &args))
                            .strong()
                            .size(18.0),
                    );

                    ui.add_space(30.0);
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "✔ {}",
                                    i18n.get("gemini-btn-allow-start")
                                ))
                                .size(16.0),
                            )
                            .fill(egui::Color32::from_rgb(40, 120, 40)),
                        )
                        .clicked()
                    {
                        let plugin_manager = {
                            let sh = shared.lock().expect("lock");
                            Arc::clone(&sh.registry.plugins)
                        };
                        let config = {
                            let sh = shared.lock().expect("lock");
                            sh.settings
                                .plugins
                                .get(&id)
                                .map(|s| s.config.clone())
                                .unwrap_or_default()
                        };

                        if let Err(e) = plugin_manager.authorize(&id, &config) {
                            ws.plugin_error = Some(format!("Authorization failed: {}", e));
                        }
                        ui.ctx().request_repaint();
                    }

                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new(i18n.get("plugin-auth-bar-hint"))
                            .small()
                            .weak(),
                    );
                });
                return;
            }

            if ws.gemini_show_settings {
                ui.group(|ui| {
                    ui.label(egui::RichText::new(i18n.get("gemini-settings-title")).strong());
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label(i18n.get("gemini-label-language"));
                        egui::ComboBox::from_id_salt("gemini_lang")
                            .selected_text(crate::i18n::lang_display_name(&ws.gemini_language))
                            .show_ui(ui, |ui| {
                                for lang in crate::i18n::SUPPORTED_LANGS {
                                    ui.selectable_value(
                                        &mut ws.gemini_language,
                                        lang.to_string(),
                                        crate::i18n::lang_display_name(lang),
                                    );
                                }
                            });

                        if ui
                            .button(i18n.get("gemini-btn-reset"))
                            .on_hover_text("Factory Reset")
                            .clicked()
                        {
                            ws.gemini_system_prompt = i18n.get("gemini-default-prompt");
                            ws.gemini_language = i18n.lang().to_string();
                        }
                    });

                    ui.add_space(4.0);
                    ui.label(i18n.get("gemini-label-system-prompt"));
                    ui.add(
                        egui::TextEdit::multiline(&mut ws.gemini_system_prompt)
                            .desired_width(f32::INFINITY)
                            .desired_rows(3),
                    );
                });
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                // 1. VSTUPNÍ POLE (Dole)
                ui.add_space(8.0);

                let (send_via_kb, edit_resp) = StandardAI::ui_input(
                    ui,
                    &mut prompt,
                    font_size,
                    &i18n.get("gemini-placeholder-prompt"),
                    &ws.gemini_history,
                    &mut ws.gemini_history_index,
                );

                if send_via_kb && action.is_none() {
                    action = Some(GeminiModalAction::Send);
                }

                ui.label(egui::RichText::new(i18n.get("gemini-label-prompt")).strong());

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // FOOTER / INFO
                ui.horizontal(|ui| {
                    let path_str = ws.sandbox.root.to_string_lossy();
                    let display_path = if let Some(home) = dirs::home_dir() {
                        let home_str = home.to_string_lossy();
                        if path_str.starts_with(&*home_str) {
                            path_str.replacen(&*home_str, "~", 1)
                        } else {
                            path_str.into_owned()
                        }
                    } else {
                        path_str.into_owned()
                    };

                    if loading {
                        // Animated color spinner
                        let time = ui.input(|i| i.time);
                        let hue = (time * 0.5).fract() as f32;
                        let color: egui::Color32 =
                            egui::ecolor::Hsva::new(hue, 0.8, 1.0, 1.0).into();
                        ui.add(egui::Spinner::new().color(color));
                    } else {
                        ui.label(egui::RichText::new("📁").weak());
                    }

                    ui.label(egui::RichText::new(display_path).weak());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let tokens = ws.gemini_total_tokens;
                        let token_text = if tokens >= 1_000_000 {
                            format!("{:.2}M", tokens as f32 / 1_000_000.0)
                        } else if tokens >= 1_000 {
                            format!("{:.2}k", tokens as f32 / 1_000.0)
                        } else {
                            format!("{}", tokens)
                        };
                        ui.label(
                            egui::RichText::new(format!("Session tokens: {}", token_text)).weak(),
                        );
                    });
                });

                // 2. ODPOVĚĎ AI (Vyplní zbytek nahoře)
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    if !ws.gemini_conversation.is_empty() {
                        ui.label(egui::RichText::new(i18n.get("gemini-label-response")).strong());
                        StandardAI::ui_response(
                            ui,
                            &ws.gemini_conversation,
                            font_size,
                            &mut ws.markdown_cache,
                        );
                    } else if loading {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(egui::RichText::new(i18n.get("gemini-loading")).strong());
                            });
                            ui.add_space(4.0);
                            egui::ScrollArea::vertical()
                                .id_salt("gemini_monologue_scroll")
                                .max_height(200.0)
                                .stick_to_bottom(true)
                                .show(ui, |ui| {
                                    for line in &ws.gemini_monologue {
                                        ui.label(
                                            egui::RichText::new(format!("> {}", line))
                                                .weak()
                                                .monospace(),
                                        );
                                    }
                                });
                        });
                    } else {
                        // Prázdný stav - zobrazíme aspoň místo
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("PolyCredo Gemini").weak().size(20.0));
                        });
                    }
                });

                // Auto-fokus na začátku, ale jen pokud nejsou otevřená nastavení (aby nekradl fokus)
                if response_text.is_none() && !loading && !ws.gemini_show_settings {
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
                    let captured_prompt = ws.gemini_prompt.clone();

                    // Add to history
                    if ws.gemini_history.last() != Some(&captured_prompt) {
                        ws.gemini_history.push(captured_prompt.clone());
                    }
                    ws.gemini_history_index = None;
                    ws.gemini_monologue.clear();

                    // Add to conversation view and CLEAR prompt
                    ws.gemini_conversation
                        .push((captured_prompt.clone(), String::new()));
                    ws.gemini_prompt.clear();

                    ws.gemini_loading = true;
                    ws.gemini_response = None;

                    let sys_prompt = ws.gemini_system_prompt.clone();
                    let lang = ws.gemini_language.clone();

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
                        let mut config = {
                            let shared_lock = shared_arc.lock().expect("lock");
                            shared_lock
                                .settings
                                .plugins
                                .get("gemini")
                                .map(|s| s.config.clone())
                                .unwrap_or_default()
                        };

                        // Override/Inject user-customized settings
                        config.insert("SYSTEM_PROMPT".to_string(), sys_prompt);
                        config.insert("LANGUAGE".to_string(), lang);

                        let result =
                            plugin_manager.call("gemini", "ask_gemini", &captured_prompt, &config);
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
