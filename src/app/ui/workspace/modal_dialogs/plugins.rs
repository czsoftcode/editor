use crate::app::types::AppShared;
use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use crate::settings::PluginSettings;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub enum PluginModalAction {
    Save,
    Cancel,
}

pub fn show(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
    _id_salt: &std::ffi::OsString,
) {
    if !ws.show_plugins {
        return;
    }

    if ws.plugins_draft.is_none() {
        let shared_lock = shared.lock().expect("lock");
        ws.plugins_draft = Some((*shared_lock.settings).clone());
    }

    let mut draft = ws.plugins_draft.take().unwrap();
    let mut selected_id = ws
        .selected_plugin_id
        .clone()
        .unwrap_or_else(|| "system:welcome".to_string());
    let mut ai_font_scale = ws.ai.settings.font_scale;
    let mut show_flag = ws.show_plugins;

    let mut action = None;

    let modal = StandardModal::new(i18n.get("plugins-title"), "plugins_manager")
        .with_hint(i18n.get("plugins-security-info"))
        .with_close_on_click_outside(false);

    modal.show(ctx, &mut show_flag, |ui| {
        // FOOTER
        action = modal.ui_footer_actions(ui, i18n, |f| {
            if f.confirm_cancel(ws) {
                return Some(PluginModalAction::Cancel);
            }
            if f.save() {
                return Some(PluginModalAction::Save);
            }
            None
        });

        // BODY
        modal.ui_body(ui, |ui| {
            let plugin_info = {
                let shared_lock = shared.lock().expect("lock");
                let plugins = shared_lock.registry.plugins.plugins.lock().expect("lock");
                plugins
                    .iter()
                    .map(|p| {
                        let mut version = None;
                        let mut desc = None;
                        let mut p_type = None;
                        if let Some(meta) = &p.metadata {
                            version = Some(meta.version.clone());
                            desc = meta.description.clone();
                            p_type = meta.plugin_type.clone();
                        }
                        (p.id.clone(), version, desc, p_type)
                    })
                    .collect::<Vec<_>>()
            };

            // 1. LEVÝ SLOUPCE (Strom)
            egui::SidePanel::left("plugins_left_tree")
                .resizable(false)
                .default_width(220.0)
                .show_inside(ui, |ui| {
                    ui.add_space(4.0);
                    ui.strong(i18n.get("plugins-list-label"));
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .id_salt("tree_scroll")
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            let is_welcome = selected_id == "system:welcome";
                            if ui
                                .selectable_label(
                                    is_welcome,
                                    format!("ℹ {}", i18n.get("plugins-item-welcome")),
                                )
                                .clicked()
                            {
                                selected_id = "system:welcome".to_string();
                            }

                            ui.add_space(4.0);

                            ui.collapsing(i18n.get("plugins-category-ai"), |ui| {
                                if ui
                                    .selectable_label(
                                        selected_id == "system:ai_settings",
                                        format!("  ⚙ {}", i18n.get("plugins-item-settings")),
                                    )
                                    .clicked()
                                {
                                    selected_id = "system:ai_settings".to_string();
                                }
                                for (id, _, _, p_type) in &plugin_info {
                                    let is_ai = p_type.as_deref() == Some("ai_agent");
                                    if is_ai {
                                        let enabled = draft
                                            .plugins
                                            .get(id)
                                            .map(|s| s.enabled)
                                            .unwrap_or_default();
                                        let icon = if enabled { "🟢" } else { "⚪" };
                                        if ui
                                            .selectable_label(
                                                selected_id == *id,
                                                format!("  {} {}", icon, id),
                                            )
                                            .clicked()
                                        {
                                            selected_id = id.clone();
                                        }
                                    }
                                }
                            });

                            ui.collapsing(i18n.get("plugins-category-general"), |ui| {
                                for (id, _, _, p_type) in &plugin_info {
                                    let is_ai = p_type.as_deref() == Some("ai_agent");
                                    if !is_ai {
                                        let enabled = draft
                                            .plugins
                                            .get(id)
                                            .map(|s| s.enabled)
                                            .unwrap_or_default();
                                        let icon = if enabled { "🟢" } else { "⚪" };
                                        if ui
                                            .selectable_label(
                                                selected_id == *id,
                                                format!("  {} {}", icon, id),
                                            )
                                            .clicked()
                                        {
                                            selected_id = id.clone();
                                        }
                                    }
                                }
                            });
                        });
                });

            // 2. PRAVÝ SLOUPCE (Obsah)
            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("details_scroll")
                    .show(ui, |ui| {
                        let w = ui.available_width();
                        if selected_id == "system:welcome" {
                            render_welcome_page(ui, i18n);
                        } else if selected_id == "system:ai_settings" {
                            ui.strong(
                                egui::RichText::new(i18n.get("plugins-item-settings")).size(20.0),
                            );
                            ui.add_space(12.0);
                            ui.strong(i18n.get("settings-ai-font"));
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                for &scale in &[100u32, 125, 150, 200] {
                                    ui.radio_value(
                                        &mut ai_font_scale,
                                        scale,
                                        format!("{}%", scale),
                                    );
                                }
                            });
                            ui.add_space(16.0);
                            render_system_ai_settings(ui, &mut draft, i18n);
                        } else if let Some((id, v, d, _)) =
                            plugin_info.iter().find(|(id, _, _, _)| *id == selected_id)
                        {
                            render_plugin_details(
                                ui,
                                w,
                                id,
                                v.as_deref(),
                                d.as_deref(),
                                &mut draft,
                                i18n,
                            );
                        }
                    });
            });
        });
    });

    ws.selected_plugin_id = Some(selected_id);
    ws.ai.settings.font_scale = ai_font_scale;
    ws.show_plugins = show_flag;

    if let Some(act) = action {
        match act {
            PluginModalAction::Save => {
                draft.save();
                let mut shared_lock = shared.lock().expect("lock");
                shared_lock.settings = std::sync::Arc::new(draft);
                shared_lock
                    .settings_version
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                ws.toasts.push(crate::app::types::Toast::info(
                    i18n.get("plugins-settings-saved"),
                ));
                ws.show_plugins = false;
                ws.plugins_draft = None;
            }
            PluginModalAction::Cancel => {
                ws.show_plugins = false;
                ws.plugins_draft = None;
            }
        }
    } else {
        ws.plugins_draft = Some(draft);
    }
}

