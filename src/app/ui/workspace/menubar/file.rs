use super::MenuActions;
use super::shortcut_label;
use crate::app::keymap::Keymap;
use crate::app::ui::widgets::command_palette::CommandId;
use eframe::egui;

pub fn render(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    i18n: &crate::i18n::I18n,
    keymap: &Keymap,
) {
    ui.menu_button(i18n.get("menu-file"), |ui| {
        if ui.button(i18n.get("menu-file-open-folder")).clicked() {
            actions.open_folder = true;
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new(i18n.get("menu-file-save"))
                    .shortcut_text(shortcut_label(keymap, CommandId::Save)),
            )
            .clicked()
        {
            actions.save = true;
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new(i18n.get("menu-file-close-tab"))
                    .shortcut_text(shortcut_label(keymap, CommandId::CloseTab)),
            )
            .clicked()
        {
            actions.close_file = true;
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new(i18n.get("menu-file-settings"))
                    .shortcut_text(shortcut_label(keymap, CommandId::Settings)),
            )
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
