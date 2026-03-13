use super::MenuActions;
use eframe::egui;

pub fn render(ui: &mut egui::Ui, actions: &mut MenuActions, i18n: &crate::i18n::I18n) {
    ui.menu_button(i18n.get("menu-help"), |ui| {
        if ui.button(i18n.get("menu-help-about")).clicked() {
            actions.about = true;
            ui.close_menu();
        }
        ui.separator();
        if ui
            .button(format!("❤️ {}", i18n.get("menu-help-support")))
            .clicked()
        {
            actions.support = true;
            ui.close_menu();
        }
    });
}
