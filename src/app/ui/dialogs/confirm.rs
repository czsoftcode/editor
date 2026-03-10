use crate::app::ui::widgets::modal::StandardModal;
use eframe::egui;

pub(crate) enum QuitDialogResult {
    Confirmed,
    Cancelled,
    Open,
}

pub(crate) fn show_quit_confirm_dialog(
    ctx: &egui::Context,
    i18n: &crate::i18n::I18n,
) -> QuitDialogResult {
    let mut show_flag = true;
    let mut confirmed = false;
    let mut cancelled = false;

    let modal =
        StandardModal::new(i18n.get("quit-title"), "quit_confirm_modal").with_size(400.0, 250.0);

    modal.show(ctx, &mut show_flag, |ui| {
        modal.ui_footer_actions(ui, i18n, |f| {
            if f.button("quit-cancel").clicked() {
                cancelled = true;
            }
            if f.button("quit-confirm").clicked() {
                confirmed = true;
            }
            None::<()>
        });

        modal.ui_body(ui, |ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(i18n.get("quit-message")).size(14.0));
            ui.add_space(12.0);
        });
    });

    if confirmed {
        QuitDialogResult::Confirmed
    } else if cancelled || !show_flag {
        QuitDialogResult::Cancelled
    } else {
        QuitDialogResult::Open
    }
}

/// Decision made in the unsaved close guard dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnsavedGuardDecision {
    Save,
    Discard,
    Cancel,
    /// No decision yet, dialog still open.
    Pending,
}

/// Shows the unsaved close guard dialog for a single queue item.
///
/// - Always offers `Save`, `Discard`, `Cancel` with a safe default of `Cancel`.
/// - Closing via `Esc` or window close (`X`) is treated as `Cancel`.
pub(crate) fn show_unsaved_close_guard_dialog(
    ctx: &egui::Context,
    i18n: &crate::i18n::I18n,
    file_name: &str,
    file_path: &str,
    inline_error: Option<&str>,
) -> UnsavedGuardDecision {
    let mut show_flag = true;
    let mut decision = UnsavedGuardDecision::Pending;

    let modal = StandardModal::new(i18n.get("unsaved-close-guard-title"), "unsaved_close_guard")
        .with_size(520.0, 260.0)
        .with_close_on_click_outside(true);

    modal.show(ctx, &mut show_flag, |ui| {
        modal.ui_body(ui, |ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(i18n.get("unsaved-close-guard-message"))
                    .size(14.0)
                    .line_height(Some(20.0)),
            );
            ui.add_space(12.0);

            ui.label(
                egui::RichText::new(file_name)
                    .strong()
                    .size(14.0),
            );
            ui.monospace(file_path);

            if let Some(err) = inline_error {
                ui.add_space(8.0);
                ui.colored_label(egui::Color32::RED, err);
            }
        });

        modal.ui_footer_actions(ui, i18n, |f| {
            // Order matters: first button is rightmost and acts as the primary escape hatch.
            if f.button("unsaved-close-guard-cancel").clicked() {
                decision = UnsavedGuardDecision::Cancel;
            }
            if f.button("unsaved-close-guard-discard").clicked() {
                decision = UnsavedGuardDecision::Discard;
            }
            if f.button("unsaved-close-guard-save").clicked() {
                decision = UnsavedGuardDecision::Save;
            }
            None::<()>
        });
    });

    // Treat closing the window (including Esc / X / backdrop) as Cancel if no explicit choice was made.
    if decision == UnsavedGuardDecision::Pending && !show_flag {
        UnsavedGuardDecision::Cancel
    } else {
        decision
    }
}

pub(crate) fn show_close_project_confirm_dialog(
    ctx: &egui::Context,
    modal_id: &str,
    project_path: &str,
    i18n: &crate::i18n::I18n,
) -> QuitDialogResult {
    let mut show_flag = true;
    let mut confirmed = false;
    let mut cancelled = false;

    let modal =
        StandardModal::new(i18n.get("close-project-title"), modal_id).with_size(450.0, 280.0);

    modal.show(ctx, &mut show_flag, |ui| {
        modal.ui_footer_actions(ui, i18n, |f| {
            if f.button("close-project-cancel").clicked() {
                cancelled = true;
            }
            if f.button("close-project-confirm").clicked() {
                confirmed = true;
            }
            None::<()>
        });

        modal.ui_body(ui, |ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(i18n.get("close-project-message")).size(14.0));
            ui.monospace(project_path);
            ui.add_space(12.0);
        });
    });

    if confirmed {
        QuitDialogResult::Confirmed
    } else if cancelled || !show_flag {
        QuitDialogResult::Cancelled
    } else {
        QuitDialogResult::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsaved_close_guard_esc_cancel() {
        let ctx = egui::Context::default();

        ctx.begin_pass(egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        });

        assert!(consume_unsaved_guard_escape(&ctx));
        assert!(!consume_unsaved_guard_escape(&ctx));
        assert_eq!(
            resolve_unsaved_guard_decision(UnsavedGuardDecision::Pending, true, true),
            UnsavedGuardDecision::Cancel
        );

        let _ = ctx.end_pass();
    }
}
