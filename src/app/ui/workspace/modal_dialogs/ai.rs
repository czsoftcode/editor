use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;

pub fn show(ctx: &egui::Context, ws: &mut WorkspaceState, i18n: &I18n) {
    // Promotion success dialog
    if let Some(path) = ws.promotion_success.clone() {
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());

        egui::Window::new(i18n.get("ai-promotion-success-title"))
            .id(egui::Id::new("ai_promotion_success_win"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .pivot(egui::Align2::CENTER_CENTER)
            .fixed_size([300.0, 200.0])
            .show(ctx, |ui: &mut egui::Ui| {
                ui.vertical_centered(|ui: &mut egui::Ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("\u{2714}")
                            .size(32.0)
                            .color(egui::Color32::from_rgb(100, 200, 100)),
                    );
                    ui.add_space(12.0);
                    ui.label(i18n.get("ai-promotion-success-body"));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(filename).strong());
                    ui.add_space(20.0);
                    if ui.button(format!("  {}  ", i18n.get("btn-ok"))).clicked() {
                        ws.promotion_success = None;
                    }
                    ui.add_space(8.0);
                });
            });
    }

    // List of all staged files in Sandbox.
    if ws.show_sandbox_staged {
        let staged_files = ws.sandbox.get_staged_files();
        if staged_files.is_empty() {
            ws.show_sandbox_staged = false;
        } else {
            let mut close_requested = false;
            egui::Window::new(i18n.get("ai-staged-files"))
                .id(egui::Id::new("ai_sandbox_staged_list_win"))
                .collapsible(false)
                .resizable(true)
                .default_size([400.0, 300.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui: &mut egui::Ui| {
                    ui.label(i18n.get("ai-staged-modal-hint"));
                    ui.add_space(8.0);

                    egui::ScrollArea::vertical()
                        .id_salt("staged_files_scroll")
                        .max_height(400.0)
                        .show(ui, |ui| {
                            for rel_path in staged_files {
                                let path_str = rel_path.to_string_lossy().to_string();
                                let project_exists = ws.root_path.join(&rel_path).exists();
                                let sandbox_exists = ws.sandbox.root.join(&rel_path).exists();

                                ui.horizontal(|ui| {
                                    if !sandbox_exists {
                                        ui.label(
                                            egui::RichText::new(i18n.get("ai-staged-del"))
                                                .color(egui::Color32::from_rgb(210, 80, 80))
                                                .small(),
                                        );
                                    } else if !project_exists {
                                        ui.label(
                                            egui::RichText::new(i18n.get("ai-staged-new"))
                                                .color(egui::Color32::from_rgb(100, 200, 100))
                                                .small(),
                                        );
                                    } else {
                                        ui.label(
                                            egui::RichText::new(i18n.get("ai-staged-mod"))
                                                .color(egui::Color32::from_rgb(200, 150, 50))
                                                .small(),
                                        );
                                    }

                                    if ui.link(&path_str).clicked() {
                                        let sandbox_path = ws.sandbox.root.join(&rel_path);
                                        let project_path = ws.root_path.join(&rel_path);

                                        let new_content = std::fs::read_to_string(sandbox_path)
                                            .unwrap_or_default();
                                        let old_content = std::fs::read_to_string(project_path)
                                            .unwrap_or_default();

                                        ws.editor.pending_ai_diff = Some((
                                            ws.root_path
                                                .join(rel_path)
                                                .to_string_lossy()
                                                .to_string(),
                                            old_content,
                                            new_content,
                                        ));
                                    }
                                });
                            }
                        });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(i18n.get("btn-close")).clicked() {
                            close_requested = true;
                        }
                    });
                });

            if close_requested {
                ws.show_sandbox_staged = false;
            }
        }
    }
}
