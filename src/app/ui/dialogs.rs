use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use eframe::egui;

use super::super::types::{AppShared, ProjectType, default_wizard_path, path_env};
use crate::app::ui::widgets::modal::StandardModal;

// ---------------------------------------------------------------------------
// PrivacyState — state of the privacy policy dialog
// ---------------------------------------------------------------------------

#[derive(Default)]
pub(crate) struct PrivacyState {
    pub cache: egui_commonmark::CommonMarkCache,
    pub content: Option<String>,
}

pub(crate) enum PrivacyResult {
    Accepted,
    LanguageChanged(String),
    None,
}

pub(crate) fn show_privacy_dialog(
    ctx: &egui::Context,
    state: &mut PrivacyState,
    i18n: &crate::i18n::I18n,
) -> PrivacyResult {
    if state.content.is_none() {
        let lang = i18n.lang();
        let mut path = PathBuf::from("privacy").join(format!("Privacy_{}.md", lang));
        if !path.exists() {
            path = PathBuf::from("privacy").join("Privacy_en.md");
        }

        match std::fs::read_to_string(&path) {
            Ok(c) => state.content = Some(c),
            Err(e) => {
                state.content = Some(format!(
                    "Error loading privacy policy from {}: {}",
                    path.display(),
                    e
                ));
            }
        }
    }

    let mut local_show = true;
    let modal =
        StandardModal::new(i18n.get("privacy-title"), "privacy_modal").with_size(700.0, 550.0);

    let res = modal.show(ctx, &mut local_show, |ui| {
        let mut result = PrivacyResult::None;

        // FOOTER
        if let Some(r) = modal.ui_footer(ui, |ui| {
            if ui
                .button(egui::RichText::new(i18n.get("btn-accept-privacy")).strong())
                .clicked()
            {
                return Some(PrivacyResult::Accepted);
            }
            if ui.button(i18n.get("startup-quit")).clicked() {
                std::process::exit(0);
            }
            None
        }) {
            result = r;
        }

        // BODY
        modal.ui_body(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("🌐");
                for &lang_code in crate::i18n::SUPPORTED_LANGS {
                    if ui
                        .selectable_label(i18n.lang() == lang_code, lang_code.to_uppercase())
                        .clicked()
                    {
                        result = PrivacyResult::LanguageChanged(lang_code.to_string());
                    }
                }
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .id_salt("privacy_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Some(content) = &state.content {
                        egui_commonmark::CommonMarkViewer::new().show(
                            ui,
                            &mut state.cache,
                            content,
                        );
                    }
                });
        });

        result
    });

    res.unwrap_or(PrivacyResult::None)
}

// ---------------------------------------------------------------------------
// WizardState — state of the new project wizard
// ---------------------------------------------------------------------------

pub(crate) struct WizardState {
    pub name: String,
    pub path: String,
    pub project_type: ProjectType,
    pub error: String,
    pub creating: bool,
    browse_rx: Option<mpsc::Receiver<Option<PathBuf>>>,
    create_rx: Option<mpsc::Receiver<ProjectCreateResult>>,
}

impl Default for WizardState {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: default_wizard_path(),
            project_type: ProjectType::Rust,
            error: String::new(),
            creating: false,
            browse_rx: None,
            create_rx: None,
        }
    }
}

enum ProjectCreateResult {
    Success(PathBuf),
    Error(String),
}

fn spawn_folder_picker(start_dir: String) -> mpsc::Receiver<Option<PathBuf>> {
    crate::app::ui::background::spawn_task(move || {
        let dialog = rfd::FileDialog::new();
        if start_dir.trim().is_empty() {
            dialog.pick_folder()
        } else {
            dialog.set_directory(start_dir).pick_folder()
        }
    })
}

use crate::app::validation::is_valid_project_name;

// ---------------------------------------------------------------------------
// show_project_wizard — unified new project wizard
// ---------------------------------------------------------------------------

