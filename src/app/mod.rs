use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

pub(crate) mod modules;
mod types;
mod build_runner;
mod dialogs;
mod workspace;

use types::*;
use dialogs::{WizardState, show_project_wizard, show_quit_confirm_dialog, QuitDialogResult,
              show_startup_dialog, StartupAction};
use workspace::{WorkspaceState, SecondaryWorkspace, ws_to_panel_state, init_workspace,
                render_workspace};

use crate::config;
use crate::ipc::{self, Ipc, IpcServer};

use eframe::egui;

// ---------------------------------------------------------------------------
// EditorApp — hlavní aplikace (kořenový viewport)
// ---------------------------------------------------------------------------

pub struct EditorApp {
    /// Kořenový workspace (None = startup dialog)
    root_ws: Option<WorkspaceState>,
    /// Sekundární workspacy (za Arc<Mutex> pro deferred viewporty)
    secondary: Vec<SecondaryWorkspace>,
    /// Sdílený stav — komunikace mezi viewporty
    shared: Arc<Mutex<AppShared>>,
    /// Čítač pro jedinečné ViewportId
    next_viewport_counter: u64,

    /// Stav uložený pro obnovení při zavření/otevření workspace
    saved_panel_state: PersistentState,

    // --- Startup dialog ---
    path_buffer: String,

    // --- Wizard nového projektu (startup screen) ---
    show_startup_wizard: bool,
    startup_wizard: WizardState,

    // --- Ukončení aplikace ---
    show_quit_confirm: bool,
    quit_confirmed: bool,

    _ipc_server: Option<IpcServer>,
    focus_rx: mpsc::Receiver<()>,
}

// ---------------------------------------------------------------------------
// EditorApp — implementace
// ---------------------------------------------------------------------------

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext, root_path: Option<PathBuf>) -> Self {
        let panel_state: PersistentState = cc.storage
            .and_then(|s| eframe::get_value(s, STORAGE_KEY))
            .unwrap_or_default();

        let ipc_server = IpcServer::start();
        let focus_rx = ipc::start_process_listener();

        // Načíst nedávné projekty
        let recent_projects = Ipc::recent();

        // Určit seznam projektů k otevření
        let paths_to_open: Vec<PathBuf> = if let Some(p) = root_path {
            // CLI argument — otevřít jen tento projekt
            vec![p]
        } else {
            // Session restore
            ipc::load_session()
        };

        // Registrovat všechny projekty
        for p in &paths_to_open {
            Ipc::register(p);
        }

        // Přidat do nedávných
        for p in &paths_to_open {
            Ipc::add_recent(p);
        }

        // Inicializovat kořenový workspace
        let root_ws = paths_to_open.first().map(|p| {
            init_workspace(p.clone(), &panel_state)
        });

        // Inicializovat sekundární workspacy ze session
        let mut counter = 0u64;
        let secondary: Vec<SecondaryWorkspace> = paths_to_open.get(1..)
            .unwrap_or(&[])
            .iter()
            .map(|p| {
                let vid = egui::ViewportId::from_hash_of(format!("workspace_{}", counter));
                counter += 1;
                SecondaryWorkspace {
                    viewport_id: vid,
                    state: Arc::new(Mutex::new(init_workspace(p.clone(), &panel_state))),
                }
            })
            .collect();

        let shared = Arc::new(Mutex::new(AppShared {
            recent_projects,
            actions: Vec::new(),
        }));

        // Aktualizovat lokální cache nedávných
        {
            let mut s = shared.lock().unwrap();
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

        Self {
            root_ws,
            secondary,
            shared,
            next_viewport_counter: counter,
            saved_panel_state: panel_state,
            path_buffer: home,
            show_startup_wizard: false,
            startup_wizard: WizardState { path: projects_dir, ..WizardState::default() },
            show_quit_confirm: false,
            quit_confirmed: false,
            _ipc_server: ipc_server,
            focus_rx,
        }
    }

    fn current_panel_state(&self) -> PersistentState {
        self.root_ws.as_ref()
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
        let mut shared = self.shared.lock().unwrap();
        shared.recent_projects.retain(|p| p != &path);
        shared.recent_projects.insert(0, path);
        shared.recent_projects.truncate(config::MAX_RECENT_PROJECTS);
    }

    fn open_in_new_window(&mut self, path: PathBuf, ctx: &egui::Context) {
        // Zkontrolovat, zda projekt již není otevřen v tomto procesu
        let already_open = self.root_ws.as_ref().map(|ws| ws.root_path == path).unwrap_or(false)
            || self.secondary.iter().any(|sw| {
                sw.state.try_lock().ok().map(|ws| ws.root_path == path).unwrap_or(false)
            });
        if already_open {
            return;
        }

        let vid = egui::ViewportId::from_hash_of(format!("workspace_{}", self.next_viewport_counter));
        self.next_viewport_counter += 1;
        let panel_state = self.current_panel_state();
        let ws = init_workspace(path.clone(), &panel_state);
        self.secondary.push(SecondaryWorkspace {
            viewport_id: vid,
            state: Arc::new(Mutex::new(ws)),
        });
        Ipc::register(&path);
        self.push_recent(path);
        self.save_session();
        // Vynutit okamžitý repaint, aby se nové okno registrovalo v tomto snímku
        ctx.request_repaint();
    }

    fn process_actions(&mut self, ctx: &egui::Context) {
        let actions = std::mem::take(&mut self.shared.lock().unwrap().actions);
        for action in actions {
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
                    self.show_quit_confirm = true;
                }
            }
        }
    }

    fn register_deferred_viewports(&self, ctx: &egui::Context) {
        for sw in &self.secondary {
            let ws_arc = Arc::clone(&sw.state);
            let shared_arc = Arc::clone(&self.shared);
            let vid = sw.viewport_id;

            let title = sw.state.try_lock()
                .map(|ws| format!("Rust Editor — {}", ws.root_path.display()))
                .unwrap_or_else(|_| "Rust Editor".to_string());

            ctx.show_viewport_deferred(
                vid,
                egui::ViewportBuilder::default()
                    .with_title(title)
                    .with_inner_size([config::WINDOW_DEFAULT_WIDTH, config::WINDOW_DEFAULT_HEIGHT]),
                move |ctx, _class| {
                    // Zavření sekundárního okna — informovat root viewport
                    if ctx.input(|i| i.viewport().close_requested()) {
                        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                        shared_arc.lock().unwrap().actions.push(AppAction::CloseWorkspace(vid));
                        return;
                    }

                    let mut ws = ws_arc.lock().unwrap();
                    if let Some(new_path) = render_workspace(ctx, &mut ws, &shared_arc) {
                        let panel = ws_to_panel_state(&ws);
                        let new_path_clone = new_path.clone();
                        *ws = init_workspace(new_path, &panel);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                            format!("Rust Editor — {}", ws.root_path.display())
                        ));
                        shared_arc.lock().unwrap().actions.push(AppAction::AddRecent(new_path_clone));
                    }
                },
            );
        }
    }

    fn show_quit_confirm_dialog(&mut self, ctx: &egui::Context) {
        match show_quit_confirm_dialog(ctx) {
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

    fn do_startup_dialog(&mut self, ctx: &egui::Context) {
        let recent_snapshot = self.shared.lock().unwrap().recent_projects.clone();
        match show_startup_dialog(ctx, &mut self.path_buffer, self.show_startup_wizard, &recent_snapshot) {
            StartupAction::OpenPath(path) => {
                self.open_workspace_from_startup(ctx, path);
            }
            StartupAction::OpenWizard => {
                self.show_startup_wizard = true;
            }
            StartupAction::None => {}
        }
    }

    fn open_workspace_from_startup(&mut self, ctx: &egui::Context, path: PathBuf) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            format!("Rust Editor — {}", path.display()),
        ));
        Ipc::register(&path);
        self.push_recent(path.clone());
        let ps = self.current_panel_state();
        self.root_ws = Some(init_workspace(path, &ps));
        self.save_session();
    }

    fn do_startup_wizard(&mut self, ctx: &egui::Context) {
        let shared = Arc::clone(&self.shared);
        let mut success_path: Option<PathBuf> = None;
        show_project_wizard(
            ctx,
            &mut self.startup_wizard,
            &mut self.show_startup_wizard,
            "startup_wizard_modal",
            &shared,
            |path, _sh| {
                success_path = Some(path);
            },
        );
        if let Some(path) = success_path {
            self.open_workspace_from_startup(ctx, path);
        }
    }
}

