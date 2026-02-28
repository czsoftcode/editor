use crate::app::ui::widgets::modal::StandardModal;
use eframe::egui;

pub(crate) fn show_support_dialog(
    ctx: &egui::Context,
    ws: &mut crate::app::ui::workspace::state::WorkspaceState,
    i18n: &crate::i18n::I18n,
) {
    if !ws.show_support {
        return;
    }

    let mut local_show = ws.show_support;
    let modal = StandardModal::new(
        format!("❤️ {}", i18n.get("support-modal-title")),
        "support_modal",
    )
    .with_size(550.0, 400.0);

    let mut close_requested = false;

    modal.show(ctx, &mut local_show, |ui| {
        // FOOTER
        modal.ui_footer(ui, |ui| {
            if ui.button(i18n.get("btn-close")).clicked() {
                close_requested = true;
            }
            None::<()>
        });

        // BODY
        modal.ui_body(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Icon / Visual element
                ui.label(egui::RichText::new("🚀").size(64.0));
                ui.add_space(15.0);

                // Headline
                ui.heading(egui::RichText::new(i18n.get("support-modal-title")).strong());
                ui.add_space(15.0);

                // Description text
                ui.label(
                    egui::RichText::new(i18n.get("support-modal-body"))
                        .line_height(Some(22.0))
                        .size(15.0)
                        .color(ui.visuals().widgets.noninteractive.text_color()),
                );

                ui.add_space(30.0);

                // Action Buttons
                ui.horizontal(|ui| {
                    let btn_width = 200.0;
                    let spacing = 20.0;
                    let total_width = btn_width * 2.0 + spacing;
                    let start_x = (ui.available_width() - total_width) / 2.0;
                    ui.add_space(start_x.max(0.0));

                    // GitHub Button
                    let github_btn = ui.add_sized(
                        [btn_width, 40.0],
                        egui::Button::new(format!("⭐ {}", i18n.get("support-modal-github"))),
                    );
                    if github_btn.clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(
                            "https://github.com/czsoftcode/editor",
                        ));
                    }

                    ui.add_space(spacing);

                    // Donate Button
                    let donate_btn = ui.add_sized(
                        [btn_width, 40.0],
                        egui::Button::new(format!("🎁 {}", i18n.get("support-modal-donate")))
                            .fill(egui::Color32::from_rgb(180, 100, 30)),
                    );
                    if donate_btn.clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(
                            "https://github.com/czsoftcode/editor",
                        )); // TODO: Specific donation page
                    }
                });

                ui.add_space(20.0);
            });
        });
    });

    if close_requested {
        local_show = false;
    }
    ws.show_support = local_show;
}