pub(crate) fn show_project_wizard(
    ctx: &egui::Context,
    state: &mut WizardState,
    show: &mut bool,
    modal_id: &str,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
    on_success: impl FnOnce(PathBuf, &Arc<Mutex<AppShared>>),
) {
    if !*show {
        return;
    }

    let mut success_path: Option<PathBuf> = None;
    if let Some(rx) = state.browse_rx.as_ref()
        && let Ok(picked) = rx.try_recv()
    {
        state.browse_rx = None;
        if let Some(dir) = picked {
            state.path = dir.to_string_lossy().to_string();
        }
    }
    if let Some(rx) = state.create_rx.as_ref()
        && let Ok(result) = rx.try_recv()
    {
        state.create_rx = None;
        state.creating = false;
        match result {
            ProjectCreateResult::Success(path) => {
                state.error.clear();
                state.name.clear();
                *show = false;
                success_path = Some(path);
            }
            ProjectCreateResult::Error(err) => {
                state.error = err;
            }
        }
    }

    let mut request_browse = false;
    let mut create_project: Option<(ProjectType, String, String)> = None;

    let modal = StandardModal::new(i18n.get("wizard-title"), modal_id).with_size(550.0, 450.0);

    if let Some(true) = modal.show(ctx, show, |ui| {
        // FOOTER
        if let Some(close) = modal.ui_footer(ui, |ui| {
            let raw_name = state.name.trim();
            let name_valid = is_valid_project_name(raw_name);
            let can_create = name_valid && !state.path.trim().is_empty() && !state.creating;
            if ui
                .add_enabled(can_create, egui::Button::new(i18n.get("btn-create")))
                .clicked()
            {
                create_project = Some((
                    state.project_type,
                    state.name.trim().to_string(),
                    state.path.trim().to_string(),
                ));
            }
            if ui.button(i18n.get("btn-cancel")).clicked() {
                return Some(true);
            }
            None
        }) && close
        {
            return true;
        }

        // BODY
        modal.ui_body(ui, |ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(i18n.get("wizard-project-type")).strong());
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut state.project_type,
                    ProjectType::Rust,
                    i18n.get("wizard-type-rust"),
                );
                ui.radio_value(
                    &mut state.project_type,
                    ProjectType::Symfony,
                    i18n.get("wizard-type-symfony"),
                );
            });
            ui.add_space(12.0);

            egui::Grid::new("wizard_grid")
                .num_columns(2)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    ui.label(i18n.get("btn-name-label"));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.name)
                            .desired_width(ui.available_width() - 20.0),
                    );
                    ui.end_row();

                    ui.label(i18n.get("wizard-project-path"));
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut state.path)
                                .desired_width(ui.available_width() - 40.0),
                        );
                        if ui.button("…").clicked() {
                            request_browse = true;
                        }
                    });
                    ui.end_row();
                });

            let raw_name = state.name.trim();
            let display_name = if state.project_type == ProjectType::Rust {
                raw_name.to_lowercase()
            } else {
                raw_name.to_string()
            };
            let name_valid = is_valid_project_name(raw_name);
            if !display_name.is_empty() {
                if !name_valid {
                    ui.add_space(4.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(0xe0, 0x70, 0x10),
                        i18n.get("wizard-name-hint"),
                    );
                } else {
                    let preview = PathBuf::from(state.path.trim())
                        .join(state.project_type.subdir())
                        .join(&display_name);
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(i18n.get("settings-creates-in"));
                        ui.monospace(preview.to_string_lossy().to_string());
                    });
                }
            }

            if state.creating {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(egui::RichText::new(i18n.get("wizard-creating")).weak());
                });
            }

            if !state.error.is_empty() {
                ui.add_space(8.0);
                ui.colored_label(egui::Color32::RED, &state.error);
            }
        });
        false
    }) {
        *show = false;
    }

    if request_browse && state.browse_rx.is_none() {
        state.browse_rx = Some(spawn_folder_picker(state.path.clone()));
    }

    if let Some((project_type, raw_name, base_path)) = create_project {
        state.error.clear();
        state.creating = true;
        let i18n_arc = std::sync::Arc::clone(
            &shared
                .lock()
                .expect("Failed to lock AppShared for i18n in project wizard")
                .i18n,
        );
        state.create_rx = Some(crate::app::ui::background::spawn_task(move || {
            let i18n = &*i18n_arc;
            let name = if project_type == ProjectType::Rust {
                raw_name.to_lowercase()
            } else {
                raw_name
            };
            let type_dir = PathBuf::from(&base_path).join(project_type.subdir());
            if let Err(e) = std::fs::create_dir_all(&type_dir) {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("reason", e.to_string());
                return ProjectCreateResult::Error(
                    i18n.get_args("error-project-dir-create", &args),
                );
            }

            let full_path = type_dir.join(&name);
            let env = path_env();
            let output = match project_type {
                ProjectType::Rust => std::process::Command::new("cargo")
                    .args(["new", &name])
                    .current_dir(&type_dir)
                    .env("PATH", &env)
                    .output(),
                ProjectType::Symfony => std::process::Command::new("composer")
                    .args(["create-project", "symfony/skeleton", &name])
                    .current_dir(&type_dir)
                    .env("PATH", &env)
                    .output(),
            };

            match output {
                Ok(output) if output.status.success() => {
                    ProjectCreateResult::Success(full_path.canonicalize().unwrap_or(full_path))
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let msg = if !stderr.is_empty() {
                        stderr.to_string()
                    } else if !stdout.is_empty() {
                        stdout.to_string()
                    } else {
                        let mut args = fluent_bundle::FluentArgs::new();
                        args.set("code", output.status.to_string());
                        i18n.get_args("error-cmd-failed", &args)
                    };
                    ProjectCreateResult::Error(msg)
                }
                Err(e) => {
                    let mut args = fluent_bundle::FluentArgs::new();
                    args.set("reason", e.to_string());
                    ProjectCreateResult::Error(i18n.get_args("error-cmd-start", &args))
                }
            }
        }));
    }

    if let Some(path) = success_path {
        on_success(path, shared);
    }
}

