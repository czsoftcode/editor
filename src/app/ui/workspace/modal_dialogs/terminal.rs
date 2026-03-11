use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;

pub fn show(ctx: &egui::Context, ws: &mut WorkspaceState, i18n: &I18n, _id_salt: &std::ffi::OsStr) {
    if let Some(idx) = ws.terminal_close_requested {
        let mut close_confirmed = false;
        let mut cancel_requested = false;
        let mut show_flag = true;

        let modal = StandardModal::new(
            i18n.get("terminal-close-confirm-title"),
            "terminal_close_modal",
        )
        .with_size(400.0, 250.0);

        modal.show(ctx, &mut show_flag, |ui| {
            // FOOTER
            modal.ui_footer_actions(ui, i18n, |f| {
                if f.close() || f.cancel() {
                    cancel_requested = true;
                }
                if f.button("btn-confirm").clicked() {
                    close_confirmed = true;
                }
                None::<()>
            });

            // BODY
            modal.ui_body(ui, |ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(i18n.get("terminal-close-confirm-msg")).size(14.0));
                ui.add_space(16.0);
            });
        });

        if !show_flag {
            cancel_requested = true;
        }

        if close_confirmed {
            if let Some(_terminal) = ws.claude_tabs.get(idx) {
                #[cfg(unix)]
                _terminal.kill_process_group();
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