fn render_welcome_page(ui: &mut egui::Ui, i18n: &I18n) {
    ui.strong(egui::RichText::new(i18n.get("plugins-welcome-title")).size(24.0));
    ui.add_space(16.0);
    ui.label(egui::RichText::new(i18n.get("plugins-welcome-text")).size(14.0));
    ui.add_space(24.0);
    ui.label(
        egui::RichText::new(i18n.get("plugins-welcome-hint"))
            .italics()
            .weak(),
    );
}

fn render_system_ai_settings(
    ui: &mut egui::Ui,
    draft: &mut crate::settings::Settings,
    i18n: &I18n,
) {
    ui.strong(i18n.get("settings-diff-mode"));
    ui.horizontal(|ui| {
        ui.radio_value(
            &mut draft.diff_side_by_side,
            false,
            i18n.get("settings-diff-inline"),
        );
        ui.radio_value(
            &mut draft.diff_side_by_side,
            true,
            i18n.get("settings-diff-side-by-side"),
        );
    });
    ui.add_space(16.0);

    ui.checkbox(
        &mut draft.auto_show_ai_diff,
        i18n.get("settings-auto-show-diff"),
    );
    ui.add_space(20.0);

    ui.separator();
    ui.add_space(10.0);
    ui.strong(i18n.get("settings-blacklist"));
    ui.label(
        egui::RichText::new(i18n.get("settings-blacklist-hint"))
            .weak()
            .size(11.0),
    );
    ui.add_space(6.0);

    let mut to_remove = None;
    let available_w = ui.available_width();
    egui::Frame::new()
        .fill(egui::Color32::from_black_alpha(20))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(available_w - 20.0);
            for (i, pattern) in draft.blacklist.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(pattern)
                            .desired_width(ui.available_width() - 40.0),
                    );
                    if ui.button("✖").clicked() {
                        to_remove = Some(i);
                    }
                });
            }
        });
    if let Some(idx) = to_remove {
        draft.blacklist.remove(idx);
    }
    if ui.button(i18n.get("settings-blacklist-add")).clicked() {
        draft.blacklist.push("".to_string());
    }

    ui.add_space(10.0);
    ui.label(
        egui::RichText::new(i18n.get("settings-suggested-patterns"))
            .strong()
            .size(11.0),
    );
    ui.horizontal_wrapped(|ui| {
        let suggestions = [".env", "*.key", "id_rsa", "Cargo.lock", "target/*"];
        for s in suggestions {
            if !draft.blacklist.contains(&s.to_string())
                && ui.small_button(format!("+ {}", s)).clicked()
            {
                draft.blacklist.push(s.to_string());
            }
        }
    });
}

