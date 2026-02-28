use crate::app::types::{AppAction, AppShared, PersistentState};
use crate::app::ui::dialogs::{StartupAction, show_project_wizard, show_startup_dialog};
use crate::app::ui::workspace::{WorkspaceState, init_workspace, ws_to_panel_state};
use crate::ipc::Ipc;
use eframe::egui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct StartupState<'a> {
    pub root_ws: &'a mut Option<WorkspaceState>,
    pub shared: &'a Arc<Mutex<AppShared>>,
    pub path_buffer: &'a mut String,
    pub show_startup_wizard: &'a mut bool,
    pub startup_wizard: &'a mut crate::app::ui::dialogs::WizardState,
    pub startup_browse_rx: &'a mut Option<std::sync::mpsc::Receiver<Option<PathBuf>>>,
    pub missing_session_paths: &'a mut Vec<PathBuf>,
    pub saved_panel_state: &'a PersistentState,
}

pub fn render(ctx: &egui::Context, state: StartupState) {
    if state.root_ws.is_some() {
        return;
    }

    // 1. Startup Dialog
    let (recent_snapshot, i18n_arc) = {
        let shared = state
            .shared
            .lock()
            .expect("Failed to lock AppShared for startup dialog state");
        (
            shared.recent_projects.clone(),
            std::sync::Arc::clone(&shared.i18n),
        )
    };

    let action = show_startup_dialog(
        ctx,
        state.path_buffer,
        *state.show_startup_wizard,
        &recent_snapshot,
        state.startup_browse_rx,
        state.missing_session_paths,
        &i18n_arc,
    );

    match action {
        StartupAction::OpenPath(path) => {
            open_workspace(
                ctx,
                path,
                state.root_ws,
                state.shared,
                state.saved_panel_state,
                state.missing_session_paths,
            );
        }
        StartupAction::OpenWizard => {
            *state.show_startup_wizard = true;
        }
        StartupAction::QuitApp => {
            // This is handled in the main loop by setting show_quit_confirm,
            // but for simplicity we can trigger it via an action if needed.
            state
                .shared
                .lock()
                .expect("Failed to lock AppShared for quit app action in startup")
                .actions
                .push(AppAction::QuitAll);
        }
        StartupAction::None => {}
    }

    // 2. Startup Wizard
    if *state.show_startup_wizard {
        let mut success_path: Option<PathBuf> = None;
        show_project_wizard(
            ctx,
            state.startup_wizard,
            state.show_startup_wizard,
            "startup_wizard_modal",
            state.shared,
            &i18n_arc,
            |path, _sh| {
                success_path = Some(path);
            },
        );
        if let Some(path) = success_path {
            open_workspace(
                ctx,
                path,
                state.root_ws,
                state.shared,
                state.saved_panel_state,
                state.missing_session_paths,
            );
        }
    }
}

fn open_workspace(
    ctx: &egui::Context,
    path: PathBuf,
    root_ws: &mut Option<WorkspaceState>,
    shared: &Arc<Mutex<AppShared>>,
    saved_panel_state: &PersistentState,
    missing_session_paths: &mut Vec<PathBuf>,
) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
        "PolyCredo Editor — {}",
        path.display()
    )));
    Ipc::register(&path);

    // Add to recent
    {
        let mut s = shared
            .lock()
            .expect("Failed to lock AppShared for adding to recent in open_workspace");
        s.recent_projects.retain(|p| p != &path);
        s.recent_projects.insert(0, path.clone());
        s.recent_projects
            .truncate(crate::config::MAX_RECENT_PROJECTS);
    }
    Ipc::add_recent(&path);

    let ps = root_ws
        .as_ref()
        .map(ws_to_panel_state)
        .unwrap_or_else(|| saved_panel_state.clone());

    let settings = shared.lock().expect("lock").settings.clone();
    // User opened a new project — missing session projects are no longer relevant
    missing_session_paths.clear();
    *root_ws = Some(init_workspace(
        path,
        &ps,
        ctx.clone(),
        &settings,
        Arc::clone(shared),
    ));

    // Save session
    let mut paths = Vec::new();
    if let Some(ws) = root_ws {
        paths.push(ws.root_path.clone());
    }
    crate::ipc::save_session(&paths);
}
