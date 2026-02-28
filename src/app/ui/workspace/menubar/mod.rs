use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::app::ui::background::spawn_task;
use eframe::egui;

use super::super::super::build_runner::run_build_check;
use super::super::super::types::{AppAction, AppShared, Toast};
use super::state::{FilePicker, WorkspaceState};

mod build;
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
    pub install_appimage: bool,
    pub install_appimagetool: bool,
    pub install_nsis: bool,
    pub install_rpm: bool,
    pub install_generate_rpm: bool,
    pub install_deb_tools: bool,
    pub install_aur: bool,
    pub install_flatpak: bool,
    pub install_snap: bool,
    pub install_tar: bool,
    pub install_xwin: bool,
    pub install_clang: bool,
    pub install_lld: bool,
    pub install_windows_target: bool,
    pub plugins_target: Option<String>,
    pub run_agent: Option<String>,
    pub run_plugin: Option<(String, String)>,
    pub build: bool,
    pub run: bool,
    pub build_deb: bool,
    pub build_rpm: bool,
    pub build_aur: bool,
    pub build_flatpak: bool,
    pub build_snap: bool,
    pub build_appimage: bool,
    pub build_tar_gz: bool,
    pub build_exe: bool,
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
            build::render(ui, &mut actions, ws, i18n);
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
            shared.lock().expect("lock").settings.project_read_only,
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
                let context = crate::app::ui::terminal::right::format_context_for_terminal(
                    &crate::app::ai::AiManager::generate_context(ws, shared),
                );
                if let Some(terminal) = ws.claude_tabs.get_mut(active) {
                    terminal.send_command(&cmd);
                    if agent.context_aware {
                        terminal.send_command(&context);
                    }
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
    if actions.install_appimagetool {
        ws.dep_wizard.open_for_appimagetool();
    }
    if actions.install_appimage {
        ws.dep_wizard.open_for_appimage();
    }
    if actions.install_nsis {
        ws.dep_wizard.open_for_nsis();
    }
    if actions.install_rpm {
        ws.dep_wizard.open_for_rpm();
    }
    if actions.install_generate_rpm {
        ws.dep_wizard.open_for_generate_rpm();
    }
    if actions.install_deb_tools {
        ws.dep_wizard.open_for_deb_tools();
    }
    if actions.install_aur {
        ws.dep_wizard.open_for_aur();
    }
    if actions.install_flatpak {
        ws.dep_wizard.open_for_flatpak();
    }
    if actions.install_snap {
        ws.dep_wizard.open_for_snap();
    }
    if actions.install_tar {
        ws.dep_wizard.open_for_tar();
    }
    if actions.install_xwin {
        ws.dep_wizard.open_for_xwin();
    }
    if actions.install_clang {
        ws.dep_wizard.open_for_clang();
    }
    if actions.install_lld {
        ws.dep_wizard.open_for_lld();
    }
    if actions.install_windows_target {
        ws.dep_wizard.open_for_windows_target();
    }
    if actions.build_deb
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command("mkdir -p target/dist && ./packaging/deb/build-deb.sh && mv target/debian/*.deb target/dist/ 2>/dev/null || true");
    }
    if actions.build_rpm
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command("mkdir -p target/dist && cargo generate-rpm && mv *.rpm target/dist/ 2>/dev/null || true");
    }
    if actions.build_aur
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command("mkdir -p target/dist && cargo aur && mv target/cargo-aur/*.pkg.tar.zst target/dist/ 2>/dev/null || true");
    }
    if actions.build_flatpak
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command("mkdir -p target/dist && flatpak-builder --force-clean build-dir org.polycredo.Editor.yaml && flatpak-builder --repo=repo --force-clean build-dir org.polycredo.Editor.yaml && flatpak build-bundle repo target/dist/polycredo-editor.flatpak org.polycredo.Editor");
    }
    if actions.build_snap
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command(
            "mkdir -p target/dist && snapcraft && mv *.snap target/dist/ 2>/dev/null || true",
        );
    }
    if actions.build_appimage
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command("mkdir -p target/dist && cargo appimage && mv *.AppImage target/dist/ 2>/dev/null || true");
    }
    if actions.build_tar_gz
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command("mkdir -p target/dist && cargo build --release && tar -C target/release -czvf target/dist/polycredo-editor.tar.gz polycredo-editor");
    }
    if actions.build_exe
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command("mkdir -p target/dist && export PATH=$PATH:/usr/lib/llvm-19/bin && cargo xwin build --release --target x86_64-pc-windows-msvc && cp target/x86_64-pc-windows-msvc/release/polycredo-editor.exe target/dist/ 2>/dev/null || true");
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
