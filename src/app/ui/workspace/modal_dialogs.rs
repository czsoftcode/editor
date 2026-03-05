use eframe::egui;
use std::sync::{Arc, Mutex};

use super::super::super::types::{AppAction, AppShared};
use super::super::dialogs::show_project_wizard;
use super::state::WorkspaceState;

mod about;
mod ai_dialogs;
mod conflict;
mod plugins;
mod settings;
mod terminal;

pub(crate) use settings::restore_runtime_settings_from_snapshot;

// ---------------------------------------------------------------------------
// render_dialogs
// ---------------------------------------------------------------------------

/// Renders modal dialogs (About, Settings, New project, Conflict). Returns true if interacted with.
pub(super) fn render_dialogs(
    ctx: &egui::Context,
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) -> bool {
    let any_interacted = false;
    // Salt ensures uniqueness of modal ID within the egui Context, which is shared
    // between all windows (viewports). Without salt, two windows with the same dialog
    // would share state (open/closed, form values).
    let id_salt = ws.root_path.as_os_str().to_os_string();

    // 1. About dialog
    about::show(ctx, ws, i18n, &id_salt);

    // 1a. Support dialog
    crate::app::ui::dialogs::show_support_dialog(ctx, ws, i18n);

    // 2. Settings dialog
    settings::show(ctx, ws, shared, i18n, &id_salt);

    // 3. Plugins dialog
    plugins::show(ctx, ws, shared, i18n, &id_salt);

    // 4. New project wizard (within workspace)
    if ws.show_new_project {
        let wizard_modal_id = format!("ws_new_project_modal_{}", ws.root_path.display());
        let mut wizard_state = std::mem::take(&mut ws.wizard);
        let mut show_flag = ws.show_new_project;

        let args = crate::app::ui::dialogs::WizardArgs {
            ctx,
            state: &mut wizard_state,
            show: &mut show_flag,
            modal_id: &wizard_modal_id,
            shared,
            i18n,
            ws: Some(ws),
        };

        show_project_wizard(args, |path, sh| {
            let mut sh = sh
                .lock()
                .expect("Failed to lock AppShared in new project wizard callback");
            sh.actions.push(AppAction::AddRecent(path.clone()));
            sh.actions.push(AppAction::OpenInNewWindow(path));
        });
        ws.wizard = wizard_state;
        ws.show_new_project = show_flag;
    }

    // 4. External change conflict dialog
    conflict::show(ctx, ws, shared, i18n, &id_salt);

    // 5. Terminal close confirmation dialog
    terminal::show(ctx, ws, i18n, &id_salt);

    // 6. AI related dialogs (Promotion success, Sandbox staged files, Sync confirmation)
    ai_dialogs::show(ctx, ws, shared, i18n);

    // 7. Global confirm discard dialog
    if crate::app::ui::widgets::modal::render_confirm_discard_dialog(ctx, ws, i18n) {
        // Settings modal may have already pushed live preview into shared runtime settings.
        // Confirm-discard must always restore snapshot and clear draft lifecycle state.
        settings::discard_settings_draft(ws, shared);

        // If confirmed, close common modals
        ws.show_settings = false;
        ws.show_plugins = false;
        ws.show_new_project = false;
        ws.show_ai_chat = false;
        ws.plugins_draft = None;
    }

    any_interacted
}
