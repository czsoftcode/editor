use eframe::egui;
use std::sync::{Arc, Mutex};

use super::super::super::types::{AppAction, AppShared};
use super::super::dialogs::show_project_wizard;
use super::state::WorkspaceState;

mod about;
mod ai_dialogs;
mod conflict;
mod settings;
mod terminal;

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
    settings::show(ctx, ws, shared, i18n);

    // 3. New project wizard (within workspace)
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
            sh.actions.push(AppAction::OpenWithChoice(path));
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
        ws.show_new_project = false;
    }

    any_interacted
}

pub(super) fn save_settings_draft(
    ws: &mut WorkspaceState,
    shared: &Arc<Mutex<AppShared>>,
    i18n: &crate::i18n::I18n,
) {
    settings::save_settings_draft(ws, shared, i18n);
}

// ---------------------------------------------------------------------------
// OpenChoice — modal pro výběr kam otevřít projekt
// ---------------------------------------------------------------------------

/// Výsledek modalu "Kam otevřít projekt?"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpenChoice {
    /// Otevřít v novém okně.
    NewWindow,
    /// Otevřít ve stávajícím okně (nahradit aktuální projekt).
    CurrentWindow,
    /// Uživatel zrušil výběr.
    Cancelled,
    /// Modal je stále otevřený, žádná volba zatím.
    Pending,
}

/// Zobrazí modal se 3 tlačítky: "Nové okno" (výchozí), "Stávající okno", "Zrušit".
pub(super) fn show_open_choice_modal(
    ctx: &egui::Context,
    id_salt: &std::ffi::OsString,
    i18n: &crate::i18n::I18n,
) -> OpenChoice {
    use crate::app::ui::widgets::modal::StandardModal;

    let mut show_flag = true;
    let mut choice = OpenChoice::Pending;
    let esc_pressed =
        ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    if esc_pressed {
        return OpenChoice::Cancelled;
    }

    let modal_id = format!("open_choice_modal_{}", id_salt.to_string_lossy());
    let modal = StandardModal::new(i18n.get("open-choice-title"), &modal_id)
        .with_size(420.0, 220.0)
        .with_close_on_click_outside(true);

    modal.show(ctx, &mut show_flag, |ui| {
        modal.ui_body(ui, |ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(i18n.get("open-choice-description"))
                    .size(14.0)
                    .line_height(Some(20.0)),
            );
            ui.add_space(12.0);
        });

        modal.ui_footer(ui, |ui| {
            // Pořadí: cancel nejdřív (bude vpravo), pak stávající okno, pak nové okno (vlevo = primární)
            if ui.button(i18n.get("open-choice-cancel")).clicked() {
                choice = OpenChoice::Cancelled;
            }
            if ui.button(i18n.get("open-choice-current-window")).clicked() {
                choice = OpenChoice::CurrentWindow;
            }
            // Primární tlačítko — "Nové okno" zvýrazněné
            let new_window_btn =
                egui::Button::new(egui::RichText::new(i18n.get("open-choice-new-window")).strong());
            if ui.add(new_window_btn).clicked() {
                choice = OpenChoice::NewWindow;
            }
            None::<()>
        });
    });

    // Backdrop klik (show_flag se nastaví na false) = Cancel
    if !show_flag && choice == OpenChoice::Pending {
        choice = OpenChoice::Cancelled;
    }

    choice
}
