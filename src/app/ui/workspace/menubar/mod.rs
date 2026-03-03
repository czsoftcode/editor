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
    pub install_flatpak: bool,
    pub install_snap: bool,
    pub configure_lxd: bool,
    pub install_xwin: bool,
    pub install_clang: bool,
    pub install_lld: bool,
    pub install_windows_target: bool,
    pub install_freebsd_target: bool,
    pub install_cross: bool,
    pub install_fpm: bool,
    pub install_podman: bool,
    pub build_all: bool,
    pub plugins_target: Option<String>,
    pub run_agent: Option<String>,
    pub run_plugin: Option<(String, String)>,
    pub build: bool,
    pub run: bool,
    pub build_deb: bool,
    pub build_rpm: bool,
    pub build_flatpak: bool,
    pub build_snap: bool,
    pub build_appimage: bool,
    pub build_exe: bool,
    pub build_freebsd: bool,
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
    if actions.install_freebsd_target {
        ws.dep_wizard.open_for_freebsd_target();
    }
    if actions.install_cross {
        ws.dep_wizard.open_for_cross();
    }
    if actions.install_fpm {
        ws.dep_wizard.open_for_fpm();
    }
    if actions.install_podman {
        ws.dep_wizard.open_for_podman();
    }
    if actions.build_all {
        ws.build_all_modal.start(ws.root_path.clone(), _ctx.clone());
    }
    if actions.install_flatpak {
        ws.dep_wizard.open_for_flatpak();
    }
    if actions.install_snap {
        ws.dep_wizard.open_for_snap();
    }
    if actions.configure_lxd {
        ws.dep_wizard.open_for_lxd();
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
    let root = {
        let c: Vec<_> = ws.root_path.components().collect();
        let n = c.len();
        if n >= 2
            && c[n - 1].as_os_str() == "sandbox"
            && c[n - 2].as_os_str() == ".polycredo"
        {
            ws.root_path
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| ws.root_path.clone())
        } else {
            ws.root_path.clone()
        }
    }
    .display()
    .to_string();
    let _sandbox_root = ws.sandbox.root.display().to_string();
    let upload_to_github = "&& (VERSION=$(./scripts/get-version.sh); \
                             gh release create \"v$VERSION\" --title \"Release v$VERSION\" --notes \"Automated build from PolyCredo Editor\" 2>/dev/null || true; \
                             gh release upload \"v$VERSION\" target/dist/* --clobber 2>/dev/null || true)";

    if actions.build_deb
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command(&format!(
            "cd \"{root}\" && mkdir -p target/dist && \
             export DEB_BUILD_TYPE=deb && \
             export CARGO_TARGET_DIR=\"$HOME/.cache/polycredo-editor/target\" && \
             ./packaging/deb/build-deb.sh \
             {upload_to_github} || true"
        ));
    }
    if actions.build_rpm
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command(&format!(
            "cd \"{root}\" && mkdir -p target/dist && \
             CARGO_TARGET_DIR=\"$HOME/.cache/polycredo-editor/target\" \
             cargo generate-rpm -o target/dist/ 2>&1 || \
             (CARGO_TARGET_DIR=\"$HOME/.cache/polycredo-editor/target\" \
              cargo generate-rpm 2>&1 && mv ./*.rpm target/dist/) {upload_to_github} || true"
        ));
    }
    if actions.build_freebsd
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command(&format!(
            "cd \"{root}\" && mkdir -p target/dist && \
             rustup target add x86_64-unknown-freebsd 2>/dev/null || true && \
             cross build --release --target x86_64-unknown-freebsd && \
             VERSION=$(./scripts/get-version.sh) && \
             fpm -s dir -t freebsd -n polycredo-editor -v \"$VERSION\" \
               --prefix /usr/local \
               -p \"target/dist/polycredo-editor-$VERSION-amd64.pkg\" \
               \"$HOME/.cache/polycredo-editor/target/x86_64-unknown-freebsd/release/polycredo-editor=/bin/polycredo-editor\" \
             {upload_to_github} || true"
        ));
    }
    if actions.build_flatpak
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command(&format!(
            "cd \"{root}\" && mkdir -p target/dist \
               \"$HOME/.cache/polycredo-editor/flatpak/build\" \
               \"$HOME/.cache/polycredo-editor/flatpak/repo\" \
               \"$HOME/.cache/polycredo-editor/flatpak/state\" && \
             flatpak-builder --force-clean \
               --state-dir=\"$HOME/.cache/polycredo-editor/flatpak/state\" \
               --repo=\"$HOME/.cache/polycredo-editor/flatpak/repo\" \
               \"$HOME/.cache/polycredo-editor/flatpak/build\" \
               \"{root}/.polycredo/sandbox/org.polycredo.Editor.yaml\" && \
             VERSION=$(./scripts/get-version.sh) && \
             flatpak build-bundle \"$HOME/.cache/polycredo-editor/flatpak/repo\" \
               target/dist/polycredo-editor-$VERSION.flatpak org.polycredo.Editor {upload_to_github}"
        ));
    }
    if actions.build_snap
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command(&format!(
            "cd \"{root}\" && mkdir -p target/dist && export PATH=$PATH:/snap/bin && \
             VERSION=$(./scripts/get-version.sh) && \
             (sg lxd -c \"snapcraft pack --output target/dist/polycredo-editor-$VERSION-amd64.snap\" 2>/dev/null || \
              snapcraft pack --output target/dist/polycredo-editor-$VERSION-amd64.snap 2>/dev/null || \
              (snapcraft pack && mv *.snap target/dist/polycredo-editor-$VERSION-amd64.snap 2>/dev/null)) \
             {upload_to_github} || true",
        ));
    }
    if actions.build_appimage
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command(&format!(
            "cd \"{root}\" && mkdir -p target/dist && \
             cp packaging/icons/icon-256.png icon.png 2>/dev/null || true && \
             cargo appimage && \
             VERSION=$(./scripts/get-version.sh) && \
             APPIMAGE=$(find \"$HOME/.cache/polycredo-editor/target\" . -maxdepth 5 -name '*.AppImage' ! -path '*/target/dist/*' 2>/dev/null | head -1) && \
             [ -n \"$APPIMAGE\" ] && cp \"$APPIMAGE\" \"target/dist/polycredo-editor-$VERSION-x86_64.AppImage\" \
             {upload_to_github} || true"
        ));
    }
    if actions.build_exe
        && let Some(t) = &mut ws.build_terminal
    {
        t.send_command(&format!(
            "cd \"{root}\" && mkdir -p target/dist && \
             export PATH=$PATH:/usr/lib/llvm-19/bin && \
             cargo xwin build --release --target x86_64-pc-windows-msvc && \
             VERSION=$(./scripts/get-version.sh) && \
             (cp \"$HOME/.cache/polycredo-editor/target/x86_64-pc-windows-msvc/release/polycredo-editor.exe\" \
                \"target/dist/polycredo-editor-$VERSION-x86_64.exe\" 2>/dev/null || \
              cp \"$HOME/.cache/polycredo-editor/target/release/polycredo-editor.exe\" \
                \"target/dist/polycredo-editor-$VERSION-x86_64.exe\" 2>/dev/null) \
             {upload_to_github} || true"
        ));
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
