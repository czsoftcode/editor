use eframe::egui;
use std::sync::{Arc, Mutex};

use super::super::super::types::{AppAction, AppShared};
use super::super::dialogs::show_project_wizard;
use super::state::WorkspaceState;

mod about;
mod ai;
mod ai_chat;
mod conflict;
mod plugins;
mod settings;
mod terminal;

// ---------------------------------------------------------------------------
// render_dialogs
// ---------------------------------------------------------------------------

/// Renders modal dialogs (About, Settings, New project, Conflict).
pub(super) fn render_dialogs(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) {
    // Salt ensures uniqueness of modal ID within the egui Context, which is shared
    // between all windows (viewports). Without salt, two windows with the same dialog
    // would share state (open/closed, form values).
    let id_salt = ws.root_path.as_os_str().to_os_string();

    // 1. About dialog
    about::show(ctx, ws, i18n, &id_salt);

    // 2. Settings dialog
    settings::show(ctx, ws, shared, i18n, &id_salt);

    // 3. Plugins dialog
    plugins::show(ctx, ws, shared, i18n, &id_salt);

    // 4. AI Chat dialog
    ai_chat::show(ctx, ws, shared, i18n, &id_salt);

    // 5. New project wizard (within workspace)
    if ws.show_new_project {
        let wizard_modal_id = format!("ws_new_project_modal_{}", ws.root_path.display());
        show_project_wizard(
            ctx,
            &mut ws.wizard,
            &mut ws.show_new_project,
            &wizard_modal_id,
            shared,
            i18n,
            |path, sh| {
                let mut sh = sh
                    .lock()
                    .expect("Failed to lock AppShared in new project wizard callback");
                sh.actions.push(AppAction::AddRecent(path.clone()));
                sh.actions.push(AppAction::OpenInNewWindow(path));
            },
        );
    }

    // 4. External change conflict dialog
    conflict::show(ctx, ws, shared, i18n, &id_salt);

    // 5. Terminal close confirmation dialog
    terminal::show(ctx, ws, i18n, &id_salt);

    // 6. AI related dialogs (Promotion success, Sandbox staged files, Sync confirmation)
    ai::show(ctx, ws, shared, i18n);
}
