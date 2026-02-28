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
    pub support: bool,
    pub settings: bool,
    pub plugins: bool,
    pub install_appimagetool: bool,
    pub install_nsis: bool,
    pub install_rpm: bool,
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
    pub build_appimage: bool,
    pub build_tar_gz: bool,
    pub build_exe: bool,
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
                ui.menu_button(i18n.get("menu-file-plugins"), |ui| {
                    if ui
                        .add(
                            egui::Button::new(i18n.get("menu-file-plugins-manager"))
                                .shortcut_text("Ctrl+Shift+L"),
                        )
                        .clicked()
                    {
                        actions.plugins = true;
                        ui.close_menu();
                    }
                    ui.separator();

                    let plugins = {
                        let shared_lock = shared.lock().expect("lock");
                        let p_list = shared_lock.registry.plugins.plugins.lock().expect("lock");
                        p_list
                            .iter()
                            .map(|p| {
                                (
                                    p.id.clone(),
                                    p.metadata
                                        .as_ref()
                                        .map(|m| m.name.clone())
                                        .unwrap_or_else(|| p.id.clone()),
                                    p.metadata
                                        .as_ref()
                                        .and_then(|m| m.plugin_type.clone())
                                        .unwrap_or_default(),
                                )
                            })
                            .collect::<Vec<_>>()
                    };

                    ui.menu_button(i18n.get("plugins-category-ai"), |ui| {
                        // List all loaded plugins of type "ai_agent"
                        for (id, name, p_type) in &plugins {
                            if p_type == "ai_agent" && ui.button(name).clicked() {
                                // Set this as the selected provider and open the unified modal
                                actions.run_agent = Some("ai_chat".to_string());
                                actions.run_plugin = Some((id.clone(), "OPEN_AI_CHAT".to_string()));
                                ui.close_menu();
                            }
                        }
                    });

                    ui.menu_button(i18n.get("plugins-category-general"), |ui| {
                        for (id, name, p_type) in &plugins {
                            if p_type != "ai_agent" && ui.button(name).clicked() {
                                // For general plugins, we call the function named same as ID by default
                                actions.run_plugin = Some((id.clone(), id.clone()));
                                ui.close_menu();
                            }
                        }
                    });
                });
                if ui
                    .add(egui::Button::new(i18n.get("menu-file-settings")).shortcut_text("Ctrl+,"))
                    .clicked()
                {
                    actions.settings = true;
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

            ui.menu_button(i18n.get("menu-build"), |ui| {
                if ui.button(i18n.get("menu-build-deb")).clicked() {
                    actions.build_deb = true;
                    ui.close_menu();
                }
                if ui.button(i18n.get("menu-build-rpm")).clicked() {
                    actions.build_rpm = true;
                    ui.close_menu();
                }
                if ui.button(i18n.get("menu-build-appimage")).clicked() {
                    actions.build_appimage = true;
                    ui.close_menu();
                }
                if ui.button(i18n.get("menu-build-tar-gz")).clicked() {
                    actions.build_tar_gz = true;
                    ui.close_menu();
                }
                if ui.button(i18n.get("menu-build-exe")).clicked() {
                    actions.build_exe = true;
                    ui.close_menu();
                }
                ui.separator();

                ui.menu_button(i18n.get("menu-build-windows"), |ui| {
                    let get_icon = |id: &str| {
                        if *ws.win_tool_available.get(id).unwrap_or(&false) {
                            "✅"
                        } else {
                            "❌"
                        }
                    };

                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("windows-target"),
                            i18n.get("command-name-install-windows-target")
                        ))
                        .clicked()
                    {
                        actions.install_windows_target = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("xwin"),
                            i18n.get("command-name-install-xwin")
                        ))
                        .clicked()
                    {
                        actions.install_xwin = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("clang"),
                            i18n.get("command-name-install-clang")
                        ))
                        .clicked()
                    {
                        actions.install_clang = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("lld"),
                            i18n.get("command-name-install-lld")
                        ))
                        .clicked()
                    {
                        actions.install_lld = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("nsis"),
                            i18n.get("command-name-install-nsis")
                        ))
                        .clicked()
                    {
                        actions.install_nsis = true;
                        ui.close_menu();
                    }
                });

                if ui.button(i18n.get("command-name-install-appimagetool")).clicked() {
                    actions.install_appimagetool = true;
                    ui.close_menu();
                }
                if ui.button(i18n.get("command-name-install-rpm")).clicked() {
                    actions.install_rpm = true;
                    ui.close_menu();
                }
            });

            ui.menu_button(i18n.get("menu-help"), |ui| {
                if ui.button(i18n.get("menu-help-about")).clicked() {
                    actions.about = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .button(format!("❤️ {}", i18n.get("menu-help-support")))
                    .clicked()
                {
                    actions.support = true;
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
            // Logic to start other agents in the terminal
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
            // Initialize new conversation for the selected provider
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
    if actions.install_nsis {
        ws.dep_wizard.open_for_nsis();
    }
    if actions.install_rpm {
        ws.dep_wizard.open_for_rpm();
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
    if actions.build_deb {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("./packaging/deb/build-deb.sh");
        }
    }
    if actions.build_rpm {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo generate-rpm");
        }
    }
    if actions.build_appimage {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo appimage");
        }
    }
    if actions.build_tar_gz {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("cargo build --release && tar -C target/release -czvf polycredo-editor.tar.gz polycredo-editor");
        }
    }
    if actions.build_exe {
        if let Some(t) = &mut ws.build_terminal {
            t.send_command("export PATH=$PATH:/usr/lib/llvm-19/bin && cargo xwin build --release --target x86_64-pc-windows-msvc");
        }
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
