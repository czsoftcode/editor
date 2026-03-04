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
    let mut selected_cat = ws
        .selected_settings_category
        .clone()
        .unwrap_or_else(|| "general".to_string());
    let mut show_flag = ws.show_settings;

    let modal = StandardModal::new(i18n.get("settings-title"), "main_settings");

    modal.show(ctx, &mut show_flag, |ui| {
        // FOOTER
        action = modal.ui_footer_actions(ui, i18n, |f| {
            if f.confirm_cancel(ws) {
                return Some(SettingsModalAction::Cancel);
            }
            if f.save() {
                return Some(SettingsModalAction::Save);
            }
            None
        });

        // BODY
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let draft = ws.settings_draft.as_mut().unwrap();

            // 1. LEFT PANEL (Tree/Categories)
            egui::SidePanel::left("settings_left_tree")
                .resizable(false)
                .default_width(180.0)
                .show_inside(ui, |ui| {
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical()
                        .id_salt("settings_cat_scroll")
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            if ui
                                .selectable_label(
                                    selected_cat == "general",
                                    format!("⚙ {}", i18n.get("settings-category-general")),
                                )
                                .clicked()
                            {
                                selected_cat = "general".to_string();
                            }
                            if ui
                                .selectable_label(
                                    selected_cat == "editor",
                                    format!("📝 {}", i18n.get("settings-category-editor")),
                                )
                                .clicked()
                            {
                                selected_cat = "editor".to_string();
                            }
                            if ui
                                .selectable_label(
                                    selected_cat == "ai",
                                    format!("🤖 {}", i18n.get("settings-category-ai")),
                                )
                                .clicked()
                            {
                                selected_cat = "ai".to_string();
                            }
                        });
                });

            // 2. RIGHT PANEL (Content)
            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("settings_content_scroll")
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        if selected_cat == "general" {
                            ui.strong(
                                egui::RichText::new(i18n.get("settings-category-general"))
                                    .size(18.0),
                            );
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
                            ui.add_space(20.0);

                            ui.separator();
                            ui.add_space(10.0);
                            ui.checkbox(
                                &mut draft.project_read_only,
                                i18n.get("settings-safe-mode"),
                            );
                            ui.label(
                                egui::RichText::new(i18n.get("settings-safe-mode-hint"))
                                    .small()
                                    .weak(),
                            );
                        } else if selected_cat == "editor" {
                            ui.strong(
                                egui::RichText::new(i18n.get("settings-category-editor"))
                                    .size(18.0),
                            );
                            ui.add_space(12.0);

                            ui.strong(i18n.get("settings-theme"));
                            ui.horizontal(|ui| {
                                ui.radio_value(
                                    &mut draft.dark_theme,
                                    true,
                                    i18n.get("settings-theme-dark"),
                                );
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
                        } else if selected_cat == "ai" {
                            ui.strong(
                                egui::RichText::new(i18n.get("settings-category-ai")).size(18.0),
                            );
                            ui.add_space(12.0);

                            ui.label(i18n.get("settings-ai-hint"));
                            ui.add_space(12.0);

                            let mut to_remove = None;
                            for (idx, agent) in draft.custom_agents.iter_mut().enumerate() {
                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.label(i18n.get("settings-ai-name"));
                                                ui.add(
                                                    egui::TextEdit::singleline(&mut agent.name)
                                                        .hint_text("Gemini"),
                                                );
                                            });
                                            ui.vertical(|ui| {
                                                ui.label(i18n.get("settings-ai-command"));
                                                ui.add(
                                                    egui::TextEdit::singleline(&mut agent.command)
                                                        .hint_text("gemini"),
                                                );
                                            });
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.button("🗑").clicked() {
                                                        to_remove = Some(idx);
                                                    }
                                                },
                                            );
                                        });

                                        ui.add_space(4.0);

                                        ui.vertical(|ui| {
                                            ui.label(i18n.get("settings-ai-args"));
                                            ui.add(
                                                egui::TextEdit::singleline(&mut agent.args)
                                                    .hint_text("--chat")
                                                    .desired_width(ui.available_width()),
                                            );
                                        });
                                    });
                                });
                                ui.add_space(8.0);
                            }

                            if let Some(idx) = to_remove {
                                draft.custom_agents.remove(idx);
                            }

                            if ui
                                .button(format!("+ {}", i18n.get("settings-ai-add")))
                                .clicked()
                            {
                                draft.custom_agents.push(crate::settings::CustomAgent {
                                    name: "".to_string(),
                                    command: "".to_string(),
                                    args: "".to_string(),
                                });
                            }
                        }
                    });
            });
        });
    });

    ws.selected_settings_category = Some(selected_cat);
    ws.show_settings = show_flag;

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

                    // Immediate agent registry update
                    s.registry.agents.clear();
                    for ca in &draft.custom_agents {
                        let cmd = if ca.args.is_empty() {
                            ca.command.clone()
                        } else {
                            format!("{} {}", ca.command, ca.args)
                        };
                        s.registry.agents.register(crate::app::registry::Agent {
                            id: ca.name.to_lowercase().replace(' ', "_"),
                            label: ca.name.clone(),
                            command: cmd,
                            context_aware: true,
                        });
                    }

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
