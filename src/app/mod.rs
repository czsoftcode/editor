use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub mod ai;
mod build_runner;
mod fonts;
pub mod local_history;
pub mod lsp;
mod project_config;
pub(crate) mod project_templates;
pub mod registry;
pub mod sandbox;
mod startup;
mod types;
pub(crate) mod ui;
pub(crate) mod validation;

use types::*;
use ui::dialogs::{
    PrivacyResult, PrivacyState, QuitDialogResult, WizardState, show_close_project_confirm_dialog,
    show_privacy_dialog, show_quit_confirm_dialog,
};
use ui::workspace::{
    SecondaryWorkspace, WorkspaceState, init_workspace, render_workspace, ws_to_panel_state,
};

use crate::config;
use crate::ipc::{self, Ipc, IpcServer};

use eframe::egui;

// ---------------------------------------------------------------------------
// EditorApp — Main application (root viewport)
// ---------------------------------------------------------------------------

use crate::tr;

pub struct EditorApp {
    /// Root workspace (None = startup dialog)
    root_ws: Option<WorkspaceState>,
    /// Secondary workspaces (behind Arc<Mutex> for deferred viewports)
    secondary: Vec<SecondaryWorkspace>,
    /// Shared state — communication between viewports
    shared: Arc<Mutex<AppShared>>,
    /// Counter for unique ViewportId
    next_viewport_counter: u64,

    /// State saved for restoration when closing/opening workspace
    saved_panel_state: PersistentState,

    privacy_state: PrivacyState,

    // --- Startup dialog ---
    path_buffer: String,
    startup_browse_rx: Option<mpsc::Receiver<Option<PathBuf>>>,

    // --- New project wizard (startup screen) ---
    show_startup_wizard: bool,
    startup_wizard: WizardState,

    // --- Application termination ---
    show_quit_confirm: bool,
    quit_confirmed: bool,
    // --- Closing active project in root window ---
    show_close_project_confirm: bool,

    _ipc_server: Option<IpcServer>,
    focus_rx: mpsc::Receiver<()>,
    action_rx: mpsc::Receiver<AppAction>,
    /// Incoming requests from secondary instances to open project in a new window.
    open_request_rx: Option<mpsc::Receiver<PathBuf>>,

    /// Projects from session that could not be restored (directory does not exist).
    /// Displayed as a toast or warning in the startup dialog.
    missing_session_paths: Vec<PathBuf>,

    /// Last settings version applied to the root viewport context (Audit S-4).
    applied_settings_version: u64,
}

