use super::super::state::WorkspaceState;
use super::MenuActions;
use eframe::egui;

pub fn render(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    ws: &WorkspaceState,
    i18n: &crate::i18n::I18n,
) {
    ui.menu_button(i18n.get("menu-view"), |ui| {
        let files_label = format!(
            "{} {}",
            if ws.show_left_panel { "✓" } else { " " },
            i18n.get("menu-view-files")
        );
        if ui.button(files_label).clicked() {
            actions.toggle_left = true;
            ui.close_menu();
        }
        let build_label = format!(
            "{} {}",
            if ws.show_build_terminal { "✓" } else { " " },
            i18n.get("menu-view-build-terminal")
        );
        if ui.button(build_label).clicked() {
            actions.toggle_build = true;
            ui.close_menu();
        }
        let right_label = format!(
            "{} {}",
            if ws.show_right_panel { "✓" } else { " " },
            i18n.get("menu-view-ai-panel")
        );
        if ui.button(right_label).clicked() {
            actions.toggle_right = true;
            ui.close_menu();
        }
        let float_label = format!(
            "{} {}",
            if ws.claude_float { "✓" } else { " " },
            i18n.get("menu-view-ai-float")
        );
        if ui.button(float_label).clicked() {
            actions.toggle_float = true;
            ui.close_menu();
        }
    });
}
