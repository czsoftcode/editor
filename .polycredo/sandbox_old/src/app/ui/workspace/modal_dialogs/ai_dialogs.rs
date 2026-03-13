use crate::app::types::AppShared;
use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn show(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    _shared: &Arc<Mutex<AppShared>>,
    i18n: &I18n,
) {
    // Plugin error dialog
    if let Some(err) = ws.plugin_error.clone() {
        let mut local_show = true;
        let mut close_req = false;
        let modal = StandardModal::new(i18n.get("plugin-error-title"), "plugin_error_modal")
            .with_size(600.0, 400.0);

        modal.show(ctx, &mut local_show, |ui| {
            if let Some(c) = modal.ui_footer_actions(ui, i18n, |f| {
                if f.close() {
                    return Some(true);
                }
                None
            }) {
                close_req = c;
            }

            modal.ui_body(ui, |ui| {
                ui.label(
                    egui::RichText::new(i18n.get("plugin-error-heading"))
                        .color(egui::Color32::RED)
                        .strong(),
                );
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .id_salt("plugin_err_scroll")
                    .show(ui, |ui| {
                        let mut err_mut = err.clone();
                        ui.add(
                            egui::TextEdit::multiline(&mut err_mut)
                                .code_editor()
                                .desired_width(f32::INFINITY),
                        );
                    });
            });
        });
        if !local_show || close_req {
            ws.plugin_error = None;
        }
    }
}
