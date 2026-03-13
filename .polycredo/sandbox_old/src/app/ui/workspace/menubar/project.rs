use super::MenuActions;
use eframe::egui;
use std::path::PathBuf;

pub fn render(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    recent_snapshot: &[PathBuf],
    i18n: &crate::i18n::I18n,
) {
    ui.menu_button(i18n.get("menu-project"), |ui| {
        if ui.button(i18n.get("menu-project-open")).clicked() {
            actions.open_project = true;
            ui.close_menu();
        }
        if ui.button(i18n.get("menu-project-new")).clicked() {
            actions.new_project = true;
            ui.close_menu();
        }
        if !recent_snapshot.is_empty() {
            ui.separator();
            ui.menu_button(i18n.get("menu-project-recent"), |ui| {
                for path in recent_snapshot {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.to_string_lossy().into_owned());
                    if ui
                        .button(&name)
                        .on_hover_text(path.to_string_lossy())
                        .clicked()
                    {
                        actions.open_recent = Some(path.clone());
                        ui.close_menu();
                    }
                }
            });
        }
    });
}
