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
        modal.ui_footer(ui, |ui| {
            if ui.button(i18n.get("quit-confirm")).clicked() {
                confirmed = true;
            }
            if ui.button(i18n.get("quit-cancel")).clicked() {
                cancelled = true;
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
        modal.ui_footer(ui, |ui| {
            if ui.button(i18n.get("close-project-confirm")).clicked() {
                confirmed = true;
            }
            if ui.button(i18n.get("close-project-cancel")).clicked() {
                cancelled = true;
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
