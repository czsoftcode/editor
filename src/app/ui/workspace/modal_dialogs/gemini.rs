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

    // Dočasné proměnné pro rozbití mutable výpůjček
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
    let modal = StandardModal::new(i18n.get("gemini-title"), (id_salt, "gemini_modal"))
        .with_size(modal_width, 700.0);

    modal.show(ctx, &mut show_flag, |ui| {
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
                return Some(GeminiModalAction::ToggleInspector);
            }

            if (response_text.is_some() || !ws.gemini_conversation.is_empty())
                && ui.button(i18n.get("gemini-btn-new")).clicked()
            {
                return Some(GeminiModalAction::NewQuery);
            }

            if ui.button(i18n.get("btn-close")).clicked() {
                return Some(GeminiModalAction::Close);
            }
            None
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

            // ROZDĚLENÍ OKNA (SidePanel pro CLI, CentralPanel pro Inspektor)
            if inspector_open {
                egui::SidePanel::left("gemini_cli_side")
                    .resizable(true)
                    .default_width(600.0)
                    .width_range(400.0..=800.0)
                    .show_inside(ui, |ui| {
                        render_gemini_main_ui(
                            ui,
                            ws,
                            i18n,
                            &mut prompt,
                            font_size,
                            &mut action,
                            loading,
                            response_text.is_some(),
                        );
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    render_gemini_inspector(ui, ws, font_size);
                });
            } else {
                render_gemini_main_ui(
                    ui,
                    ws,
                    i18n,
                    &mut prompt,
                    font_size,
                    &mut action,
                    loading,
                    response_text.is_some(),
                );
            }
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
                    let context_payload = StandardAI::generate_context(ws);

                    // Construct JSON input for the plugin (including history and unified context)
                    let plugin_input = serde_json::json!({
                        "prompt": captured_prompt,
                        "history": ws.gemini_conversation,
                        "context": context_payload,
                        "tools": StandardAI::get_standard_tools()
                    });
                    let input_json = serde_json::to_string(&plugin_input).unwrap_or_default();

                    // Initial feedback for inspector
                    ws.gemini_last_payload = "Constructing final request in WASM plugin...\n\n\
                        The final payload including hardcoded RAG mandates and filtered history \
                        will appear here once the plugin prepares the outgoing API call."
                        .to_string();

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

                    ws.gemini_cancellation_token =
                        Arc::new(std::sync::atomic::AtomicBool::new(false));

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
                            plugin_manager.call("gemini", "ask_gemini", &input_json, &config);
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
                ws.gemini_last_payload.clear();
                ws.gemini_total_tokens = 0;
                ctx.request_repaint(); // Vynutit okamžité překreslení!

                // Get current model from settings to show in logo
                let gemini_model = {
                    let sh = shared.lock().expect("lock");
                    sh.settings
                        .plugins
                        .get("gemini")
                        .and_then(|s| s.config.get("MODEL").cloned())
                        .unwrap_or_else(|| "gemini-1.5-flash".to_string())
                };

                ws.gemini_conversation = vec![(
                    String::new(),
                    StandardAI::get_logo(
                        crate::config::CLI_VERSION,
                        &gemini_model,
                        crate::config::CLI_TIER,
                    ),
                )];
            }
            GeminiModalAction::ToggleInspector => {
                ws.gemini_inspector_open = !ws.gemini_inspector_open;
            }
            GeminiModalAction::SaveSettings => {
                let sys_prompt = ws.gemini_system_prompt.clone();
                let lang = ws.gemini_language.clone();

                let mut shared_lock = shared.lock().expect("lock");
                let mut settings = (*shared_lock.settings).clone();

                let gemini_settings = settings.plugins.entry("gemini".to_string()).or_default();
                gemini_settings
                    .config
                    .insert("SYSTEM_PROMPT".to_string(), sys_prompt);
                gemini_settings.config.insert("LANGUAGE".to_string(), lang);

                settings.save();
                shared_lock.settings = Arc::new(settings);
                ws.toasts.push(crate::app::types::Toast::info(
                    "Gemini settings saved as default.",
                ));
            }
            GeminiModalAction::Close => {
                ws.show_gemini = false;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_gemini_main_ui(
    ui: &mut egui::Ui,
    ws: &mut WorkspaceState,
    i18n: &I18n,
    prompt: &mut String,
    font_size: f32,
    action: &mut Option<GeminiModalAction>,
    loading: bool,
    _has_response: bool,
) {
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

                if ui
                    .button(
                        egui::RichText::new(format!("✔ {}", i18n.get("btn-save")))
                            .color(egui::Color32::from_rgb(150, 255, 150)),
                    )
                    .on_hover_text("Save as global default")
                    .clicked()
                {
                    *action = Some(GeminiModalAction::SaveSettings);
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

    // 1. VSTUP / SCHVALOVÁNÍ (NAHOŘE)
    ui.add_space(4.0);

    // Handle cancellation via Esc
    if loading && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        ws.gemini_cancellation_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    let mut edit_resp = None;

    if let Some((id, _action_name, details, sender)) = ws.pending_plugin_approval.take() {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(60, 45, 10))
            .stroke(egui::Stroke::new(1.0, egui::Color32::YELLOW))
            .inner_margin(10.0)
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚠️").size(24.0));
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Agent '{}' vyžaduje schválení akce:", id))
                                .strong()
                                .color(egui::Color32::YELLOW)
                                .size(16.0),
                        );
                        ui.label(
                            egui::RichText::new("Bez vašeho potvrzení nemůže pokračovat.")
                                .small()
                                .weak(),
                        );
                    });
                });

                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .id_salt("approval_details_scroll")
                    .max_height(250.0)
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(egui::RichText::new(&details).monospace()).wrap());
                    });

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let btn_approve = ui.add(egui::Button::new(
                        egui::RichText::new("1 - Provést akci").strong(),
                    ));
                    if btn_approve.clicked() || ui.input(|i| i.key_pressed(egui::Key::Num1)) {
                        let _ = sender.send(crate::app::types::PluginApprovalResponse::Approve);
                    } else if ui.button("2 - Schvalovat vždy").clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Num2))
                    {
                        let _ =
                            sender.send(crate::app::types::PluginApprovalResponse::ApproveAlways);
                    } else if ui.button("3/Esc - Zamítnout").clicked()
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
    } else {
        ui.label(egui::RichText::new(i18n.get("gemini-label-prompt")).strong());
        let (send_via_kb, resp) = StandardAI::ui_input(
            ui,
            prompt,
            font_size,
            &i18n.get("gemini-placeholder-prompt"),
            &ws.gemini_history,
            &mut ws.gemini_history_index,
        );
        edit_resp = Some(resp);

        if send_via_kb && action.is_none() {
            *action = Some(GeminiModalAction::Send);
        }
    }

    ui.add_space(8.0);

    // INFO ŘÁDEK (Mezi vstupem a historií)
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
            let time = ui.input(|i| i.time);
            let hue = (time * 0.5).fract() as f32;
            let color: egui::Color32 = egui::ecolor::Hsva::new(hue, 0.8, 1.0, 1.0).into();
            ui.add(egui::Spinner::new().color(color));
        } else {
            ui.label(egui::RichText::new("📁").weak());
        }

        ui.label(egui::RichText::new(display_path).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let tokens = ws.gemini_total_tokens;
            ui.label(egui::RichText::new(format!("Session tokens: {}", tokens)).weak());
        });
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(8.0);

    // 2. HISTORIE (VYPLNÍ ZBYTEK)
    ui.vertical(|ui| {
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
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        let path_purple = egui::Color32::from_rgb(120, 80, 170);
                        let terminal_text = egui::Color32::from_rgb(175, 175, 175); // Jasnější šedá

                        ui.scope(|ui| {
                            let style = ui.style_mut();
                            style.visuals.widgets.noninteractive.fg_stroke.color = terminal_text;
                            style.visuals.widgets.inactive.fg_stroke.color = terminal_text;
                            style.visuals.widgets.active.fg_stroke.color = path_purple;
                            style.visuals.hyperlink_color = path_purple;
                            style.visuals.code_bg_color = egui::Color32::TRANSPARENT;

                            let path_re = regex::Regex::new(r"(?P<link>\[[^\]]+\]\([^\)]+\))|`(?P<code_inner>[^`]+)`|(?P<path>\b(?:src|locales|docs|app|ui|workspace|packaging|privacy|vendor|target)/[a-zA-Z0-9_\-./]+\.[a-z0-9]+\b|\b[a-zA-Z0-9_\-./]+\.(?:rs|toml|md|ftl|sh|json)\b)").ok();

                            let mut full_monologue = String::new();
                            for line in &ws.gemini_monologue {
                                let mut processed_line = line.clone();
                                if let Some(re) = &path_re {
                                    processed_line = re.replace_all(&processed_line, |caps: &regex::Captures| {
                                        if caps.name("link").is_some() { caps[0].to_string() }
                                        else if let Some(c) = caps.name("code_inner") { format!("[{}](code)", c.as_str()) }
                                        else { format!("[{}](path)", &caps[0]) }
                                    }).to_string();
                                }

                                let trimmed = processed_line.trim();
                                if trimmed.starts_with("Step") {
                                    full_monologue.push_str(&format!("_{}_\n", trimmed.replace('>', "").trim()));
                                } else {
                                    full_monologue.push_str(&format!("│ {}\n", trimmed.replace('>', "").trim()));
                                }
                            }

                            if !full_monologue.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    let (rect, _) = ui.allocate_at_least(egui::vec2(2.0, 0.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, 0.0, terminal_text);

                                    egui::Frame::new()
                                        .fill(egui::Color32::from_gray(35)) // Subtle background
                                        .inner_margin(egui::Margin::symmetric(12, 12)) // Padding top/bottom and sides
                                        .corner_radius(egui::CornerRadius {
                                            nw: 0, ne: 4, sw: 0, se: 4
                                        })
                                        .show(ui, |ui| {
                                            egui_commonmark::CommonMarkViewer::new()
                                                .max_image_width(Some(512))
                                                .show(ui, &mut ws.markdown_cache, &full_monologue);
                                        });
                                });
                            }
                        });
                    });
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("PolyCredo Gemini").weak().size(20.0));
            });
        }
    });

    // Auto-focus logic
    if ws.gemini_focus_requested && !ws.gemini_show_settings {
        if let Some(resp) = edit_resp {
            resp.request_focus();
        }
        ws.gemini_focus_requested = false;
    }
}

fn render_gemini_inspector(ui: &mut egui::Ui, ws: &mut WorkspaceState, font_size: f32) {
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

        egui::Frame::new()
            .fill(egui::Color32::from_rgb(30, 30, 35))
            .inner_margin(4.0)
            .show(ui, |ui| {
                egui::ScrollArea::both()
                    .id_salt("gemini_inspector_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut text = ws.gemini_last_payload.clone();
                        ui.add(
                            egui::TextEdit::multiline(&mut text)
                                .font(egui::FontId::monospace(font_size * 0.9))
                                .desired_width(f32::INFINITY)
                                .code_editor()
                                .interactive(true)
                                .lock_focus(false),
                        );
                    });
            });
    });
}