// ---------------------------------------------------------------------------
// EditorApp — Implementation
// ---------------------------------------------------------------------------

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext, root_path: Option<PathBuf>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        fonts::setup_custom_fonts(&cc.egui_ctx);

        let panel_state: PersistentState = cc
            .storage
            .and_then(|s| eframe::get_value(s, STORAGE_KEY))
            .unwrap_or_default();

        let (ipc_server, open_request_rx) = match IpcServer::start() {
            Some((server, rx)) => (Some(server), Some(rx)),
            None => (None, None),
        };
        let (action_tx, action_rx) = mpsc::channel();
        let focus_rx = ipc::start_process_listener();

        // Load recent projects
        let recent_projects = Ipc::recent();

        // Determine list of projects to open
        let (paths_to_open, missing_session_paths): (Vec<PathBuf>, Vec<PathBuf>) =
            if let Some(p) = root_path {
                // CLI argument — open only this project
                (vec![p], vec![])
            } else {
                // Session restore: distinguish between found and missing projects
                ipc::load_session_checked()
            };

        let settings = std::sync::Arc::new(crate::settings::Settings::load());
        // Apply theme before first frame to avoid startup flash.
        settings.apply(&cc.egui_ctx);
        let i18n = std::sync::Arc::new(crate::i18n::I18n::new(&settings.lang));

        let sandbox_root = paths_to_open
            .first()
            .map(|p| p.join(".polycredo").join("sandbox"))
            .unwrap_or_else(|| PathBuf::from("/tmp/polycredo-sandbox"));

        let mut registry = crate::app::registry::Registry::new(sandbox_root);
        *registry.plugins.action_sender.lock().expect("lock") = Some(action_tx);
        *registry.plugins.egui_ctx.lock().expect("lock") = Some(cc.egui_ctx.clone());
        registry.init_defaults();

        // Register agents exclusively from settings
        for ca in &settings.custom_agents {
            let cmd = if ca.args.is_empty() {
                ca.command.clone()
            } else {
                format!("{} {}", ca.command, ca.args)
            };
            registry.agents.register(crate::app::registry::Agent {
                id: ca.name.to_lowercase().replace(' ', "_"),
                label: ca.name.clone(),
                command: cmd,
                context_aware: true,
            });
        }

        // Load WASM plugins from Global fallback
        registry.plugins.set_blacklist(settings.blacklist.clone());

        // Priority: Global Fallback (~/.config/polycredo-editor/plugins)
        let global_plugins = ipc::plugins_dir();
        if let Err(e) = registry.plugins.load_from_dir(&global_plugins) {
            eprintln!(
                "Failed to load global plugins from {:?}: {}",
                global_plugins, e
            );
        }

        // Auto-authorize enabled plugins that require internet access (allowed_hosts)
        let pending = registry.plugins.get_pending_authorizations();
        for (id, _metadata) in pending {
            let is_enabled = settings.plugins.get(&id).map(|s| s.enabled).unwrap_or(true);
            if is_enabled {
                let config = settings
                    .plugins
                    .get(&id)
                    .map(|s| s.config.clone())
                    .unwrap_or_default();
                if let Err(e) = registry.plugins.authorize(&id, &config) {
                    eprintln!("Failed to auto-authorize plugin {}: {}", id, e);
                }
            }
        }

        // Auto-register "hello" plugin command if loaded
        if registry
            .plugins
            .get_loaded_ids()
            .contains(&"hello".to_string())
        {
            registry.commands.register(crate::app::registry::Command {
                id: "plugin.hello".to_string(),
                i18n_key: "command-name-plugin-hello",
                shortcut: None,
                action: crate::app::registry::CommandAction::Plugin {
                    plugin_id: "hello".to_string(),
                    func_name: "hello".to_string(),
                },
            });
        }

        // Dynamically register AI agent commands
        let ai_agents = registry.plugins.get_ai_agents();
        for (id, _meta) in ai_agents {
            let cmd_id = format!("plugin.{}", id);
            let func_name = format!("ask_{}", id);

            // We use a generic i18n key or construct one if needed.
            // For now, we'll try to find a specific one or fallback to a default.
            let i18n_key = if id == "gemini" {
                "command-name-plugin-gemini"
            } else if id == "ollama" {
                "command-name-plugin-ollama"
            } else {
                "command-name-plugin-ai-chat" // We should add this to locales
            };

            registry.commands.register(crate::app::registry::Command {
                id: cmd_id,
                i18n_key,
                shortcut: None,
                action: crate::app::registry::CommandAction::Plugin {
                    plugin_id: id.clone(),
                    func_name,
                },
            });
        }

        let shared = Arc::new(Mutex::new(AppShared {
            recent_projects,
            actions: Vec::new(),
            settings: Arc::clone(&settings),
            i18n,
            is_internal_save: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            registry,
            settings_version: std::sync::atomic::AtomicU64::new(1),
            sandbox_off_toast_shown: false,
            bert_model: None,
            bert_tokenizer: None,
        }));

        // Register all projects
        for p in &paths_to_open {
            Ipc::register(p);
        }

        // Add to recent
        for p in &paths_to_open {
            Ipc::add_recent(p);
        }

        // Initialize root workspace
        let root_ws = paths_to_open.first().map(|p| {
            init_workspace(
                p.clone(),
                &panel_state,
                cc.egui_ctx.clone(),
                &settings,
                Arc::clone(&shared),
            )
        });

        // Initialize secondary workspaces from session
        let mut counter = 0u64;
        let secondary: Vec<SecondaryWorkspace> = paths_to_open
            .get(1..)
            .unwrap_or(&[])
            .iter()
            .map(|p| {
                let vid = egui::ViewportId::from_hash_of(format!("workspace_{}", counter));
                counter += 1;
                SecondaryWorkspace {
                    viewport_id: vid,
                    state: Arc::new(Mutex::new(init_workspace(
                        p.clone(),
                        &panel_state,
                        cc.egui_ctx.clone(),
                        &settings,
                        Arc::clone(&shared),
                    ))),
                    close_requested: Arc::new(AtomicBool::new(false)),
                }
            })
            .collect();

        // Update local cache of recent projects
        {
            let mut s = shared
                .lock()
                .expect("Failed to lock AppShared during initialization");
            for p in &paths_to_open {
                s.recent_projects.retain(|rp| rp != p);
                s.recent_projects.insert(0, p.clone());
            }
            s.recent_projects.truncate(config::MAX_RECENT_PROJECTS);
        }

        let home = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .to_string_lossy()
            .to_string();

        let projects_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join("MyProject")
            .to_string_lossy()
            .to_string();
        let startup_wizard = WizardState {
            path: projects_dir,
            ..WizardState::default()
        };

        Self {
            root_ws,
            secondary,
            shared,
            next_viewport_counter: counter,
            saved_panel_state: panel_state,
            privacy_state: PrivacyState::default(),
            path_buffer: home,
            startup_browse_rx: None,
            show_startup_wizard: false,
            startup_wizard,
            show_quit_confirm: false,
            quit_confirmed: false,
            show_close_project_confirm: false,
            _ipc_server: ipc_server,
            focus_rx,
            action_rx,
            open_request_rx,
            missing_session_paths,
            applied_settings_version: 0,
        }
    }

    fn current_panel_state(&self) -> PersistentState {
        self.root_ws
            .as_ref()
            .map(ws_to_panel_state)
            .unwrap_or_else(|| self.saved_panel_state.clone())
    }

    fn all_open_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(ws) = &self.root_ws {
            paths.push(ws.root_path.clone());
        }
        for sw in &self.secondary {
            if let Ok(ws) = sw.state.try_lock() {
                paths.push(ws.root_path.clone());
            }
        }
        paths
    }

    fn save_session(&self) {
        let paths = self.all_open_paths();
        ipc::save_session(&paths);
    }

    fn push_recent(&mut self, path: PathBuf) {
        Ipc::add_recent(&path);
        let mut shared = self
            .shared
            .lock()
            .expect("Failed to lock AppShared in push_recent");
        shared.recent_projects.retain(|p| p != &path);
        shared.recent_projects.insert(0, path);
        shared.recent_projects.truncate(config::MAX_RECENT_PROJECTS);
    }

    fn open_in_new_window(&mut self, path: PathBuf, ctx: &egui::Context) {
        // Check if project is already open in this process
        let already_open = self
            .root_ws
            .as_ref()
            .map(|ws| ws.root_path == path)
            .unwrap_or(false)
            || self.secondary.iter().any(|sw| {
                sw.state
                    .try_lock()
                    .ok()
                    .map(|ws| ws.root_path == path)
                    .unwrap_or(false)
            });
        if already_open {
            return;
        }

        let vid =
            egui::ViewportId::from_hash_of(format!("workspace_{}", self.next_viewport_counter));
        self.next_viewport_counter += 1;
        let panel_state = self.current_panel_state();
        let settings = self.shared.lock().expect("lock").settings.clone();
        let ws = init_workspace(
            path.clone(),
            &panel_state,
            ctx.clone(),
            &settings,
            Arc::clone(&self.shared),
        );
        self.secondary.push(SecondaryWorkspace {
            viewport_id: vid,
            state: Arc::new(Mutex::new(ws)),
            close_requested: Arc::new(AtomicBool::new(false)),
        });
        Ipc::register(&path);
        self.push_recent(path);
        self.save_session();
        // Force immediate repaint so new window registers in this frame
        ctx.request_repaint();
    }

    fn process_actions(&mut self, ctx: &egui::Context) {
        // Forward incremental actions from plugins (monologues)
        while let Ok(action) = self.action_rx.try_recv() {
            self.shared.lock().expect("lock").actions.push(action);
        }

        let mut actions = {
            let mut sh = self
                .shared
                .lock()
                .expect("Failed to lock AppShared in process_actions");
            std::mem::take(&mut sh.actions)
        };

        let start_time = std::time::Instant::now();

        // Process actions with a time limit (Plan 03: 2ms)
        while !actions.is_empty() {
            let action = actions.remove(0);

            match action {
                AppAction::OpenInNewWindow(path) => {
                    self.open_in_new_window(path, ctx);
                }
                AppAction::CloseWorkspace(vid) => {
                    self.secondary.retain(|sw| sw.viewport_id != vid);
                    self.save_session();
                }
                AppAction::AddRecent(path) => {
                    self.push_recent(path);
                }
                AppAction::QuitAll => {
                    self.show_close_project_confirm = false;
                    self.show_quit_confirm = true;
                }
                AppAction::PluginResponse(id, result) => {
                    if let Some(ws) = &mut self.root_ws
                        && id == ws.ai_selected_provider
                    {
                        ws.ai_loading = false;
                        match result {
                            Ok(text) => {
                                ws.ai_response = Some(text.clone());
                                // Update conversation history - append AFTER monologue
                                if let Some(last) = ws.ai_conversation.last_mut() {
                                    if !last.1.is_empty() {
                                        last.1.push_str("\n\n");
                                    }
                                    last.1.push_str(&text);
                                }
                            }
                            Err(err) => ws.plugin_error = Some(err),
                        }
                    }
                }
                AppAction::PluginMonologue(id, message) => {
                    if let Some(ws) = &mut self.root_ws
                        && id == ws.ai_selected_provider
                    {
                        ws.ai_monologue.push(message.clone());
                    }
                }
                AppAction::PluginUsage(id, in_t, out_t) => {
                    if let Some(ws) = &mut self.root_ws
                        && id == ws.ai_selected_provider
                    {
                        // Add the input and output tokens consumed by the last request to the session counter.
                        ws.ai_in_tokens = ws.ai_in_tokens.saturating_add(in_t);
                        ws.ai_out_tokens = ws.ai_out_tokens.saturating_add(out_t);
                    }
                }
                AppAction::PluginPayload(id, payload) => {
                    if let Some(ws) = &mut self.root_ws
                        && id == ws.ai_selected_provider
                    {
                        ws.ai_last_payload = payload;
                    }
                }
                AppAction::PluginApprovalRequest(id, action, details, sender) => {
                    if let Some(ws) = &mut self.root_ws
                        && id == ws.ai_selected_provider
                    {
                        ws.pending_plugin_approval = Some((id, action, details, sender));
                        ws.ai_focus_requested = true;
                    }
                }
                AppAction::PluginAskUser(id, question, options, sender) => {
                    if let Some(ws) = &mut self.root_ws
                        && id == ws.ai_selected_provider
                    {
                        ws.pending_ask_user = Some((id, question, options, String::new(), sender));
                        ws.ai_focus_requested = true;
                    }
                }
                AppAction::PluginCompleted(id, summary) => {
                    if let Some(ws) = &mut self.root_ws
                        && id == ws.ai_selected_provider
                    {
                        ws.ai_monologue.push(format!("✅ DONE: {}", summary));
                        ws.ai_loading = false;
                    }
                }
            }

            // If we've spent more than 2ms, defer remaining actions to the next frame
            if start_time.elapsed().as_millis() >= 2 && !actions.is_empty() {
                let mut sh = self.shared.lock().expect("lock");
                // Prepend remaining actions back to the queue
                let mut combined = actions;
                combined.append(&mut sh.actions);
                sh.actions = combined;
                ctx.request_repaint(); // Ensure we come back soon to finish
                break;
            }
        }
    }

    fn register_deferred_viewports(&self, ctx: &egui::Context) {
        for sw in &self.secondary {
            let ws_arc = Arc::clone(&sw.state);
            let shared_arc = Arc::clone(&self.shared);
            let close_requested = Arc::clone(&sw.close_requested);
            let vid = sw.viewport_id;

            let title = sw
                .state
                .try_lock()
                .map(|ws| format!("PolyCredo Editor — {}", ws.root_path.display()))
                .unwrap_or_else(|_| "PolyCredo Editor".to_string());

            let shared_for_init = Arc::clone(&self.shared);
            ctx.show_viewport_deferred(
                vid,
                egui::ViewportBuilder::default()
                    .with_title(title)
                    .with_inner_size([config::WINDOW_DEFAULT_WIDTH, config::WINDOW_DEFAULT_HEIGHT]),
                move |ctx, _class| {
                    // Closing secondary window — show confirmation in the given viewport
                    if ctx.input(|i| i.viewport().close_requested()) {
                        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                        close_requested.store(true, Ordering::SeqCst);
                    }

                    let mut ws = ws_arc
                        .lock()
                        .expect("Failed to lock WorkspaceState in deferred viewport");

                    // Apply settings only when changed
                    {
                        let shared = shared_arc
                            .lock()
                            .expect("Failed to lock AppShared in deferred viewport settings apply");
                        let v = shared
                            .settings_version
                            .load(std::sync::atomic::Ordering::SeqCst);
                        if ws.applied_settings_version != v {
                            let theme_name = shared.settings.syntect_theme_name();
                            ws.editor.highlighter.set_theme(theme_name);
                            let settings_snapshot = (*shared.settings).clone();
                            shared.settings.apply(ctx);
                            ws.applied_settings_version = v;

                            if ws.sandbox_mode_enabled != settings_snapshot.sandbox_mode {
                                if ws.settings_draft.is_some() {
                                    ws.settings_conflict = Some(
                                        crate::app::ui::workspace::state::SettingsConflict {
                                            new_settings: settings_snapshot.clone(),
                                        },
                                    );
                                }
                                let needs_request = ws
                                    .pending_sandbox_apply
                                    .as_ref()
                                    .map(|req| req.version != v)
                                    .unwrap_or(true);
                                if needs_request {
                                    ws.pending_sandbox_apply = Some(
                                        crate::app::ui::workspace::state::SandboxApplyRequest {
                                            target_mode: settings_snapshot.sandbox_mode,
                                            version: v,
                                            defer_until_clear: true,
                                            force_apply: false,
                                            prompted: false,
                                            notify_on_apply: true,
                                        },
                                    );
                                }
                            }
                        }
                    }

                    if let Some(new_path) = render_workspace(ctx, &mut ws, &shared_arc) {
                        let panel = ws_to_panel_state(&ws);
                        let new_path_clone = new_path.clone();
                        let settings = shared_arc.lock().expect("lock").settings.clone();
                        *ws = init_workspace(
                            new_path,
                            &panel,
                            ctx.clone(),
                            &settings,
                            Arc::clone(&shared_for_init),
                        );
                        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                            "PolyCredo Editor — {}",
                            ws.root_path.display()
                        )));
                        shared_arc
                            .lock()
                            .expect("Failed to lock AppShared in AddRecent action")
                            .actions
                            .push(AppAction::AddRecent(new_path_clone));
                    }

                    if close_requested.load(Ordering::SeqCst) {
                        let modal_id = format!("close_project_modal_{vid:?}");
                        let project_path = ws.root_path.display().to_string();
                        let i18n_arc = {
                            std::sync::Arc::clone(
                                &shared_arc
                                    .lock()
                                    .expect(
                                        "Failed to lock AppShared for i18n in deferred viewport",
                                    )
                                    .i18n,
                            )
                        };
                        match show_close_project_confirm_dialog(
                            ctx,
                            &modal_id,
                            &project_path,
                            &i18n_arc,
                        ) {
                            QuitDialogResult::Confirmed => {
                                close_requested.store(false, Ordering::SeqCst);
                                shared_arc
                                    .lock()
                                    .expect("Failed to lock AppShared in CloseWorkspace action")
                                    .actions
                                    .push(AppAction::CloseWorkspace(vid));
                            }
                            QuitDialogResult::Cancelled => {
                                close_requested.store(false, Ordering::SeqCst);
                            }
                            QuitDialogResult::Open => {}
                        }
                    }
                },
            );
        }
    }

    fn show_quit_confirm_dialog(&mut self, ctx: &egui::Context) {
        let i18n_arc = {
            std::sync::Arc::clone(
                &self
                    .shared
                    .lock()
                    .expect("Failed to lock AppShared for i18n in quit confirmation")
                    .i18n,
            )
        };
        match show_quit_confirm_dialog(ctx, &i18n_arc) {
            QuitDialogResult::Confirmed => {
                self.save_session();
                self.quit_confirmed = true;
                self.secondary.clear();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            QuitDialogResult::Cancelled => {
                self.show_quit_confirm = false;
            }
            QuitDialogResult::Open => {}
        }
    }

    fn show_close_project_confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some(ws) = self.root_ws.as_ref() else {
            self.show_close_project_confirm = false;
            return;
        };

        let project_path = ws.root_path.display().to_string();
        let i18n_arc = {
            std::sync::Arc::clone(
                &self
                    .shared
                    .lock()
                    .expect("Failed to lock AppShared for i18n in close project confirmation")
                    .i18n,
            )
        };
        match show_close_project_confirm_dialog(
            ctx,
            "close_project_root_modal",
            &project_path,
            &i18n_arc,
        ) {
            QuitDialogResult::Confirmed => {
                self.save_session();
                self.root_ws = None;
                self.show_close_project_confirm = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Title("PolyCredo Editor".to_string()));
            }
            QuitDialogResult::Cancelled => {
                self.show_close_project_confirm = false;
            }
            QuitDialogResult::Open => {}
        }
    }
}

