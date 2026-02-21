use eframe::egui;

/// Result of a modal dialog.
#[derive(Debug)]
pub(crate) enum ModalResult<T> {
    /// User confirmed the dialog (clicked OK or pressed Enter).
    Confirmed(T),
    /// User cancelled the dialog (clicked Cancel, pressed Escape or closed the window).
    Cancelled,
    /// Dialog is still open, no action has occurred yet.
    Pending,
}

/// Displays a modal window with a title, custom content, and OK/Cancel buttons.
///
/// - `content` receives `&mut egui::Ui` and returns `Option<T>`.
///   If it returns `None`, the OK button is disabled.
///   If it returns `Some(v)`, OK is active — clicking it returns `Confirmed(v)`.
/// - Enter confirms (if `content` returns `Some`).
/// - Escape or closing returns `Cancelled`.
pub(crate) fn show_modal<T>(
    ctx: &egui::Context,
    id: impl Into<egui::Id>,
    title: &str,
    ok_label: &str,
    cancel_label: &str,
    content: impl FnOnce(&mut egui::Ui) -> Option<T>,
) -> ModalResult<T> {
    let mut result = ModalResult::Pending;

    let modal = egui::Modal::new(id.into());
    let modal_resp = modal.show(ctx, |ui| {
        ui.heading(title);
        ui.add_space(8.0);

        let value = content(ui);

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        let can_confirm = value.is_some();
        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

        ui.horizontal(|ui| {
            if (ui
                .add_enabled(can_confirm, egui::Button::new(ok_label))
                .clicked()
                || (can_confirm && enter_pressed))
                && let Some(v) = value
            {
                result = ModalResult::Confirmed(v);
            }
            if ui.button(cancel_label).clicked() {
                result = ModalResult::Cancelled;
            }
        });
    });

    if modal_resp.should_close() {
        result = ModalResult::Cancelled;
    }

    result
}