// ---------------------------------------------------------------------------
// Drop — úklid při ukončení
// ---------------------------------------------------------------------------

impl Drop for EditorApp {
    fn drop(&mut self) {
        self.save_session();
        Ipc::unregister();
        let _ = std::fs::remove_file(ipc::process_socket_path(std::process::id()));
    }
}

// ---------------------------------------------------------------------------
// eframe::App implementace
// ---------------------------------------------------------------------------

impl eframe::App for EditorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let ps = self.current_panel_state();
        eframe::set_value(storage, STORAGE_KEY, &ps);
        self.save_session();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // IPC focus request
        if self.focus_rx.try_recv().is_ok() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // Zachytit křížek kořenového okna
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.quit_confirmed {
                // Potvrzeno — nechat zavřít
            } else if self.root_ws.is_some() {
                // Zavřít aktuální workspace (přejít na startup)
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.save_session();
                self.root_ws = None;
                ctx.send_viewport_cmd(egui::ViewportCommand::Title("Rust Editor".to_string()));
            } else {
                // Startup dialog — ukončit aplikaci
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_quit_confirm = true;
            }
        }

        // Registrovat deferred viewporty pro stávající sekundární workspacy
        // (musí být voláno každý snímek, aby okna zůstala otevřená)
        self.register_deferred_viewports(ctx);

        // Renderovat obsah kořenového viewportu
        if self.root_ws.is_none() {
            self.do_startup_dialog(ctx);
            self.do_startup_wizard(ctx);
        } else {
            // Dočasně vyjmout root_ws, aby bylo možné volat &mut self
            let mut ws = self.root_ws.take().unwrap();
            let reinit = render_workspace(ctx, &mut ws, &self.shared);
            if let Some(new_path) = reinit {
                let panel = ws_to_panel_state(&ws);
                let new_path_clone = new_path.clone();
                ws = init_workspace(new_path, &panel);
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                    format!("Rust Editor — {}", ws.root_path.display())
                ));
                self.push_recent(new_path_clone);
                self.save_session();
            }
            self.root_ws = Some(ws);
        }

        // Zpracovat akce z tohoto snímku (nové projekty, zavřené workspacy atd.)
        // Voláno ZA renderem, aby se akce z kliknutí zpracovaly okamžitě
        self.process_actions(ctx);

        // Znovu zaregistrovat viewporty — pro nové sekundární workspacy přidané výše
        self.register_deferred_viewports(ctx);

        // Dialog potvrzení ukončení
        if self.show_quit_confirm {
            self.show_quit_confirm_dialog(ctx);
        }
    }
}
