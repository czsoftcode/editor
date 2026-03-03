use crate::app::types::{AppShared, ProjectType, default_wizard_path};
use crate::app::ui::widgets::modal::StandardModal;
use crate::app::validation::is_valid_project_name;
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub(crate) struct WizardState {
    pub name: String,
    pub path: String,
    pub project_type: ProjectType,
    pub error: String,
    pub creating: bool,
    pub(crate) browse_rx: Option<mpsc::Receiver<Option<PathBuf>>>,
    pub(crate) create_rx: Option<mpsc::Receiver<ProjectCreateResult>>,
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

pub(crate) enum ProjectCreateResult {
    Success(PathBuf),
    Error(String),
}

pub(crate) fn spawn_folder_picker(start_dir: String) -> mpsc::Receiver<Option<PathBuf>> {
    crate::app::ui::background::spawn_task(move || {
        let dialog = rfd::FileDialog::new();
        if start_dir.trim().is_empty() {
            dialog.pick_folder()
        } else {
            dialog.set_directory(start_dir).pick_folder()
        }
    })
}

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
            if ui.button(i18n.get("btn-close")).clicked() {
                return Some(true);
            }
            if ui.button(i18n.get("btn-cancel")).clicked() {
                return Some(true);
            }
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
            None
        }) && close
        {
            return true;
        }

        // BODY
        modal.ui_body(ui, |ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(i18n.get("wizard-project-type")).strong());
            egui::ComboBox::from_id_salt("project_type_combo")
                .selected_text(match state.project_type {
                    ProjectType::Rust => i18n.get("wizard-type-rust-2024"),
                    ProjectType::Symfony74 => i18n.get("wizard-type-symfony-7-4"),
                    ProjectType::Symfony80 => i18n.get("wizard-type-symfony-8-0"),
                    ProjectType::Laravel12 => i18n.get("wizard-type-laravel-12"),
                    ProjectType::Nette32 => i18n.get("wizard-type-nette-3-2"),
                    ProjectType::Nette30 => i18n.get("wizard-type-nette-3-0"),
                    ProjectType::FastApi => i18n.get("wizard-type-fastapi-0-135"),
                    ProjectType::NextJs => i18n.get("wizard-type-nextjs-16-1"),
                    ProjectType::ExpressJs => i18n.get("wizard-type-expressjs-5-0"),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut state.project_type,
                        ProjectType::Rust,
                        i18n.get("wizard-type-rust-2024"),
                    );
                    ui.selectable_value(
                        &mut state.project_type,
                        ProjectType::Symfony74,
                        i18n.get("wizard-type-symfony-7-4"),
                    );
                    ui.selectable_value(
                        &mut state.project_type,
                        ProjectType::Symfony80,
                        i18n.get("wizard-type-symfony-8-0"),
                    );
                    ui.selectable_value(
                        &mut state.project_type,
                        ProjectType::Laravel12,
                        i18n.get("wizard-type-laravel-12"),
                    );
                    ui.selectable_value(
                        &mut state.project_type,
                        ProjectType::Nette32,
                        i18n.get("wizard-type-nette-3-2"),
                    );
                    ui.selectable_value(
                        &mut state.project_type,
                        ProjectType::Nette30,
                        i18n.get("wizard-type-nette-3-0"),
                    );
                    ui.separator();
                    ui.selectable_value(
                        &mut state.project_type,
                        ProjectType::FastApi,
                        i18n.get("wizard-type-fastapi-0-135"),
                    );
                    ui.selectable_value(
                        &mut state.project_type,
                        ProjectType::NextJs,
                        i18n.get("wizard-type-nextjs-16-1"),
                    );
                    ui.selectable_value(
                        &mut state.project_type,
                        ProjectType::ExpressJs,
                        i18n.get("wizard-type-expressjs-5-0"),
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
            let name_valid = is_valid_project_name(raw_name);
            if !raw_name.is_empty() {
                if !name_valid {
                    ui.add_space(4.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(0xe0, 0x70, 0x10),
                        i18n.get("wizard-name-hint"),
                    );
                } else {
                    let preview = PathBuf::from(state.path.trim())
                        .join(state.project_type.subdir())
                        .join(raw_name);
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

    if let Some((project_type, name, base_path)) = create_project {
        state.error.clear();
        state.creating = true;
        let i18n_arc = Arc::clone(&shared.lock().expect("lock").i18n);
        state.create_rx = Some(crate::app::ui::background::spawn_task(move || {
            let i18n = &*i18n_arc;
            let type_dir = PathBuf::from(&base_path).join(project_type.subdir());
            if let Err(e) = std::fs::create_dir_all(&type_dir) {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("reason", e.to_string());
                return ProjectCreateResult::Error(
                    i18n.get_args("error-project-dir-create", &args),
                );
            }
            let full_path = type_dir.join(&name);
            match crate::app::project_templates::generate_project(project_type, &name, &full_path) {
                Ok(_) => {
                    ProjectCreateResult::Success(full_path.canonicalize().unwrap_or(full_path))
                }
                Err(e) => ProjectCreateResult::Error(e),
            }
        }));
    }

    if let Some(path) = success_path {
        on_success(path, shared);
    }
}
