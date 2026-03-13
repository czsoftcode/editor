use crate::app::ui::widgets::modal::StandardModal;
use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;

pub fn show(ctx: &egui::Context, ws: &mut WorkspaceState, i18n: &I18n, _id_salt: &std::ffi::OsStr) {
    if !ws.show_about {
        return;
    }

    let mut local_show = ws.show_about;
    let mut close_requested = false;

    let modal = StandardModal::new(i18n.get("about-title"), "about_modal").with_size(450.0, 350.0);

    modal.show(ctx, &mut local_show, |ui| {
        // FOOTER
        modal.ui_footer_actions(ui, i18n, |f| {
            if f.close() {
                close_requested = true;
            }
            None::<()>
        });

        // BODY
        modal.ui_body(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("PolyCredo Editor")
                        .size(24.0)
                        .strong()
                        .color(egui::Color32::from_rgb(100, 150, 255)),
                );
                ui.add_space(8.0);

                let mut ver_args = fluent_bundle::FluentArgs::new();
                ver_args.set("version", env!("BUILD_VERSION"));
                ui.label(egui::RichText::new(i18n.get_args("about-version", &ver_args)).size(14.0));

                let mut build_args = fluent_bundle::FluentArgs::new();
                build_args.set("build", env!("BUILD_NUMBER"));
                ui.label(egui::RichText::new(i18n.get_args("about-build", &build_args)).weak());

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);

                ui.label(egui::RichText::new(i18n.get("about-description")).size(13.0));
                ui.add_space(12.0);
            });
        });
    });

    if !local_show || close_requested {
        ws.show_about = false;
    }
}
