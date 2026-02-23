use crate::app::types::AppShared;
use crate::app::ui::background::spawn_task;
use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub enum SettingsModalAction {
    Save,
    Cancel,
}

pub fn show(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
    _id_salt: &std::ffi::OsStr,
) {
    if !ws.show_settings {
        return;
    }

    if ws.settings_draft.is_none() {
        ws.settings_draft = Some((*shared.lock().expect("lock").settings).clone());
    }

    if let Some(rx) = ws.settings_folder_pick_rx.as_ref()
        && let Ok(picked) = rx.try_recv()
    {
        ws.settings_folder_pick_rx = None;
        if let Some(dir) = picked
            && let Some(draft) = ws.settings_draft.as_mut()
        {
            draft.default_project_path = dir.to_string_lossy().to_string();
        }
    }

    let mut action = None;
    let mut browse_requested = false;
    let mut selected_cat = ws.selected_settings_category.clone().unwrap_or_else(|| "general".to_string());

    StandardModal::new(i18n.get("settings-title"), "main_settings").show(
        ctx,
        &mut ws.show_settings,
        |ui| {
            let draft = ws.settings_draft.as_mut().unwrap();

            // 1. LEFT PANEL (Tree/Categories)
            egui::SidePanel::left("settings_left_tree")
                .resizable(false)
                .default_width(180.0)
                .show_inside(ui, |ui| {
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical().id_salt("settings_cat_scroll").show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        if ui.selectable_label(selected_cat == "general", format!("⚙ {}", i18n.get("settings-category-general"))).clicked() {
                            selected_cat = "general".to_string();
                        }
                        if ui.selectable_label(selected_cat == "editor", format!("📝 {}", i18n.get("settings-category-editor"))).clicked() {
                            selected_cat = "editor".to_string();
                        }
                    });
                });

            // 2. RIGHT PANEL (Content)
            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical().id_salt("settings_content_scroll").show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    if selected_cat == "general" {
                        ui.strong(egui::RichText::new(i18n.get("settings-category-general")).size(18.0));
                        ui.add_space(12.0);

                        ui.strong(i18n.get("settings-language"));
                        ui.add_space(4.0);
                        egui::ComboBox::from_id_salt("settings_lang_combo")
                            .selected_text(crate::i18n::lang_display_name(&draft.lang))
                            .width(160.0)
                            .show_ui(ui, |ui| {
                                for &lang in crate::i18n::SUPPORTED_LANGS {
                                    if ui
                                        .selectable_label(
                                            draft.lang == lang,
                                            crate::i18n::lang_display_name(lang),
                                        )
                                        .clicked()
                                    {
                                        draft.lang = lang.to_string();
                                    }
                                }
                            });
                        ui.add_space(16.0);

                        ui.strong(i18n.get("settings-default-path"));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.default_project_path)
                                    .desired_width(ui.available_width() - 40.0),
                            );
                            if ui.button("…").clicked() {
                                browse_requested = true;
                            }
                        });
                    } else if selected_cat == "editor" {
                        ui.strong(egui::RichText::new(i18n.get("settings-category-editor")).size(18.0));
                        ui.add_space(12.0);

                        ui.strong(i18n.get("settings-theme"));
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut draft.dark_theme, true, i18n.get("settings-theme-dark"));
                            ui.radio_value(
                                &mut draft.dark_theme,
                                false,
                                i18n.get("settings-theme-light"),
                            );
                        });
                        ui.add_space(16.0);

                        ui.strong(i18n.get("settings-editor-font"));
                        ui.add_space(4.0);
                        ui.add(
                            egui::Slider::new(&mut draft.editor_font_size, 10.0..=24.0)
                                .step_by(1.0)
                                .suffix(" px"),
                        );
                    }
                });
            });
        },
        |ui| {
            if ui.button(i18n.get("btn-cancel")).clicked() {
                action = Some(SettingsModalAction::Cancel);
            }
            if ui.button(i18n.get("btn-save")).clicked() {
                action = Some(SettingsModalAction::Save);
            }
            None::<()>
        },
    );

    ws.selected_settings_category = Some(selected_cat);

    if browse_requested && ws.settings_folder_pick_rx.is_none() {
        let start_dir = ws
            .settings_draft
            .as_ref()
            .map(|d| d.default_project_path.clone())
            .unwrap_or_default();
        ws.settings_folder_pick_rx = Some(spawn_task(move || {
            let d = rfd::FileDialog::new();
            if start_dir.trim().is_empty() {
                d.pick_folder()
            } else {
                d.set_directory(start_dir).pick_folder()
            }
        }));
    }

    if let Some(act) = action {
        match act {
            SettingsModalAction::Save => {
                if let Some(draft) = ws.settings_draft.take() {
                    draft.save();
                    ws.wizard.path = draft.default_project_path.clone();
                    let lang = draft.lang.clone();
                    let mut s = shared.lock().expect("lock");
                    s.settings = Arc::new(draft);
                    s.i18n = Arc::new(crate::i18n::I18n::new(&lang));
                    s.settings_version
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
                ws.show_settings = false;
            }
            SettingsModalAction::Cancel => {
                ws.settings_draft = None;
                ws.show_settings = false;
            }
        }
    }
}
