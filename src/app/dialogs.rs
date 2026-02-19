use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use eframe::egui;

use super::types::{AppShared, ProjectType, default_wizard_path, path_env};

// ---------------------------------------------------------------------------
// WizardState — stav průvodce novým projektem
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
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let dialog = rfd::FileDialog::new();
        let picked = if start_dir.trim().is_empty() {
            dialog.pick_folder()
        } else {
            dialog.set_directory(start_dir).pick_folder()
        };
        let _ = tx.send(picked);
    });
    rx
}

// ---------------------------------------------------------------------------
// Validace názvu projektu
// ---------------------------------------------------------------------------

/// Povolené znaky: písmena, číslice, podtržítko, pomlčka.
/// Název nesmí být prázdný ani začínat pomlčkou.
pub(crate) fn is_valid_project_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if name.starts_with('-') {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

// ---------------------------------------------------------------------------
// show_project_wizard — sjednocený wizard nového projektu
// ---------------------------------------------------------------------------

/// Zobrazí modální dialog průvodce novým projektem.
/// `show` se nastaví na `false` při zavření.
/// `on_success` je voláno s výslednou cestou po úspěšném vytvoření projektu.
pub(crate) fn show_project_wizard(
    ctx: &egui::Context,
    state: &mut WizardState,
    show: &mut bool,
    modal_id: &str,
    shared: &Arc<Mutex<AppShared>>,
    on_success: impl FnOnce(PathBuf, &Arc<Mutex<AppShared>>),
) {
    if !*show {
        return;
    }

    let mut success_path: Option<PathBuf> = None;
    if let Some(rx) = state.browse_rx.as_ref() {
        if let Ok(picked) = rx.try_recv() {
            state.browse_rx = None;
            if let Some(dir) = picked {
                state.path = dir.to_string_lossy().to_string();
            }
        }
    }
    if let Some(rx) = state.create_rx.as_ref() {
        if let Ok(result) = rx.try_recv() {
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
    }

    let modal = egui::Modal::new(egui::Id::new(modal_id));
    let mut close_dialog = false;
    let mut create_project: Option<(ProjectType, String, String)> = None;
    let mut request_browse = false;

    modal.show(ctx, |ui| {
        ui.heading("Nový projekt");
        ui.add_space(12.0);

        ui.label("Typ projektu:");
        ui.horizontal(|ui| {
            ui.radio_value(&mut state.project_type, ProjectType::Rust, "Rust");
            ui.radio_value(&mut state.project_type, ProjectType::Symfony, "Symfony");
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Název:");
            ui.add(egui::TextEdit::singleline(&mut state.name).desired_width(250.0));
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Pracovní adresář:");
            ui.add(egui::TextEdit::singleline(&mut state.path).desired_width(200.0));
            if ui.button("Procházet…").clicked() {
                request_browse = true;
            }
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
                    "Název smí obsahovat pouze písmena, číslice, _ a - (nesmí začínat pomlčkou).",
                );
            } else {
                let preview = PathBuf::from(state.path.trim())
                    .join(state.project_type.subdir())
                    .join(&display_name);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Vytvoří se v:");
                    ui.monospace(preview.to_string_lossy().to_string());
                });
            }
        }

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            let can_create = name_valid && !state.path.trim().is_empty() && !state.creating;
            if ui
                .add_enabled(can_create, egui::Button::new("Vytvořit"))
                .clicked()
            {
                create_project = Some((
                    state.project_type,
                    state.name.trim().to_string(),
                    state.path.trim().to_string(),
                ));
            }
            if ui.button("Zrušit").clicked() {
                close_dialog = true;
            }
        });
        if state.creating {
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Vytváření projektu…").weak());
        }

        if !state.error.is_empty() {
            ui.add_space(4.0);
            ui.colored_label(egui::Color32::RED, &state.error);
        }
    });

    if close_dialog {
        state.browse_rx = None;
        state.create_rx = None;
        state.creating = false;
        *show = false;
        return;
    }

    if request_browse && state.browse_rx.is_none() {
        state.browse_rx = Some(spawn_folder_picker(state.path.clone()));
    }

    if let Some((project_type, raw_name, base_path)) = create_project {
        state.error.clear();
        state.creating = true;
        let (tx, rx) = mpsc::channel();
        state.create_rx = Some(rx);
        std::thread::spawn(move || {
            let name = if project_type == ProjectType::Rust {
                raw_name.to_lowercase()
            } else {
                raw_name
            };
            let type_dir = PathBuf::from(&base_path).join(project_type.subdir());
            if let Err(e) = std::fs::create_dir_all(&type_dir) {
                let _ = tx.send(ProjectCreateResult::Error(format!(
                    "Nelze vytvořit adresář projektu: {e}"
                )));
                return;
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

            let result = match output {
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
                        format!("Příkaz selhal s kódem: {}", output.status)
                    };
                    ProjectCreateResult::Error(msg)
                }
                Err(e) => ProjectCreateResult::Error(format!("Nepodařilo se spustit příkaz: {e}")),
            };
            let _ = tx.send(result);
        });
    }

    if let Some(path) = success_path {
        on_success(path, shared);
    }
}

