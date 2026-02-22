use crate::app::types::AppShared;
use crate::app::ui::background::spawn_task;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn show(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
    id_salt: &std::ffi::OsStr,
) {
    if ws.show_settings {
        // Initialize draft only once (at the first opening of the dialog)
        if ws.settings_draft.is_none() {
            ws.settings_draft = Some(
                (*shared
                    .lock()
                    .expect("Failed to lock AppShared for settings draft initialization")
                    .settings)
                    .clone(),
            );
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

        let mut do_save = false;
        let mut do_close = false;
        let mut request_settings_browse = false;
        let mut browse_start_dir: Option<String> = None;

        let modal = egui::Modal::new(egui::Id::new(("settings_modal", id_salt)));
        modal.show(ctx, |ui| {
            ui.heading(i18n.get("settings-title"));
            ui.add_space(10.0);

            let draft = ws.settings_draft.as_mut().unwrap();

            // Language
            ui.strong(i18n.get("settings-language"));
            ui.add_space(4.0);
            egui::ComboBox::from_id_salt("settings_lang_combo")
                .selected_text(crate::i18n::lang_display_name(&draft.lang))
                .width(160.0)
                .show_ui(ui, |ui| {
                    for &lang in crate::i18n::SUPPORTED_LANGS {
                        let is_selected = draft.lang == lang;
                        if ui
                            .selectable_label(is_selected, crate::i18n::lang_display_name(lang))
                            .clicked()
                        {
                            draft.lang = lang.to_string();
                        }
                    }
                });
            ui.add_space(10.0);

            // Theme
            ui.strong(i18n.get("settings-theme"));
            ui.horizontal(|ui| {
                ui.radio_value(&mut draft.dark_theme, true, i18n.get("settings-theme-dark"));
                ui.radio_value(
                    &mut draft.dark_theme,
                    false,
                    i18n.get("settings-theme-light"),
                );
            });
            ui.add_space(10.0);

            // Diff Mode
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
            ui.add_space(10.0);

            // Auto-show AI Diff
            ui.checkbox(
                &mut draft.auto_show_ai_diff,
                i18n.get("settings-auto-show-diff"),
            );
            ui.add_space(10.0);

            // Editor font
            ui.strong(i18n.get("settings-editor-font"));
            ui.add_space(4.0);
            ui.add(
                egui::Slider::new(&mut draft.editor_font_size, 10.0..=24.0)
                    .step_by(1.0)
                    .suffix(" px")
                    .clamping(egui::SliderClamping::Always),
            );
            ui.add_space(10.0);

            // AI terminal font scale (per-workspace, outside global settings)
            ui.strong(i18n.get("settings-ai-font"));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for &scale in &[100u32, 125, 150, 200] {
                    ui.radio_value(&mut ws.ai_font_scale, scale, format!("{}%", scale));
                }
            });
            ui.add_space(10.0);

            // Default projects path
            ui.strong(i18n.get("settings-default-path"));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut draft.default_project_path)
                        .desired_width(280.0),
                );
                if ui.button("…").clicked() {
                    request_settings_browse = true;
                    browse_start_dir = Some(draft.default_project_path.clone());
                }
            });
            ui.add_space(14.0);

            ui.horizontal(|ui| {
                if ui.button(i18n.get("btn-save")).clicked() {
                    do_save = true;
                }
                if ui.button(i18n.get("btn-close")).clicked() {
                    do_close = true;
                }
            });
        });

        if request_settings_browse && ws.settings_folder_pick_rx.is_none() {
            let start_dir = browse_start_dir.unwrap_or_default();
            ws.settings_folder_pick_rx = Some(spawn_task(move || {
                let dialog = rfd::FileDialog::new();
                if start_dir.trim().is_empty() {
                    dialog.pick_folder()
                } else {
                    dialog.set_directory(start_dir).pick_folder()
                }
            }));
        }

        if do_save {
            let draft = ws.settings_draft.take().unwrap();
            draft.save();
            ws.wizard.path = draft.default_project_path.clone();
            let new_lang = draft.lang.clone();
            {
                let mut s = shared
                    .lock()
                    .expect("Failed to lock AppShared for saving settings");
                s.settings = std::sync::Arc::new(draft);
                s.i18n = std::sync::Arc::new(crate::i18n::I18n::new(&new_lang));
            }
            ws.show_settings = false;
            ws.settings_folder_pick_rx = None;
        } else if do_close {
            ws.settings_draft = None;
            ws.show_settings = false;
            ws.settings_folder_pick_rx = None;
        }
    }
}
