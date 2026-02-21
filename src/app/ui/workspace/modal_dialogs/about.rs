use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;

pub fn show(ctx: &egui::Context, ws: &mut WorkspaceState, i18n: &I18n, id_salt: &std::ffi::OsStr) {
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
}
