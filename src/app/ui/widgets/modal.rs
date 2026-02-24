use eframe::egui;

/// Univerzální výsledek modalu.
#[derive(Debug)]
pub(crate) enum ModalResult<T> {
    Confirmed(T),
    Cancelled,
    Pending,
}

/// Základní šablona pro velká okna (Nastavení, Pluginy, atd.)
pub(crate) struct StandardModal {
    pub title: String,
    pub id: egui::Id,
    pub default_size: egui::Vec2,
    pub min_size: egui::Vec2,
    pub footer_hint: Option<String>,
}

impl StandardModal {
    pub fn new(title: impl Into<String>, id_salt: impl std::hash::Hash) -> Self {
        Self {
            title: title.into(),
            id: egui::Id::new(id_salt),
            default_size: egui::vec2(850.0, 600.0),
            min_size: egui::vec2(700.0, 500.0),
            footer_hint: None,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.default_size = egui::vec2(width, height);
        self.min_size = egui::vec2(width.min(700.0), height.min(500.0));
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.footer_hint = Some(hint.into());
        self
    }

    /// Vykreslí modal s jednotným stylem.
    /// Nyní je posouvatelný a má zavírací tlačítko v záhlaví.
    pub fn show<R>(
        &self,
        ctx: &egui::Context,
        show_flag: &mut bool,
        content: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<R> {
        if !*show_flag {
            return None;
        }

        let mut result = None;

        egui::Window::new(&self.title)
            .id(self.id)
            .open(show_flag) // Umožní zavření křížkem v záhlaví
            .collapsible(false)
            .resizable(true)
            .default_size(self.default_size)
            .min_width(self.min_size.x)
            .min_height(self.min_size.y)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                ui.set_min_height(self.min_size.y - 20.0);
                result = Some(content(ui));
            });

        result
    }

    /// Pomocná metoda pro vykreslení patičky s tlačítky.
    pub fn ui_footer<R>(
        &self,
        ui: &mut egui::Ui,
        buttons: impl FnOnce(&mut egui::Ui) -> Option<R>,
    ) -> Option<R> {
        let mut result = None;
        egui::TopBottomPanel::bottom(self.id.with("footer"))
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if let Some(hint) = &self.footer_hint {
                        ui.label(egui::RichText::new(hint).weak().size(11.0));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        result = buttons(ui);
                    });
                });
                ui.add_space(10.0);
            });
        result
    }

    /// Pomocná metoda pro vykreslení těla (CentralPanel).
    pub fn ui_body(&self, ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            body(ui);
        });
    }
}

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
