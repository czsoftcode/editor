use crate::app::types::AppShared;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use crate::settings::PluginSettings;
use eframe::egui;
use std::sync::{Arc, Mutex};

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

    egui::Window::new(i18n.get("plugins-title"))
        .id(egui::Id::new("plugins_manager_win"))
        .collapsible(false)
        .resizable(true)
        .default_size([600.0, 400.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui: &mut egui::Ui| {
            let mut close_dialog = false;

            if let Some(draft) = &mut ws.settings_draft {
                let plugin_info = {
                    let shared_lock = shared.lock().expect("lock");
                    let plugins = shared_lock.registry.plugins.plugins.lock().expect("lock");
                    plugins
                        .iter()
                        .map(|p| {
                            let mut version = None;
                            let mut desc = None;
                            if let crate::app::registry::plugins::PluginStatus::PendingAuthorization {
                                metadata,
                                ..
                            } = &p.status
                            {
                                version = Some(metadata.version.clone());
                                desc = metadata.description.clone();
                            }
                            (p.id.clone(), version, desc)
                        })
                        .collect::<Vec<_>>()
                };

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (id, version, desc) in plugin_info {
                        let plugin_settings: &mut PluginSettings =
                            draft.plugins.entry(id.clone()).or_default();

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut plugin_settings.enabled, "");
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&id).strong().size(14.0));
                                    if let Some(v) = version {
                                        ui.label(egui::RichText::new(v).weak().size(10.0));
                                    }
                                    if let Some(d) = desc {
                                        ui.label(d);
                                    }
                                });
                            });

                            if plugin_settings.enabled {
                                ui.add_space(4.0);
                                ui.indent(id.clone() + "_config", |ui| {
                                    ui.label(i18n.get("plugins-config-label"));

                                    render_config_field(
                                        ui,
                                        plugin_settings,
                                        "API_KEY",
                                        &i18n.get("plugins-placeholder-api-key"),
                                        &id,
                                    );
                                    render_config_field(
                                        ui,
                                        plugin_settings,
                                        "MODEL",
                                        &i18n.get("plugins-placeholder-model"),
                                        &id,
                                    );
                                });
                            }
                        });
                        ui.add_space(8.0);
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(i18n.get("plugins-security-info"))
                            .weak()
                            .size(11.0),
                    );
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(i18n.get("btn-save")).clicked() {
                        draft.save();
                        let mut shared_lock = shared.lock().expect("lock");
                        shared_lock.settings = std::sync::Arc::new(draft.clone());
                        ws.toasts.push(crate::app::types::Toast::info(
                            i18n.get("plugins-settings-saved"),
                        ));
                        close_dialog = true;
                    }
                    if ui.button(i18n.get("btn-cancel")).clicked() {
                        close_dialog = true;
                    }
                });
            }

            if close_dialog {
                ws.show_plugins = false;
                ws.settings_draft = None;
            }
        });
}

fn render_config_field(
    ui: &mut egui::Ui,
    settings: &mut PluginSettings,
    key: &str,
    label: &str,
    plugin_id: &str,
) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", key));
        let mut value = settings.config.get(key).cloned().unwrap_or_default();
        if ui
            .add(
                egui::TextEdit::singleline(&mut value)
                    .hint_text(label)
                    .id(egui::Id::new(format!("plugin_cfg_{}_{}", plugin_id, key))),
            )
            .changed()
        {
            settings.config.insert(key.to_string(), value);
        }
    });
}
