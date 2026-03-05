use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::app::ui::background::spawn_task;
use eframe::egui;

use super::super::super::build_runner::run_build_check;
use super::super::super::types::{AppAction, AppShared, Toast};
use super::state::{FilePicker, WorkspaceState};

mod edit;
mod file;
mod help;
mod project;
mod view;

// ---------------------------------------------------------------------------
// Helper data types
// ---------------------------------------------------------------------------

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
    pub support: bool,
    pub settings: bool,
    pub plugins: bool,
    pub plugins_target: Option<String>,
    pub run_agent: Option<String>,
    pub run_plugin: Option<(String, String)>,
    pub build: bool,
    pub run: bool,
    pub open_file_picker: bool,
    pub project_search: bool,
}

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
            file::render(ui, &mut actions, shared, i18n);
            project::render(ui, &mut actions, &recent_snapshot, i18n);
            edit::render(ui, &mut actions, i18n);
            view::render(ui, &mut actions, ws, i18n);
            help::render(ui, &mut actions, i18n);
        });
    });

    actions
}

pub(super) fn process_menu_actions(
    _ctx: &egui::Context,
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
            false,
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
    if actions.support {
        ws.show_support = true;
    }
    if actions.settings {
        ws.show_settings = true;
    }
    if let Some(agent_id) = actions.run_agent {
        if agent_id == "ai_chat" {
            ws.show_ai_chat = true;
        } else {
            let agents = {
                let sh = shared.lock().expect("lock");
                sh.registry.agents.get_all().to_vec()
            };
            if let Some(agent) = agents.iter().find(|a| a.id == agent_id) {
                let cmd = agent.command.clone();
                let active = ws.claude_active_tab;
                if let Some(terminal) = ws.claude_tabs.get_mut(active) {
                    terminal.send_command(&cmd);
                }
            }
        }
    }
    if let Some((plugin_id, func)) = actions.run_plugin {
        if func == "OPEN_AI_CHAT" {
            ws.ai_selected_provider = plugin_id;
            ws.show_ai_chat = true;
            ws.ai_focus_requested = true;
            crate::app::ui::terminal::ai_chat::handle_action(
                crate::app::ui::terminal::ai_chat::AiChatAction::NewQuery,
                ws,
                shared,
            );
        } else {
            let (plugin_manager, config): (Arc<crate::app::registry::plugins::PluginManager>, _) = {
                let sh = shared.lock().expect("lock");
                let cfg = sh
                    .settings
                    .plugins
                    .get(&plugin_id)
                    .map(|s| s.config.clone())
                    .unwrap_or_default();
                (Arc::clone(&sh.registry.plugins), cfg)
            };
            match plugin_manager.call(&plugin_id, &func, "menu", &config) {
                Ok(res) => {
                    ws.toasts.push(crate::app::types::Toast::info(res));
                }
                Err(e) => {
                    ws.toasts.push(crate::app::types::Toast::error(format!(
                        "Plugin failed: {}",
                        e
                    )));
                }
            }
        }
    }
    if actions.plugins {
        ws.show_plugins = true;
        let shared_lock = shared.lock().expect("lock");
        ws.plugins_draft = Some((*shared_lock.settings).clone());
    }
    if actions.new_project {
        ws.show_new_project = true;
    }
    if actions.build {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo build 2>&1");
        }
        let build_path = ws.root_path.clone();
        ws.build_error_rx = Some(run_build_check(build_path));
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
