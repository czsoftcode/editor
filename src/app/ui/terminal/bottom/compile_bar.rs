#[cfg(target_os = "linux")]
use crate::app::types::FocusedPanel;
use crate::app::ui::workspace::state::WorkspaceState;
use eframe::egui;

pub fn render_compile_bar(ui: &mut egui::Ui, ws: &mut WorkspaceState, _i18n: &crate::i18n::I18n) {
    ui.horizontal(|ui| {
        ui.strong("Compile");
        ui.separator();

        ui.add_enabled_ui(true, |ui| {
            #[cfg(target_os = "linux")]
            {
                if ui
                    .button(_i18n.get("btn-create-deb"))
                    .on_hover_text(_i18n.get("hover-create-deb"))
                    .clicked()
                {
                    let cmd = "export DEB_BUILD_TYPE=deb-dev && ./packaging/deb/build-deb.sh";
                    ws.next_terminal_id += 1;
                    let terminal = crate::app::ui::terminal::instance::Terminal::new(
                        ws.next_terminal_id,
                        ui.ctx(),
                        &ws.root_path,
                        Some(cmd),
                    );
                    ws.build_terminal = Some(terminal);
                    ws.show_build_terminal = true;
                    ws.focused_panel = FocusedPanel::Build;
                }
            }

            #[cfg(target_os = "windows")]
            {
                ui.weak("MSI Installer (WIP)");
            }

            #[cfg(target_os = "macos")]
            {
                ui.weak("DMG Bundle (WIP)");
            }
        });

    });
}
