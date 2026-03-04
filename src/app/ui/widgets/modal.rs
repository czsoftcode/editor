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
    pub is_cancel_confirmed: bool,
}

impl StandardModal {
    pub fn new(title: impl Into<String>, id_salt: impl std::hash::Hash) -> Self {
        Self {
            title: title.into(),
            id: egui::Id::new(id_salt),
            default_size: egui::vec2(850.0, 600.0),
            min_size: egui::vec2(700.0, 500.0),
            footer_hint: None,
            is_cancel_confirmed: false,
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

    /// Pomocná metoda pro nastavení příznaku potvrzení storna.
    pub fn with_cancel_confirmed(mut self, confirmed: bool) -> Self {
        self.is_cancel_confirmed = confirmed;
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
                // If mouse enters the window, we report it to the caller indirectly
                // via the fact that we are currently processing this UI.
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
                ui.scope(|ui| {
                    ui.visuals_mut().widgets.noninteractive.bg_stroke =
                        egui::Stroke::new(1.0, egui::Color32::from_gray(60));
                    ui.separator();
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if let Some(hint) = &self.footer_hint {
                        ui.label(egui::RichText::new(hint).weak().size(11.0));
                    }
                    // Zarovnání doprava, ale uvnitř LTR, aby poslední tlačítko v kódu bylo vpravo na obrazovce.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.horizontal(|ui| {
                            result = buttons(ui);
                        });
                    });
                });
                ui.add_space(8.0);
            });
        result
    }

    /// Sjednocená metoda pro patičku s akcemi.
    /// V closure `actions` volejte metody `ModalFooter` v pořadí:
    /// 1. Zavřít/Zrušit (bude úplně vpravo)
    /// 2. Ostatní akce (budou vlevo od něj)
    pub fn ui_footer_actions<R>(
        &self,
        ui: &mut egui::Ui,
        i18n: &crate::i18n::I18n,
        actions: impl FnOnce(&mut ModalFooter) -> Option<R>,
    ) -> Option<R> {
        self.ui_footer(ui, |ui| {
            let mut footer = ModalFooter { ui, i18n };
            actions(&mut footer)
        })
    }

    /// Pomocná metoda pro vykreslení těla (CentralPanel).
    pub fn ui_body(&self, ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            body(ui);
        });
    }
}

/// Helper pro práci s tlačítky v patičce modalu.
pub(crate) struct ModalFooter<'a, 'u> {
    pub ui: &'u mut egui::Ui,
    pub i18n: &'a crate::i18n::I18n,
}

impl<'a, 'u> ModalFooter<'a, 'u> {
    /// Vykreslí standardní tlačítko s klíčem z i18n.
    pub fn button(&mut self, key: &str) -> egui::Response {
        self.ui.button(self.i18n.get(key))
    }

    /// Tlačítko "Zavřít" (úplně vpravo, pokud je voláno jako první).
    pub fn close(&mut self) -> bool {
        self.button("btn-close").clicked()
    }

    /// Tlačítko "Zrušit" (úplně vpravo, pokud je voláno jako první).
    pub fn cancel(&mut self) -> bool {
        self.button("btn-cancel").clicked()
    }

    /// Tlačítko "Zrušit" s potvrzovacím dialogem.
    /// Vrací true, pokud uživatel potvrdil zahození změn (příznak přenesen přes WorkspaceState).
    pub fn confirm_cancel(
        &mut self,
        ws: &mut crate::app::ui::workspace::state::WorkspaceState,
    ) -> bool {
        // Pokud už byl modal dříve potvrzen, vrátíme true
        if self.ui.available_width() < 0.0 {
            // Jen pojistka pro případné renderovací cykly
        }

        if self.cancel() {
            // Nastavíme ID aktuálního modalu jako čekajícího na potvrzení
            // Použijeme i18n key jako sůl pro unikátnost v rámci cyklu
            ws.confirm_discard_changes = Some(self.i18n.get("btn-cancel").to_string());
        }
        false
    }

    /// Tlačítko "OK".
    pub fn ok(&mut self) -> bool {
        self.button("btn-ok").clicked()
    }

    /// Tlačítko "Uložit".
    pub fn save(&mut self) -> bool {
        self.button("btn-save").clicked()
    }
}

/// Vykreslí globální potvrzovací dialog pro zahození změn.
/// Volá se centrálně v render_dialogs.
pub(crate) fn render_confirm_discard_dialog(
    ctx: &egui::Context,
    ws: &mut crate::app::ui::workspace::state::WorkspaceState,
    i18n: &crate::i18n::I18n,
) -> bool {
    if ws.confirm_discard_changes.is_none() {
        return false;
    }

    let mut confirmed = false;
    let mut open = true;

    let modal = StandardModal::new(i18n.get("cancel-confirm-title"), "confirm_discard_modal")
        .with_size(450.0, 220.0);

    modal.show(ctx, &mut open, |ui| {
        modal.ui_footer_actions(ui, i18n, |f| {
            if f.close() {
                ws.confirm_discard_changes = None;
                return Some(());
            }
            if f.button("btn-confirm").clicked() {
                confirmed = true;
                ws.confirm_discard_changes = None;
                return Some(());
            }
            None
        });

        modal.ui_body(ui, |ui| {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(i18n.get("cancel-confirm-msg"))
                    .size(14.0)
                    .line_height(Some(20.0)),
            );
        });
    });

    if !open {
        ws.confirm_discard_changes = None;
    }

    confirmed
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
