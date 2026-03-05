use crate::app::types::AppShared;
use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub enum ExternalConflictAction {
    ReloadFromDisk,
    KeepEditorVersion,
    Dismiss,
}

enum SettingsConflictAction {
    ReloadSettings,
    KeepEditing,
}

pub fn show(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
    id_salt: &std::ffi::OsStr,
) {
    if let Some(conflict_path) = ws.external_change_conflict.clone() {
        let filename = conflict_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| conflict_path.to_string_lossy().into_owned());

        let mut action: Option<ExternalConflictAction> = None;
        let mut show_flag = true;

        let mut msg_args = fluent_bundle::FluentArgs::new();
        msg_args.set("name", filename.clone());

        let modal = StandardModal::new(i18n.get("conflict-title"), "conflict_modal")
            .with_size(500.0, 300.0);

        modal.show(ctx, &mut show_flag, |ui| {
            // FOOTER
            action = modal.ui_footer_actions(ui, i18n, |f| {
                if f.close()
                    || f.cancel()
                    || f.button("conflict-dismiss")
                        .on_hover_text(i18n.get("conflict-hover-dismiss"))
                        .clicked()
                {
                    return Some(ExternalConflictAction::Dismiss);
                }
                if f.button("conflict-load-disk")
                    .on_hover_text(i18n.get("conflict-hover-disk"))
                    .clicked()
                {
                    return Some(ExternalConflictAction::ReloadFromDisk);
                }
                if f.button("conflict-keep-editor")
                    .on_hover_text(i18n.get("conflict-hover-keep"))
                    .clicked()
                {
                    return Some(ExternalConflictAction::KeepEditorVersion);
                }
                None
            });

            // BODY
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(i18n.get_args("conflict-message", &msg_args)).size(14.0),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(i18n.get("conflict-choose"))
                        .color(egui::Color32::from_rgb(180, 180, 180)),
                );
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);
            });
        });

        if !show_flag {
            ws.external_change_conflict = None;
        }

        match action {
            Some(ExternalConflictAction::ReloadFromDisk) => {
                ws.editor.reload_path_from_disk(&conflict_path);
                ws.external_change_conflict = None;
            }
            Some(ExternalConflictAction::KeepEditorVersion) => {
                ws.editor.save_path(
                    &conflict_path,
                    i18n,
                    &shared
                        .lock()
                        .expect("Failed to lock AppShared for conflict resolution save")
                        .is_internal_save,
                    ws.sandbox_mode_enabled,
                );
                ws.external_change_conflict = None;
            }
            Some(ExternalConflictAction::Dismiss) => {
                ws.external_change_conflict = None;
            }
            None => {}
        }
    }

    if let Some(conflict) = ws.settings_conflict.clone() {
        let mut action: Option<SettingsConflictAction> = None;
        let mut show_flag = true;
        let modal = StandardModal::new(
            i18n.get("settings-conflict-title"),
            format!("settings_conflict_modal_{}", id_salt.to_string_lossy()),
        )
        .with_size(520.0, 240.0);

        modal.show(ctx, &mut show_flag, |ui| {
            action = modal.ui_footer_actions(ui, i18n, |f| {
                if f.button("settings-conflict-keep").clicked() || f.close() {
                    return Some(SettingsConflictAction::KeepEditing);
                }
                if f.button("settings-conflict-reload").clicked() {
                    return Some(SettingsConflictAction::ReloadSettings);
                }
                None
            });

            modal.ui_body(ui, |ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(i18n.get("settings-conflict-message"))
                        .size(14.0)
                        .line_height(Some(20.0)),
                );
            });
        });

        if !show_flag {
            ws.settings_conflict = None;
        }

        match action {
            Some(SettingsConflictAction::ReloadSettings) => {
                ws.settings_draft = Some(conflict.new_settings.clone());
                ws.settings_original = Some(conflict.new_settings);
                ws.settings_conflict = None;
            }
            Some(SettingsConflictAction::KeepEditing) => {
                ws.settings_conflict = None;
            }
            None => {}
        }
    }
}
