use super::wizard::spawn_folder_picker;
use crate::app::ui::widgets::modal::StandardModal;
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;

pub(crate) enum StartupAction {
    None,
    OpenPath(PathBuf),
    OpenWizard,
    QuitApp,
}

pub(crate) fn render_recent_project_list(
    ui: &mut egui::Ui,
    recent_projects: &[PathBuf],
    item_size: f32,
) -> Option<PathBuf> {
    let mut selected = None;
    for path in recent_projects {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let resp = ui
            .add(
                egui::Label::new(egui::RichText::new(&name).size(item_size))
                    .sense(egui::Sense::click()),
            )
            .on_hover_text(path.to_string_lossy());
        if resp.clicked() {
            selected = Some(path.clone());
        }
    }
    selected
}

pub(crate) fn show_startup_dialog(
    ctx: &egui::Context,
    path_buffer: &mut String,
    show_wizard: bool,
    recent_projects: &[PathBuf],
    browse_rx: &mut Option<mpsc::Receiver<Option<PathBuf>>>,
    missing_session: &[PathBuf],
    i18n: &crate::i18n::I18n,
) -> StartupAction {
    let mut should_open = false;
    let mut should_quit = false;
    let mut request_browse = false;
    let mut open_recent: Option<PathBuf> = None;
    let mut open_wizard = false;
    let mut show_flag = true;

    let modal =
        StandardModal::new(i18n.get("open-project-title"), "startup_modal").with_size(600.0, 500.0);

    modal.show(ctx, &mut show_flag, |ui| {
        modal.ui_footer_actions(ui, i18n, |f| {
            if f.button("startup-quit").clicked() {
                should_quit = true;
            }
            if f.button("startup-new-project").clicked() {
                open_wizard = true;
            }
            if f.button("btn-open").clicked() {
                should_open = true;
            }
            None::<()>
        });

        modal.ui_body(ui, |ui| {
            let dlg_size = 14.0;
            ui.add_space(8.0);

            egui::Grid::new("startup_path_grid")
                .num_columns(2)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(i18n.get("startup-path-label")).size(dlg_size));
                    ui.horizontal(|ui| {
                        let response = ui.add(
                            egui::TextEdit::singleline(path_buffer)
                                .font(egui::FontId::proportional(dlg_size))
                                .desired_width(ui.available_width() - 40.0),
                        );
                        if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            should_open = true;
                        }
                        if !response.has_focus() && !show_wizard {
                            response.request_focus();
                        }
                        if ui.button("…").clicked() {
                            request_browse = true;
                        }
                    });
                    ui.end_row();
                });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);

            if !missing_session.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(220, 170, 60),
                    egui::RichText::new(i18n.get("startup-missing-session-label")).strong(),
                );
                for path in missing_session {
                    ui.label(egui::RichText::new(format!("  • {}", path.to_string_lossy())).weak());
                }
                ui.add_space(8.0);
            }

            if !recent_projects.is_empty() {
                ui.label(egui::RichText::new(i18n.get("startup-recent-projects")).strong());
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_salt("startup_recent_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        open_recent = render_recent_project_list(ui, recent_projects, dlg_size);
                    });
                ui.add_space(8.0);
            }
        });
    });

    if open_wizard {
        return StartupAction::OpenWizard;
    }
    if should_quit || !show_flag {
        return StartupAction::QuitApp;
    }

    if request_browse && browse_rx.is_none() {
        *browse_rx = Some(spawn_folder_picker(path_buffer.clone()));
    }
    if let Some(rx) = browse_rx.as_ref()
        && let Ok(picked) = rx.try_recv()
    {
        *browse_rx = None;
        if let Some(dir) = picked {
            *path_buffer = dir.to_string_lossy().to_string();
            should_open = true;
        }
    }

    if let Some(path) = open_recent
        && path.is_dir()
    {
        return StartupAction::OpenPath(path);
    }

    if should_open {
        let path = PathBuf::from(path_buffer.trim());
        if path.is_dir() {
            let path = path.canonicalize().unwrap_or(path);
            return StartupAction::OpenPath(path);
        }
    }

    StartupAction::None
}