fn render_plugin_details(
    ui: &mut egui::Ui,
    available_w: f32,
    id: &str,
    version: Option<&str>,
    desc: Option<&str>,
    draft: &mut crate::settings::Settings,
    i18n: &I18n,
) {
    ui.strong(egui::RichText::new(id).size(20.0));
    if let Some(v) = version {
        ui.label(egui::RichText::new(format!("Version: {}", v)).weak());
    }
    ui.add_space(12.0);
    if let Some(d) = desc {
        ui.label(egui::RichText::new(d).size(13.0));
    }
    ui.add_space(16.0);
    ui.separator();
    ui.add_space(16.0);

    let plugin_settings: &mut PluginSettings = draft.plugins.entry(id.to_string()).or_default();
    ui.checkbox(
        &mut plugin_settings.enabled,
        i18n.get("plugins-enabled-label"),
    );
    ui.add_space(12.0);

    if plugin_settings.enabled {
        ui.strong(i18n.get("plugins-config-label"));
        ui.add_space(8.0);
        egui::Grid::new("plugin_config_grid")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                render_config_field(
                    ui,
                    available_w,
                    plugin_settings,
                    "API_KEY",
                    &i18n.get("plugins-placeholder-api-key"),
                    id,
                );
                ui.end_row();
                render_config_field(
                    ui,
                    available_w,
                    plugin_settings,
                    "MODEL",
                    &i18n.get("plugins-placeholder-model"),
                    id,
                );
                ui.end_row();
                render_config_field(
                    ui,
                    available_w,
                    plugin_settings,
                    "API_URL",
                    "http://localhost:11434",
                    id,
                );
                ui.end_row();
                render_config_field(
                    ui,
                    available_w,
                    plugin_settings,
                    "LANGUAGE",
                    i18n.lang(),
                    id,
                );
                ui.end_row();
                render_config_field(
                    ui,
                    available_w,
                    plugin_settings,
                    "SYSTEM_PROMPT",
                    "Expert Developer",
                    id,
                );
                ui.end_row();
                if id.contains("ollama") {
                    render_config_field(ui, available_w, plugin_settings, "NUM_CTX", "4096", id);
                    ui.end_row();
                    render_config_field(ui, available_w, plugin_settings, "TEMPERATURE", "0.2", id);
                    ui.end_row();
                    render_config_field(
                        ui,
                        available_w,
                        plugin_settings,
                        "MAX_ITERATIONS",
                        "30",
                        id,
                    );
                    ui.end_row();
                    render_config_field(
                        ui,
                        available_w,
                        plugin_settings,
                        "MAX_READ_CHARS",
                        "10000",
                        id,
                    );
                    ui.end_row();
                }
            });
    }
}

fn render_config_field(
    ui: &mut egui::Ui,
    available_w: f32,
    settings: &mut PluginSettings,
    key: &str,
    label: &str,
    plugin_id: &str,
) {
    ui.label(format!("{}:", key));
    let mut value = settings.config.get(key).cloned().unwrap_or_default();
    if ui
        .add(
            egui::TextEdit::singleline(&mut value)
                .hint_text(label)
                .desired_width(available_w - 150.0)
                .id(egui::Id::new(format!("plugin_cfg_{}_{}", plugin_id, key))),
        )
        .changed()
    {
        settings.config.insert(key.to_string(), value);
    }
}