// ---------------------------------------------------------------------------
// Sdílené UI helpery
// ---------------------------------------------------------------------------

/// Vykreslí seznam nedávných projektů jako klikatelné položky.
/// Vrátí cestu na projekt, na který uživatel klikl.
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

/// Zobrazí dialog pro potvrzení ukončení aplikace.
/// Vrátí `true` pokud uživatel potvrdil ukončení.
pub(crate) fn show_quit_confirm_dialog(ctx: &egui::Context) -> QuitDialogResult {
    let modal = egui::Modal::new(egui::Id::new("quit_confirm_modal"));
    let mut confirmed = false;
    let mut cancelled = false;

    modal.show(ctx, |ui| {
        ui.heading("Ukončit aplikaci");
        ui.add_space(8.0);
        ui.label("Opravdu chcete ukončit Rust Editor?");
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.button("Ukončit").clicked() {
                confirmed = true;
            }
            if ui.button("Zrušit").clicked() {
                cancelled = true;
            }
        });
    });

    if confirmed {
        QuitDialogResult::Confirmed
    } else if cancelled {
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

/// Výsledek akce ze startup dialogu
pub(crate) enum StartupAction {
    None,
    OpenPath(PathBuf),
    OpenWizard,
}

pub(crate) fn show_startup_dialog(
    ctx: &egui::Context,
    path_buffer: &mut String,
    show_wizard: bool,
    recent_projects: &[PathBuf],
    browse_rx: &mut Option<mpsc::Receiver<Option<PathBuf>>>,
) -> StartupAction {
    let mut should_open = false;
    let mut request_browse = false;
    let mut open_recent: Option<PathBuf> = None;

    egui::CentralPanel::default().show(ctx, |_ui| {});

    let mut open_wizard = false;

    let modal = egui::Modal::new(egui::Id::new("startup_modal"));
    modal.show(ctx, |ui| {
        let dlg_size = 15.0;
        ui.heading("Otevřít projekt");
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Cesta:").size(dlg_size));
            let response = ui.add(
                egui::TextEdit::singleline(path_buffer)
                    .font(egui::TextStyle::Body)
                    .desired_width(350.0),
            );
            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                should_open = true;
            }
            if !response.has_focus() && !show_wizard {
                response.request_focus();
            }
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .button(egui::RichText::new("Otevřít").size(dlg_size))
                .clicked()
            {
                should_open = true;
            }
            if ui
                .button(egui::RichText::new("Procházet…").size(dlg_size))
                .clicked()
            {
                request_browse = true;
            }
        });
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);
        if ui
            .button(egui::RichText::new("Založit nový projekt…").size(dlg_size))
            .clicked()
        {
            open_wizard = true;
        }

        if !recent_projects.is_empty() {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Nedávné projekty:").size(dlg_size));
            ui.add_space(4.0);
            open_recent = render_recent_project_list(ui, recent_projects, dlg_size);
        }
    });

    if open_wizard {
        return StartupAction::OpenWizard;
    }

    if request_browse && browse_rx.is_none() {
        *browse_rx = Some(spawn_folder_picker(path_buffer.clone()));
    }
    if let Some(rx) = browse_rx.as_ref() {
        if let Ok(picked) = rx.try_recv() {
            *browse_rx = None;
            if let Some(dir) = picked {
                *path_buffer = dir.to_string_lossy().to_string();
                should_open = true;
            }
        }
    }

    if let Some(path) = open_recent {
        if path.is_dir() {
            return StartupAction::OpenPath(path);
        }
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
