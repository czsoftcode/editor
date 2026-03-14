use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub mod ai_prefs;
mod build_runner;
mod fonts;
pub(crate) mod keymap;
pub(crate) mod local_history;
pub(crate) mod lsp;
mod project_config;
pub(crate) mod project_templates;
pub(crate) mod registry;
mod startup;
pub(crate) mod trash;
mod types;
pub(crate) mod ui;
pub(crate) mod validation;

use crate::app::ui::workspace::state::{
    PendingCloseFlow, PendingCloseMode, build_dirty_close_queue,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GlobalCloseKind {
    /// User requested to quit the entire application (menu or Ctrl+Q equivalent).
    QuitAll,
    /// Root viewport window close (e.g., clicking OS window close button).
    RootViewportClose,
    /// Close project in the root window while keeping the app running.
    RootProjectClose,
}

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
    /// Incoming requests from secondary instances to open project in a new window.
    open_request_rx: Option<mpsc::Receiver<PathBuf>>,

    /// Projects from session that could not be restored (directory does not exist).
    /// Displayed as a toast or warning in the startup dialog.
    missing_session_paths: Vec<PathBuf>,

    /// Last settings version applied to the root viewport context (Audit S-4).
    applied_settings_version: u64,

    /// Pending global close guard flow for Quit/Close Project/root window close.
    /// When set, root close orchestrace waits for workspace unsaved guard to finish
    /// before showing the final confirmation dialog or terminating the app.
    pub(crate) pending_global_close: Option<GlobalCloseKind>,
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

        let mut registry = crate::app::registry::Registry::new();
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

        let shared = Arc::new(Mutex::new(AppShared {
            recent_projects,
            actions: Vec::new(),
            settings: Arc::clone(&settings),
            i18n,
            is_internal_save: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            registry,
            settings_version: std::sync::atomic::AtomicU64::new(1),
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
            open_request_rx,
            missing_session_paths,
            applied_settings_version: 0,
            pending_global_close: None,
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
                AppAction::OpenWithChoice(path) => {
                    // Nastavit pending_open_choice na root workspace.
                    // Wizard callback nemá přímý přístup k ws, proto jde přes AppAction.
                    if let Some(ws) = &mut self.root_ws {
                        ws.pending_open_choice = Some(path);
                    }
                }
                AppAction::CloseWorkspace(vid) => {
                    self.secondary.retain(|sw| sw.viewport_id != vid);
                    self.save_session();
                }
                AppAction::AddRecent(path) => {
                    self.push_recent(path);
                }
                AppAction::QuitAll => {
                    self.start_global_close_guard(GlobalCloseKind::QuitAll, ctx);
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
                            shared.settings.apply(ctx);
                            ws.applied_settings_version = v;
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

    #[cfg(test)]
    pub(crate) fn test_new_with_workspace(ws: WorkspaceState, _ctx: &egui::Context) -> Self {
        use std::sync::mpsc;

        let panel_state = PersistentState::default();
        let settings = std::sync::Arc::new(crate::settings::Settings::default());
        let i18n = std::sync::Arc::new(crate::i18n::I18n::new(&settings.lang));
        let registry = {
            let mut r = crate::app::registry::Registry::new();
            r.init_defaults();
            r
        };

        let shared = Arc::new(Mutex::new(AppShared {
            recent_projects: Vec::new(),
            actions: Vec::new(),
            settings,
            i18n,
            is_internal_save: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            registry,
            settings_version: std::sync::atomic::AtomicU64::new(1),
            bert_model: None,
            bert_tokenizer: None,
        }));

        let (_focus_tx, focus_rx) = mpsc::channel();

        EditorApp {
            root_ws: Some(ws),
            secondary: Vec::new(),
            shared,
            next_viewport_counter: 0,
            saved_panel_state: panel_state,
            privacy_state: PrivacyState::default(),
            path_buffer: String::new(),
            startup_browse_rx: None,
            show_startup_wizard: false,
            startup_wizard: WizardState::default(),
            show_quit_confirm: false,
            quit_confirmed: false,
            show_close_project_confirm: false,
            _ipc_server: None,
            focus_rx,
            open_request_rx: None,
            missing_session_paths: Vec::new(),
            applied_settings_version: 0,
            pending_global_close: None,
        }
    }

    /// Starts a workspace-level unsaved close guard run before executing a
    /// global close action (Quit/Close Project/root window close).
    ///
    /// If there is no workspace or there are no dirty tabs, the original
    /// action is executed immediately. Otherwise, this sets up a
    /// `PendingCloseFlow` in `WorkspaceClose` mode and records the
    /// `pending_global_close` kind so that `update` can resume the original
    /// action once the guard flow finishes or is cancelled.
    pub(crate) fn start_global_close_guard(&mut self, kind: GlobalCloseKind, ctx: &egui::Context) {
        if self.pending_global_close.is_some() {
            return;
        }

        let Some(ws) = self.root_ws.as_mut() else {
            // No workspace open — behave as before.
            match kind {
                GlobalCloseKind::QuitAll => {
                    self.show_close_project_confirm = false;
                    self.show_quit_confirm = true;
                }
                GlobalCloseKind::RootViewportClose => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                GlobalCloseKind::RootProjectClose => {
                    self.show_close_project_confirm = true;
                }
            }
            return;
        };

        // Snapshot dirty tabs across the workspace.
        let tabs_snapshot: Vec<(PathBuf, bool)> = ws
            .editor
            .tabs
            .iter()
            .map(|t| (t.path.clone(), t.modified))
            .collect();

        let active_path = ws.editor.active_path().cloned();
        let queue = build_dirty_close_queue(active_path.as_ref(), &tabs_snapshot);

        if queue.is_empty() {
            // Nothing dirty — execute the original action immediately.
            match kind {
                GlobalCloseKind::QuitAll => {
                    self.show_close_project_confirm = false;
                    self.show_quit_confirm = true;
                }
                GlobalCloseKind::RootViewportClose => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                GlobalCloseKind::RootProjectClose => {
                    self.show_close_project_confirm = true;
                }
            }
            return;
        }

        // Start a workspace-wide guard flow.
        ws.pending_close_flow = Some(PendingCloseFlow {
            mode: PendingCloseMode::WorkspaceClose,
            queue,
            current_index: 0,
            inline_error: None,
        });
        ws.last_unsaved_close_cancelled = false;
        self.pending_global_close = Some(kind);
        // Ensure we process guard dialogs promptly.
        ctx.request_repaint();
    }

    /// Resumes a pending global close action after the workspace-level unsaved
    /// guard flow has finished.
    fn resume_global_close_after_guard(&mut self, ctx: &egui::Context) {
        if let Some(kind) = self.pending_global_close {
            if let Some(ws) = &self.root_ws {
                if ws.pending_close_flow.is_none() {
                    let cancelled = ws.last_unsaved_close_cancelled;
                    match (kind, cancelled) {
                        // User cancelled at least jedno guard rozhodnutí — abort whole close.
                        (_, true) => {
                            self.pending_global_close = None;
                        }
                        // All dirty items resolved (Saved/Discarded) — proceed.
                        (GlobalCloseKind::QuitAll, false) => {
                            self.pending_global_close = None;
                            self.show_close_project_confirm = false;
                            self.show_quit_confirm = true;
                        }
                        (GlobalCloseKind::RootViewportClose, false) => {
                            self.pending_global_close = None;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        (GlobalCloseKind::RootProjectClose, false) => {
                            self.pending_global_close = None;
                            self.show_close_project_confirm = true;
                        }
                    }
                }
            } else {
                // No workspace but still pending flag — nothing to guard, clear it.
                self.pending_global_close = None;
            }
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
                if let Some(ws) = &mut self.root_ws {
                    ws.editor.highlighter.set_theme(theme_name);
                }
                shared.settings.apply(ctx);
                self.applied_settings_version = v;
                // If there's a workspace, sync its version too to prevent double-apply
                if let Some(ws) = &mut self.root_ws {
                    ws.applied_settings_version = v;
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
                // Open project v hlavním okně:
                // křížek má zavřít aktuální projekt (ne celou aplikaci).
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.start_global_close_guard(GlobalCloseKind::RootProjectClose, ctx);
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

        // If a global close guard is pending and the workspace guard flow has finished,
        // resume the original close action based on whether the user cancelled.
        self.resume_global_close_after_guard(ctx);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsaved_close_guard_root_flow() {
        let ctx = egui::Context::default();

        // Minimal workspace with one dirty tab.
        let mut ws = WorkspaceState {
            keymap: crate::app::keymap::Keymap {
                bindings: Vec::new(),
            },
            file_tree: crate::app::ui::file_tree::FileTree::new(),
            editor: crate::app::ui::editor::Editor::new(),
            watcher: crate::watcher::FileWatcher::new(),
            project_watcher: crate::watcher::ProjectWatcher::new(&PathBuf::from("/tmp/test")),
            project_watcher_active: true,
            project_watcher_disconnect_reported: false,
            claude_tabs: Vec::new(),
            claude_active_tab: 0,
            next_claude_tab_id: 1,
            next_terminal_id: 2,
            build_terminal: None,
            retired_terminals: Vec::new(),
            focused_panel: FocusedPanel::Editor,
            root_path: PathBuf::from("/tmp/test"),
            show_left_panel: true,
            show_right_panel: false,
            show_build_terminal: false,
            build_terminal_float: false,
            left_panel_split: 0.5,
            show_about: false,
            show_support: false,
            show_settings: false,
            show_semantic_indexing_modal: false,
            selected_settings_category: None,
            profiles: ProjectProfiles::default(),
            build_errors: Vec::new(),
            build_error_rx: None,
            selected_agent_id: String::new(),
            claude_float: false,
            show_new_project: false,
            wizard: WizardState::default(),
            toasts: Vec::new(),
            folder_pick_rx: None,
            command_palette: None,
            project_index: std::sync::Arc::new(
                crate::app::ui::workspace::index::ProjectIndex::new(PathBuf::from("/tmp/test")),
            ),
            semantic_index: std::sync::Arc::new(std::sync::Mutex::new(
                crate::app::ui::workspace::semantic_index::SemanticIndex::new(PathBuf::from(
                    "/tmp/test",
                )),
            )),
            file_picker: None,
            project_search: crate::app::ui::workspace::state::types::ProjectSearch::default(),
            lsp_client: None,
            lsp_binary_missing: false,
            lsp_install_rx: None,
            git_branch: None,
            git_branch_rx: None,
            git_status_rx: None,
            git_last_refresh: std::time::Instant::now(),
            lsp_last_retry: std::time::Instant::now(),
            settings_draft: None,
            settings_original: None,
            settings_folder_pick_rx: None,
            ai_tool_available: std::collections::HashMap::new(),
            ai_tool_check_rx: None,
            ai_tool_last_check: std::time::Instant::now(),
            win_tool_available: std::collections::HashMap::new(),
            win_tool_check_rx: None,
            win_tool_last_check: std::time::Instant::now(),
            external_change_conflict: None,
            dep_wizard: crate::app::ui::dialogs::DependencyWizard::new(),
            terminal_close_requested: None,
            ai_viewport_open: false,
            settings_conflict: None,
            ai_panel: crate::app::ai_prefs::AiPanelState::default(),
            git_cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            local_history: crate::app::local_history::LocalHistory::new(&PathBuf::from(
                "/tmp/test",
            )),
            background_io_tx: {
                let (tx, _rx) = std::sync::mpsc::channel();
                tx
            },
            background_io_rx: None,
            applied_settings_version: 0,
            confirm_discard_changes: None,
            last_keystroke_time: None,
            pending_close_flow: None,
            last_unsaved_close_cancelled: false,
            history_view: None,
            pending_open_choice: None,
        };

        let dirty_path = ws.root_path.join("dirty.txt");
        ws.editor.tabs.push(crate::app::ui::editor::Tab {
            content: String::new(),
            path: dirty_path.clone(),
            modified: true,
            deleted: false,
            last_edit: None,
            last_autosave_attempt: None,
            save_status: crate::app::ui::editor::SaveStatus::Modified,
            last_saved_content: String::new(),
            scroll_offset: 0.0,
            md_scroll_offset: 0.0,
            last_cursor_range: None,
            is_binary: false,
            image_texture: None,
            binary_data: None,
            svg_modal_shown: false,
            lsp_version: 0,
            lsp_synced_version: 0,
            read_error: false,
            canonical_path: dirty_path.clone(),
            md_cache: egui_commonmark::CommonMarkCache::default(),
        });
        ws.editor.active_tab = Some(0);

        let mut app = EditorApp::test_new_with_workspace(ws, &ctx);
        app.start_global_close_guard(GlobalCloseKind::QuitAll, &ctx);

        assert!(matches!(
            app.pending_global_close,
            Some(GlobalCloseKind::QuitAll)
        ));
        let ws_after = app.root_ws.as_ref().expect("root workspace should exist");
        let flow = ws_after
            .pending_close_flow
            .as_ref()
            .expect("pending close flow should be created");
        assert_eq!(flow.mode, PendingCloseMode::WorkspaceClose);
        assert_eq!(flow.queue.len(), 1);
        assert_eq!(flow.queue[0], dirty_path);
    }

    #[test]
    fn root_project_close_without_dirty_tabs_opens_close_project_confirm() {
        let ctx = egui::Context::default();
        let ws = WorkspaceState {
            keymap: crate::app::keymap::Keymap {
                bindings: Vec::new(),
            },
            file_tree: crate::app::ui::file_tree::FileTree::new(),
            editor: crate::app::ui::editor::Editor::new(),
            watcher: crate::watcher::FileWatcher::new(),
            project_watcher: crate::watcher::ProjectWatcher::new(&PathBuf::from("/tmp/test")),
            project_watcher_active: true,
            project_watcher_disconnect_reported: false,
            claude_tabs: Vec::new(),
            claude_active_tab: 0,
            next_claude_tab_id: 1,
            next_terminal_id: 2,
            build_terminal: None,
            retired_terminals: Vec::new(),
            focused_panel: FocusedPanel::Editor,
            root_path: PathBuf::from("/tmp/test"),
            show_left_panel: true,
            show_right_panel: false,
            show_build_terminal: false,
            build_terminal_float: false,
            left_panel_split: 0.5,
            show_about: false,
            show_support: false,
            show_settings: false,
            show_semantic_indexing_modal: false,
            selected_settings_category: None,
            profiles: ProjectProfiles::default(),
            build_errors: Vec::new(),
            build_error_rx: None,
            selected_agent_id: String::new(),
            claude_float: false,
            show_new_project: false,
            wizard: WizardState::default(),
            toasts: Vec::new(),
            folder_pick_rx: None,
            command_palette: None,
            project_index: std::sync::Arc::new(
                crate::app::ui::workspace::index::ProjectIndex::new(PathBuf::from("/tmp/test")),
            ),
            semantic_index: std::sync::Arc::new(std::sync::Mutex::new(
                crate::app::ui::workspace::semantic_index::SemanticIndex::new(PathBuf::from(
                    "/tmp/test",
                )),
            )),
            file_picker: None,
            project_search: crate::app::ui::workspace::state::types::ProjectSearch::default(),
            lsp_client: None,
            lsp_binary_missing: false,
            lsp_install_rx: None,
            git_branch: None,
            git_branch_rx: None,
            git_status_rx: None,
            git_last_refresh: std::time::Instant::now(),
            lsp_last_retry: std::time::Instant::now(),
            settings_draft: None,
            settings_original: None,
            settings_folder_pick_rx: None,
            ai_tool_available: std::collections::HashMap::new(),
            ai_tool_check_rx: None,
            ai_tool_last_check: std::time::Instant::now(),
            win_tool_available: std::collections::HashMap::new(),
            win_tool_check_rx: None,
            win_tool_last_check: std::time::Instant::now(),
            external_change_conflict: None,
            dep_wizard: crate::app::ui::dialogs::DependencyWizard::new(),
            terminal_close_requested: None,
            ai_viewport_open: false,
            settings_conflict: None,
            ai_panel: crate::app::ai_prefs::AiPanelState::default(),
            git_cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            local_history: crate::app::local_history::LocalHistory::new(&PathBuf::from(
                "/tmp/test",
            )),
            background_io_tx: {
                let (tx, _rx) = std::sync::mpsc::channel();
                tx
            },
            background_io_rx: None,
            applied_settings_version: 0,
            confirm_discard_changes: None,
            last_keystroke_time: None,
            pending_close_flow: None,
            last_unsaved_close_cancelled: false,
            history_view: None,
            pending_open_choice: None,
        };

        let mut app = EditorApp::test_new_with_workspace(ws, &ctx);
        app.start_global_close_guard(GlobalCloseKind::RootProjectClose, &ctx);

        assert!(app.pending_global_close.is_none());
        assert!(app.show_close_project_confirm);
        assert!(!app.show_quit_confirm);
    }
}
