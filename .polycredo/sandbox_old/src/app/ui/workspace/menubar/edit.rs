use super::MenuActions;
use eframe::egui;

pub fn render(ui: &mut egui::Ui, actions: &mut MenuActions, i18n: &crate::i18n::I18n) {
    ui.menu_button(i18n.get("menu-edit"), |ui| {
        ui.add_enabled(
            false,
            egui::Button::new(i18n.get("menu-edit-copy")).shortcut_text("Ctrl+C"),
        );
        ui.add_enabled(
            false,
            egui::Button::new(i18n.get("menu-edit-paste")).shortcut_text("Ctrl+V"),
        );
        ui.add_enabled(
            false,
            egui::Button::new(i18n.get("menu-edit-select-all")).shortcut_text("Ctrl+A"),
        );
        ui.separator();
        if ui
            .add(egui::Button::new(i18n.get("menu-edit-find")).shortcut_text("Ctrl+F"))
            .clicked()
        {
            ui.close_menu();
        }
        if ui
            .add(egui::Button::new(i18n.get("menu-edit-replace")).shortcut_text("Ctrl+H"))
            .clicked()
        {
            ui.close_menu();
        }
        if ui
            .add(egui::Button::new(i18n.get("menu-edit-goto-line")).shortcut_text("Ctrl+G"))
            .clicked()
        {
            ui.close_menu();
        }
        if ui
            .add(egui::Button::new(i18n.get("menu-edit-open-file")).shortcut_text("Ctrl+P"))
            .clicked()
        {
            actions.open_file_picker = true;
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new(i18n.get("menu-edit-project-search"))
                    .shortcut_text("Ctrl+Shift+F"),
            )
            .clicked()
        {
            actions.project_search = true;
            ui.close_menu();
        }
        ui.separator();
        if ui
            .add(egui::Button::new(i18n.get("menu-edit-build")).shortcut_text("Ctrl+B"))
            .clicked()
        {
            actions.build = true;
            ui.close_menu();
        }
        if ui
            .add(egui::Button::new(i18n.get("menu-edit-run")).shortcut_text("Ctrl+R"))
            .clicked()
        {
            actions.run = true;
            ui.close_menu();
        }
    });
}
