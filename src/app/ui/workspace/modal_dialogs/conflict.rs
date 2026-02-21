use crate::app::types::AppShared;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub enum ExternalConflictAction {
    ReloadFromDisk,
    KeepEditorVersion,
    Dismiss,
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
                ws.editor.save_path(
                    &conflict_path,
                    i18n,
                    &shared.lock().unwrap().is_internal_save,
                );
                ws.external_change_conflict = None;
            }
            Some(ExternalConflictAction::Dismiss) => {
                ws.external_change_conflict = None;
            }
            None => {}
        }
    }
}
