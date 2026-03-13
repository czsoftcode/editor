use super::MenuActions;
use eframe::egui;

pub fn render(ui: &mut egui::Ui, actions: &mut MenuActions, i18n: &crate::i18n::I18n) {
    ui.menu_button(i18n.get("menu-file"), |ui| {
        if ui.button(i18n.get("menu-file-open-folder")).clicked() {
            actions.open_folder = true;
            ui.close_menu();
        }
        if ui
            .add(egui::Button::new(i18n.get("menu-file-save")).shortcut_text("Ctrl+S"))
            .clicked()
        {
            actions.save = true;
            ui.close_menu();
        }
        if ui
            .add(egui::Button::new(i18n.get("menu-file-close-tab")).shortcut_text("Ctrl+W"))
            .clicked()
        {
            actions.close_file = true;
            ui.close_menu();
        }
        if ui
            .add(egui::Button::new(i18n.get("menu-file-settings")).shortcut_text("Ctrl+,"))
            .clicked()
        {
            actions.settings = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.button(i18n.get("menu-file-quit")).clicked() {
            actions.quit = true;
            ui.close_menu();
        }
    });
}