// ---------------------------------------------------------------------------
// Shared UI helpers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// show_quit_confirm_dialog
// ---------------------------------------------------------------------------

pub(crate) fn show_quit_confirm_dialog(
    ctx: &egui::Context,
    i18n: &crate::i18n::I18n,
) -> QuitDialogResult {
    let mut show_flag = true;
    let mut confirmed = false;
    let mut cancelled = false;

    let modal =
        StandardModal::new(i18n.get("quit-title"), "quit_confirm_modal").with_size(400.0, 250.0);

    modal.show(ctx, &mut show_flag, |ui| {
        modal.ui_footer(ui, |ui| {
            if ui.button(i18n.get("quit-confirm")).clicked() {
                confirmed = true;
            }
            if ui.button(i18n.get("quit-cancel")).clicked() {
                cancelled = true;
            }
            None::<()>
        });

        modal.ui_body(ui, |ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(i18n.get("quit-message")).size(14.0));
            ui.add_space(12.0);
        });
    });

    if confirmed {
        QuitDialogResult::Confirmed
    } else if cancelled || !show_flag {
        QuitDialogResult::Cancelled
    } else {
        QuitDialogResult::Open
    }
}

pub(crate) fn show_close_project_confirm_dialog(
    ctx: &egui::Context,
    modal_id: &str,
    project_path: &str,
    i18n: &crate::i18n::I18n,
) -> QuitDialogResult {
    let mut show_flag = true;
    let mut confirmed = false;
    let mut cancelled = false;

    let modal =
        StandardModal::new(i18n.get("close-project-title"), modal_id).with_size(450.0, 280.0);

    modal.show(ctx, &mut show_flag, |ui| {
        modal.ui_footer(ui, |ui| {
            if ui.button(i18n.get("close-project-confirm")).clicked() {
                confirmed = true;
            }
            if ui.button(i18n.get("close-project-cancel")).clicked() {
                cancelled = true;
            }
            None::<()>
        });

        modal.ui_body(ui, |ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(i18n.get("close-project-message")).size(14.0));
            ui.monospace(project_path);
            ui.add_space(12.0);
        });
    });

    if confirmed {
        QuitDialogResult::Confirmed
    } else if cancelled || !show_flag {
        QuitDialogResult::Cancelled
    } else {
        QuitDialogResult::Open
    }
}

pub(crate) enum QuitDialogResult {
    Confirmed,
    Cancelled,
    Open,
}

// ---------------------------------------------------------------------------
// show_startup_dialog
// ---------------------------------------------------------------------------

pub(crate) enum StartupAction {
    None,
    OpenPath(PathBuf),
    OpenWizard,
    QuitApp,
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
        modal.ui_footer(ui, |ui| {
            if ui.button(i18n.get("btn-open")).clicked() {
                should_open = true;
            }
            if ui.button(i18n.get("startup-new-project")).clicked() {
                open_wizard = true;
            }
            if ui.button(i18n.get("startup-quit")).clicked() {
                should_quit = true;
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
