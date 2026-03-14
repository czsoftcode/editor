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
    ui.menu_button(i18n.get("menu-edit"), |ui| {
        // Clipboard a select-all — zpracovává TextEdit sám, jen zobrazujeme labely.
        // Tyto nejsou v keymapu, hardcoded labely jsou OK.
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
        // Find/Replace/Goto — napojené na MenuActions flagy, dynamické shortcut labely z keymapu.
        if ui
            .add(
                egui::Button::new(i18n.get("menu-edit-find"))
                    .shortcut_text(shortcut_label(keymap, CommandId::Find)),
            )
            .clicked()
        {
            actions.find = true;
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new(i18n.get("menu-edit-replace"))
                    .shortcut_text(shortcut_label(keymap, CommandId::Replace)),
            )
            .clicked()
        {
            actions.replace = true;
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new(i18n.get("menu-edit-goto-line"))
                    .shortcut_text(shortcut_label(keymap, CommandId::GotoLine)),
            )
            .clicked()
        {
            actions.goto_line = true;
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new(i18n.get("menu-edit-open-file"))
                    .shortcut_text(shortcut_label(keymap, CommandId::OpenFile)),
            )
            .clicked()
        {
            actions.open_file_picker = true;
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new(i18n.get("menu-edit-project-search"))
                    .shortcut_text(shortcut_label(keymap, CommandId::ProjectSearch)),
            )
            .clicked()
        {
            actions.project_search = true;
            ui.close_menu();
        }
        ui.separator();
        if ui
            .add(
                egui::Button::new(i18n.get("menu-edit-build"))
                    .shortcut_text(shortcut_label(keymap, CommandId::Build)),
            )
            .clicked()
        {
            actions.build = true;
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new(i18n.get("menu-edit-run"))
                    .shortcut_text(shortcut_label(keymap, CommandId::Run)),
            )
            .clicked()
        {
            actions.run = true;
            ui.close_menu();
        }
        ui.separator();
        if ui
            .add(
                egui::Button::new(i18n.get("menu-edit-command-palette"))
                    .shortcut_text(shortcut_label(keymap, CommandId::CommandPalette)),
            )
            .clicked()
        {
            actions.command_palette = true;
            ui.close_menu();
        }
    });
}
