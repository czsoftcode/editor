use super::MenuActions;
use crate::app::types::AppShared;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn render(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) {
    ui.menu_button(i18n.get("menu-file"), |ui| {
        if ui.button(i18n.get("menu-file-open-folder")).clicked() {
            actions.open_folder = true;
            ui.close_menu();
        }
        if ui
            .add(egui::Button::new(i18n.get("menu-file-save")).shortcut_text("Ctrl+S"))
            .clicked()
        {
            actions.save = true;
            ui.close_menu();
        }
        if ui
            .add(egui::Button::new(i18n.get("menu-file-close-tab")).shortcut_text("Ctrl+W"))
            .clicked()
        {
            actions.close_file = true;
            ui.close_menu();
        }
        ui.menu_button(i18n.get("menu-file-plugins"), |ui| {
            if ui
                .add(
                    egui::Button::new(i18n.get("menu-file-plugins-manager"))
                        .shortcut_text("Ctrl+Shift+L"),
                )
                .clicked()
            {
                actions.plugins = true;
                ui.close_menu();
            }
            ui.separator();

            let plugins = {
                let shared_lock = shared.lock().expect("lock");
                let p_list = shared_lock.registry.plugins.plugins.lock().expect("lock");
                p_list
                    .iter()
                    .map(|p| {
                        (
                            p.id.clone(),
                            p.metadata
                                .as_ref()
                                .map(|m| m.name.clone())
                                .unwrap_or_else(|| p.id.clone()),
                            p.metadata
                                .as_ref()
                                .and_then(|m| m.plugin_type.clone())
                                .unwrap_or_default(),
                        )
                    })
                    .collect::<Vec<_>>()
            };

            ui.menu_button(i18n.get("plugins-category-ai"), |ui| {
                for (id, name, p_type) in &plugins {
                    if p_type == "ai_agent" && ui.button(name).clicked() {
                        actions.run_agent = Some("ai_chat".to_string());
                        actions.run_plugin = Some((id.clone(), "OPEN_AI_CHAT".to_string()));
                        ui.close_menu();
                    }
                }
            });

            ui.menu_button(i18n.get("plugins-category-general"), |ui| {
                for (id, name, p_type) in &plugins {
                    if p_type != "ai_agent" && ui.button(name).clicked() {
                        actions.run_plugin = Some((id.clone(), id.clone()));
                        ui.close_menu();
                    }
                }
            });
        });
        if ui
            .add(egui::Button::new(i18n.get("menu-file-settings")).shortcut_text("Ctrl+,"))
            .clicked()
        {
            actions.settings = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.button(i18n.get("menu-file-quit")).clicked() {
            actions.quit = true;
            ui.close_menu();
        }
    });
}
