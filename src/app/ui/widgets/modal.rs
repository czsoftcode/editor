use eframe::egui;

/// Výsledek modálního dialogu.
#[derive(Debug)]
pub(crate) enum ModalResult<T> {
    /// Uživatel potvrdil dialog (kliknul OK nebo stiskl Enter).
    Confirmed(T),
    /// Uživatel dialog zrušil (kliknul Zrušit, stiskl Escape nebo zavřel okno).
    Cancelled,
    /// Dialog je stále otevřen, žádná akce zatím neproběhla.
    Pending,
}

/// Zobrazí modální okno s nadpisem, vlastním obsahem a tlačítky OK/Zrušit.
///
/// - `content` dostane `&mut egui::Ui` a vrátí `Option<T>`.
///   Pokud vrátí `None`, je tlačítko OK deaktivováno.
///   Pokud vrátí `Some(v)`, je OK aktivní — kliknutím vrátí `Confirmed(v)`.
/// - Enter potvrzuje (pokud `content` vrátí `Some`).
/// - Escape nebo zavření vrátí `Cancelled`.
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
            if ui
                .add_enabled(can_confirm, egui::Button::new(ok_label))
                .clicked()
                || (can_confirm && enter_pressed)
            {
                if let Some(v) = value {
                    result = ModalResult::Confirmed(v);
                }
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