// Drop — Cleanup on exit
// ---------------------------------------------------------------------------

impl Drop for EditorApp {
    fn drop(&mut self) {
        self.save_session();
        Ipc::unregister();
        let _ = std::fs::remove_file(ipc::process_socket_path(std::process::id()));
    }
}

// ---------------------------------------------------------------------------
// eframe::App implementation
// ---------------------------------------------------------------------------

impl eframe::App for EditorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let ps = self.current_panel_state();
        eframe::set_value(storage, STORAGE_KEY, &ps);
        self.save_session();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply settings (theme, font) only when they change
        {
            let shared = self
                .shared
                .lock()
                .expect("Failed to lock AppShared for settings apply");
            let v = shared
                .settings_version
                .load(std::sync::atomic::Ordering::SeqCst);
            if self.applied_settings_version != v {
                let theme_name = shared.settings.syntect_theme_name();
                let settings_snapshot = (*shared.settings).clone();
                if let Some(ws) = &mut self.root_ws {
                    ws.editor.highlighter.set_theme(theme_name);
                }
                shared.settings.apply(ctx);
                self.applied_settings_version = v;
                // If there's a workspace, sync its version too to prevent double-apply
                if let Some(ws) = &mut self.root_ws {
                    ws.applied_settings_version = v;
                    if ws.sandbox_mode_enabled != settings_snapshot.sandbox_mode {
                        if ws.settings_draft.is_some() {
                            ws.settings_conflict = Some(
                                crate::app::ui::workspace::state::SettingsConflict {
                                    new_settings: settings_snapshot.clone(),
                                },
                            );
                        }
                        let needs_request = ws
                            .pending_sandbox_apply
                            .as_ref()
                            .map(|req| req.version != v)
                            .unwrap_or(true);
                        if needs_request {
                            ws.pending_sandbox_apply =
                                Some(crate::app::ui::workspace::state::SandboxApplyRequest {
                                    target_mode: settings_snapshot.sandbox_mode,
                                    version: v,
                                    defer_until_clear: true,
                                    force_apply: false,
                                    prompted: false,
                                    notify_on_apply: true,
                                });
                        }
                    }
                }
            }
        }

        let (privacy_accepted, i18n_arc) = {
            let shared = self
                .shared
                .lock()
                .expect("Failed to lock AppShared for privacy check");
            (shared.settings.privacy_accepted, Arc::clone(&shared.i18n))
        };

        if !privacy_accepted {
            match show_privacy_dialog(ctx, &mut self.privacy_state, &i18n_arc) {
                PrivacyResult::Accepted => {
                    let mut shared = self
                        .shared
                        .lock()
                        .expect("Failed to lock AppShared for privacy acceptance");
                    let mut settings = (*shared.settings).clone();
                    settings.privacy_accepted = true;
                    settings.save();
                    shared.settings = Arc::new(settings);
                }
                PrivacyResult::LanguageChanged(new_lang) => {
                    let mut shared = self
                        .shared
                        .lock()
                        .expect("Failed to lock AppShared for language change");
                    let mut settings = (*shared.settings).clone();
                    settings.lang = new_lang.clone();
                    shared.settings = Arc::new(settings);
                    shared.i18n = std::sync::Arc::new(crate::i18n::I18n::new(&new_lang));
                    // Reset content to reload in the new language
                    self.privacy_state.content = None;
                }
                PrivacyResult::None => {}
            }
            return;
        }

        // IPC focus request
        if self.focus_rx.try_recv().is_ok() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // Requests from secondary instances to open project in a new window
        let ipc_open_paths: Vec<PathBuf> = self
            .open_request_rx
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();
        for path in ipc_open_paths {
            self.open_in_new_window(path, ctx);
        }

        // Catch root window close request
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.quit_confirmed {
                // Confirmed — let it close
            } else if self.root_ws.is_some() {
                // Open project — request close confirmation.
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_close_project_confirm = true;
            } else {
                // Startup dialog — terminate application
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_quit_confirm = true;
            }
        }

        // Register deferred viewports for existing secondary workspaces
        // (must be called every frame to keep windows open)
        self.register_deferred_viewports(ctx);

        // Render content of root viewport
        if self.root_ws.is_none() {
            startup::render(
                ctx,
                startup::StartupState {
                    root_ws: &mut self.root_ws,
                    shared: &self.shared,
                    path_buffer: &mut self.path_buffer,
                    show_startup_wizard: &mut self.show_startup_wizard,
                    startup_wizard: &mut self.startup_wizard,
                    startup_browse_rx: &mut self.startup_browse_rx,
                    missing_session_paths: &mut self.missing_session_paths,
                    saved_panel_state: &self.saved_panel_state,
                },
            );
        } else {
            // Temporarily take root_ws to allow calling &mut self
            let mut ws = self.root_ws.take().unwrap();
            // One-time info toasts about projects that could not be restored from session
            if !self.missing_session_paths.is_empty() {
                let i18n_arc = {
                    std::sync::Arc::clone(
                        &self
                            .shared
                            .lock()
                            .expect("Failed to lock AppShared for i18n in missing session restore")
                            .i18n,
                    )
                };
                let i18n = &*i18n_arc;
                for path in self.missing_session_paths.drain(..) {
                    ws.toasts.push(Toast::info(tr!(
                        i18n,
                        "error-session-restore",
                        path = path.to_string_lossy().into_owned()
                    )));
                }
            }
            let reinit = render_workspace(ctx, &mut ws, &self.shared);
            if let Some(new_path) = reinit {
                let panel = ws_to_panel_state(&ws);
                let new_path_clone = new_path.clone();
                let settings = self.shared.lock().expect("lock").settings.clone();
                ws = init_workspace(
                    new_path,
                    &panel,
                    ctx.clone(),
                    &settings,
                    Arc::clone(&self.shared),
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                    "PolyCredo Editor — {}",
                    ws.root_path.display()
                )));
                self.push_recent(new_path_clone);
                self.save_session();
            }
            self.root_ws = Some(ws);
        }

        // Process actions from this frame (new projects, closed workspaces, etc.)
        // Called AFTER render so click actions are processed immediately
        self.process_actions(ctx);

        // Re-register viewports — for new secondary workspaces added above
        self.register_deferred_viewports(ctx);

        // Close confirmation dialog
        if self.show_close_project_confirm {
            self.show_close_project_confirm_dialog(ctx);
        }

        // Application quit confirmation dialog
        if self.show_quit_confirm {
            self.show_quit_confirm_dialog(ctx);
        }
    }
}
