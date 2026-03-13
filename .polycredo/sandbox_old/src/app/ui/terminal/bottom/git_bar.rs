use crate::app::ui::workspace::state::WorkspaceState;
use eframe::egui;

pub fn render_git_bar(ui: &mut egui::Ui, ws: &mut WorkspaceState, i18n: &crate::i18n::I18n) {
    ui.horizontal(|ui| {
        ui.strong(i18n.get("panel-git"));
        ui.separator();

        let git_combo = egui::ComboBox::from_id_salt("git_select_bottom")
            .selected_text(i18n.get("btn-git-profile"))
            .width(130.0);

        git_combo.show_ui(ui, |ui| {
            let mut git_cmd = None;

            if ui.button(i18n.get("git-status")).clicked() {
                git_cmd = Some("git status");
            }
            if ui.button(i18n.get("git-diff")).clicked() {
                git_cmd = Some("git diff");
            }
            ui.separator();

            ui.add_enabled_ui(true, |ui| {
                if ui.button(i18n.get("git-add-all")).clicked() {
                    git_cmd = Some("git add .");
                }
                if ui.button(i18n.get("git-commit")).clicked() {
                    git_cmd = Some("git commit -m '");
                }
                if ui.button(i18n.get("git-push")).clicked() {
                    git_cmd = Some("git push");
                }
                if ui.button(i18n.get("git-pull")).clicked() {
                    git_cmd = Some("git pull");
                }
                ui.separator();
                if ui.button(i18n.get("git-reset-hard")).clicked() {
                    git_cmd = Some("git reset --hard HEAD");
                }
            });

            if let Some(cmd) = git_cmd {
                ws.next_terminal_id += 1;
                let terminal = crate::app::ui::terminal::instance::Terminal::new(
                    ws.next_terminal_id,
                    ui.ctx(),
                    &ws.root_path,
                    Some(cmd),
                );
                ws.build_terminal = Some(terminal);
                ws.show_build_terminal = true;
                ws.focused_panel = crate::app::types::FocusedPanel::Build;
                ui.close_menu();
            }
        });
    });
}
