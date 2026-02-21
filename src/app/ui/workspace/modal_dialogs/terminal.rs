use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;

pub fn show(ctx: &egui::Context, ws: &mut WorkspaceState, i18n: &I18n, id_salt: &std::ffi::OsStr) {
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
