use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::app::ui::background::spawn_task;

use eframe::egui;

use super::super::super::build_runner::run_build_check;
use super::super::super::types::{AppAction, AppShared, Toast};
use super::state::{FilePicker, WorkspaceState};

// ---------------------------------------------------------------------------
// Helper data types for render_workspace
// ---------------------------------------------------------------------------

/// Actions originating from the menu bar — processed after menu rendering.
#[derive(Default)]
pub(crate) struct MenuActions {
    pub open_folder: bool,
    pub save: bool,
    pub close_file: bool,
    pub quit: bool,
    pub new_project: bool,
    pub open_project: bool,
    pub open_recent: Option<PathBuf>,
    pub toggle_left: bool,
    pub toggle_right: bool,
    pub toggle_build: bool,
    pub toggle_float: bool,
    pub about: bool,
    pub settings: bool,
    pub build: bool,
    pub run: bool,
    pub open_file_picker: bool,
    pub project_search: bool,
}

// ---------------------------------------------------------------------------
// render_menu_bar
// ---------------------------------------------------------------------------

/// Renders the menu bar and returns the recorded actions.
pub(super) fn render_menu_bar(
    ctx: &egui::Context,
    ws: &WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) -> MenuActions {
    let mut actions = MenuActions::default();
    let recent_snapshot = shared
        .lock()
        .expect("Failed to lock AppShared for recent projects snapshot")
        .recent_projects
        .clone();

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button(i18n.get("menu-file"), |ui| {
                if ui.button(i18n.get("menu-file-open-folder")).clicked() {
                    actions.open_folder = true;
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new(i18n.get("menu-file-save")).shortcut_text("Ctrl+S"))
                    .clicked()
                {
                    actions.save = true;
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new(i18n.get("menu-file-close-tab")).shortcut_text("Ctrl+W"))
                    .clicked()
                {
                    actions.close_file = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(i18n.get("menu-file-quit")).clicked() {
                    actions.quit = true;
                    ui.close_menu();
                }
            });

            ui.menu_button(i18n.get("menu-project"), |ui| {
                if ui.button(i18n.get("menu-project-open")).clicked() {
                    actions.open_project = true;
                    ui.close_menu();
                }
                if ui.button(i18n.get("menu-project-new")).clicked() {
                    actions.new_project = true;
                    ui.close_menu();
                }
                if !recent_snapshot.is_empty() {
                    ui.separator();
                    ui.menu_button(i18n.get("menu-project-recent"), |ui| {
                        for path in &recent_snapshot {
                            let name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| path.to_string_lossy().into_owned());
                            if ui
                                .button(&name)
                                .on_hover_text(path.to_string_lossy())
                                .clicked()
                            {
                                actions.open_recent = Some(path.clone());
                                ui.close_menu();
                            }
                        }
                    });
                }
            });

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

            ui.menu_button(i18n.get("menu-help"), |ui| {
                if ui.button(i18n.get("menu-help-settings")).clicked() {
                    actions.settings = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(i18n.get("menu-help-about")).clicked() {
                    actions.about = true;
                    ui.close_menu();
                }
            });
        });
    });

    actions
}

// ---------------------------------------------------------------------------
// process_menu_actions
// ---------------------------------------------------------------------------

/// Applies menu actions to the workspace state. Returns the path for reinitialization (if a folder was selected).
pub(super) fn process_menu_actions(
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    actions: MenuActions,
    i18n: &crate::i18n::I18n,
) -> Option<PathBuf> {
    if actions.quit {
        shared
            .lock()
            .expect("Failed to lock AppShared for quit action")
            .actions
            .push(AppAction::QuitAll);
    }
    if actions.save
        && let Some(err) = ws.editor.save(
            i18n,
            &shared
                .lock()
                .expect("Failed to lock AppShared for save action")
                .is_internal_save,
        )
    {
        ws.toasts.push(Toast::error(err));
    }
    if actions.close_file {
        ws.editor.clear();
    }
    if actions.toggle_left {
        ws.show_left_panel = !ws.show_left_panel;
    }
    if actions.toggle_right {
        ws.show_right_panel = !ws.show_right_panel;
    }
    if actions.toggle_float {
        ws.claude_float = !ws.claude_float;
    }
    if actions.toggle_build {
        ws.show_build_terminal = !ws.show_build_terminal;
    }
    if actions.about {
        ws.show_about = true;
    }
    if actions.settings {
        ws.show_settings = true;
    }
    if actions.new_project {
        ws.show_new_project = true;
    }
    if actions.build {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo build 2>&1");
        }
        ws.build_error_rx = Some(run_build_check(ws.root_path.clone()));
        ws.build_errors.clear();
    }
    if actions.run
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command("cargo run 2>&1");
    }
    if actions.open_file_picker && ws.file_picker.is_none() {
        let files = ws.project_index.get_files();
        ws.file_picker = Some(FilePicker::new(files));
    }
    if actions.project_search {
        ws.project_search.show_input = true;
        ws.project_search.focus_requested = true;
    }

    if let Some(path) = actions.open_recent
        && path.is_dir()
    {
        let mut sh = shared
            .lock()
            .expect("Failed to lock AppShared for open recent action");
        sh.actions.push(AppAction::AddRecent(path.clone()));
        sh.actions.push(AppAction::OpenInNewWindow(path));
    }

    // Result of previous async file dialog
    let mut open_here_path: Option<PathBuf> = None;
    if let Some(rx) = &ws.folder_pick_rx
        && let Ok((maybe_path, in_new_window)) = rx.try_recv()
    {
        ws.folder_pick_rx = None;
        if let Some(dir) = maybe_path {
            let path = dir.canonicalize().unwrap_or(dir);
            if in_new_window {
                let mut sh = shared
                    .lock()
                    .expect("Failed to lock AppShared for folder pick result");
                sh.actions.push(AppAction::AddRecent(path.clone()));
                sh.actions.push(AppAction::OpenInNewWindow(path));
            } else {
                open_here_path = Some(path);
            }
        }
    }

    // Launch async file dialog (does not block UI thread)
    if actions.open_project && ws.folder_pick_rx.is_none() {
        let projects_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join("MyProject");
        if let Err(e) = std::fs::create_dir_all(&projects_dir) {
            let mut args = fluent_bundle::FluentArgs::new();
            args.set("reason", e.to_string());
            ws.toasts.push(Toast::error(
                i18n.get_args("error-projects-dir-prepare", &args),
            ));
        }
        ws.folder_pick_rx = Some(spawn_task(move || {
            (
                rfd::FileDialog::new()
                    .set_directory(&projects_dir)
                    .pick_folder(),
                true,
            )
        }));
    }
    if actions.open_folder && ws.folder_pick_rx.is_none() {
        let start_dir = ws.root_path.clone();
        ws.folder_pick_rx = Some(spawn_task(move || {
            (
                rfd::FileDialog::new()
                    .set_directory(&start_dir)
                    .pick_folder(),
                false,
            )
        }));
    }

    open_here_path
}
