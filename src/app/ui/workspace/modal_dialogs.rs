use std::sync::{Arc, Mutex};

use crate::app::ui::background::spawn_task;

use eframe::egui;

use super::super::super::types::{AppAction, AppShared};
use super::super::dialogs::show_project_wizard;
use super::state::WorkspaceState;

// ---------------------------------------------------------------------------
// ExternalConflictAction
// ---------------------------------------------------------------------------

pub(super) enum ExternalConflictAction {
    ReloadFromDisk,
    KeepEditorVersion,
    Dismiss,
}

// ---------------------------------------------------------------------------
// render_dialogs
// ---------------------------------------------------------------------------

/// Renders modal dialogs (About, Settings, New project, Conflict).
pub(super) fn render_dialogs(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) {
    // Salt ensures uniqueness of modal ID within the egui Context, which is shared
    // between all windows (viewports). Without salt, two windows with the same dialog
    // would share state (open/closed, form values).
    let id_salt = ws.root_path.as_os_str();

    if ws.show_about {
        let modal = egui::Modal::new(egui::Id::new(("about_modal", id_salt)));
        modal.show(ctx, |ui| {
            ui.heading(i18n.get("about-title"));
            ui.add_space(8.0);
            let mut ver_args = fluent_bundle::FluentArgs::new();
            ver_args.set("version", env!("BUILD_VERSION"));
            ui.label(i18n.get_args("about-version", &ver_args));
            ui.add_space(8.0);
            ui.label(i18n.get("about-description"));
            ui.add_space(12.0);
            if ui.button(i18n.get("about-close")).clicked() {
                ws.show_about = false;
            }
        });
    }

    if ws.show_settings {
        // Initialize draft only once (at the first opening of the dialog)
        if ws.settings_draft.is_none() {
            ws.settings_draft = Some(shared.lock().unwrap().settings.clone());
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
                let mut s = shared.lock().unwrap();
                s.settings = draft;
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

    if ws.show_new_project {
        let wizard_modal_id = format!("ws_new_project_modal_{}", ws.root_path.display());
        show_project_wizard(
            ctx,
            &mut ws.wizard,
            &mut ws.show_new_project,
            &wizard_modal_id,
            shared,
            i18n,
            |path, sh| {
                let mut sh = sh.lock().unwrap();
                sh.actions.push(AppAction::AddRecent(path.clone()));
                sh.actions.push(AppAction::OpenInNewWindow(path));
            },
        );
    }

    // Dialog: file was modified externally, tab has unsaved changes.
    if let Some(conflict_path) = ws.external_change_conflict.clone() {
        let filename = conflict_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| conflict_path.to_string_lossy().into_owned());

        let mut action: Option<ExternalConflictAction> = None;

        let mut msg_args = fluent_bundle::FluentArgs::new();
        msg_args.set("name", filename.clone());

        egui::Modal::new(egui::Id::new(("external_change_conflict_modal", id_salt))).show(
            ctx,
            |ui| {
                ui.set_min_width(400.0);
                ui.heading(i18n.get("conflict-title"));
                ui.add_space(8.0);
                ui.label(i18n.get_args("conflict-message", &msg_args));
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(i18n.get("conflict-choose"))
                        .color(egui::Color32::from_rgb(180, 180, 180)),
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(i18n.get("conflict-load-disk"))
                        .on_hover_text(i18n.get("conflict-hover-disk"))
                        .clicked()
                    {
                        action = Some(ExternalConflictAction::ReloadFromDisk);
                    }
                    if ui
                        .button(i18n.get("conflict-keep-editor"))
                        .on_hover_text(i18n.get("conflict-hover-keep"))
                        .clicked()
                    {
                        action = Some(ExternalConflictAction::KeepEditorVersion);
                    }
                    if ui
                        .button(i18n.get("conflict-dismiss"))
                        .on_hover_text(i18n.get("conflict-hover-dismiss"))
                        .clicked()
                    {
                        action = Some(ExternalConflictAction::Dismiss);
                    }
                });
            },
        );

        match action {
            Some(ExternalConflictAction::ReloadFromDisk) => {
                ws.editor.reload_path_from_disk(&conflict_path);
                ws.external_change_conflict = None;
            }
            Some(ExternalConflictAction::KeepEditorVersion) => {
                // Save immediately so that file on disk matches the editor.
                ws.editor.save_path(&conflict_path, i18n);
                ws.external_change_conflict = None;
            }
            Some(ExternalConflictAction::Dismiss) => {
                ws.external_change_conflict = None;
            }
            None => {}
        }
    }

    // Dialog: confirm terminal closing when process is still running.
    if let Some(idx) = ws.terminal_close_requested {
        let mut close_confirmed = false;
        let mut cancel_requested = false;

        egui::Modal::new(egui::Id::new(("terminal_close_confirm_modal", id_salt))).show(
            ctx,
            |ui| {
                ui.set_min_width(320.0);
                ui.heading(i18n.get("terminal-close-confirm-title"));
                ui.add_space(8.0);
                ui.label(i18n.get("terminal-close-confirm-msg"));
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button(i18n.get("btn-confirm")).clicked() {
                        close_confirmed = true;
                    }
                    if ui.button(i18n.get("btn-cancel")).clicked() {
                        cancel_requested = true;
                    }
                });
            },
        );

        if close_confirmed {
            if idx < ws.claude_tabs.len() {
                ws.claude_tabs.remove(idx);
                if ws.claude_active_tab >= ws.claude_tabs.len() {
                    ws.claude_active_tab = ws.claude_tabs.len().saturating_sub(1);
                }
            }
            ws.terminal_close_requested = None;
        } else if cancel_requested {
            ws.terminal_close_requested = None;
        }
    }
}
