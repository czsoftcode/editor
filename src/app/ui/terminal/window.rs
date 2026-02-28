use crate::app::types::FocusedPanel;
use crate::app::ui::workspace::state::WorkspaceState;
use eframe::egui;

pub struct StandardTerminalWindow {
    pub title: String,
    pub id: egui::Id,
    pub focused_panel_type: FocusedPanel,
}

impl StandardTerminalWindow {
    pub fn new(
        title: impl Into<String>,
        id: impl Into<egui::Id>,
        panel_type: FocusedPanel,
    ) -> Self {
        Self {
            title: title.into(),
            id: id.into(),
            focused_panel_type: panel_type,
        }
    }

    pub fn show<R>(
        &self,
        ctx: &egui::Context,
        ws: &mut WorkspaceState,
        open: &mut bool,
        render_head: impl FnOnce(&mut egui::Ui, &mut WorkspaceState),
        render_body: impl FnOnce(&mut egui::Ui, &mut WorkspaceState, f32) -> Option<R>,
        render_footer: impl FnOnce(&mut egui::Ui, &mut WorkspaceState) -> Option<R>,
    ) -> (bool, Option<R>) {
        let mut interacted = false;
        let mut result = None;

        let viewer_bg = egui::Color32::from_rgb(20, 20, 25);
        let screen_rect = ctx.screen_rect();
        let max_w = screen_rect.width() * 0.9;
        let max_h = screen_rect.height() * 0.9;

        let window_res = egui::Window::new(&self.title)
            .id(self.id)
            .default_size([800.0, 600.0])
            .min_size([300.0, 200.0])
            .max_width(max_w)
            .max_height(max_h)
            .resizable(true)
            .collapsible(true)
            .vscroll(false)
            .open(open)
            .frame(egui::Frame::window(&ctx.style()).fill(viewer_bg))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;

                    // HEAD
                    ui.scope(|ui| {
                        render_head(ui, ws);
                    });
                    ui.separator();

                    // BODY
                    let footer_reserved = 40.0;
                    let body_h = (ui.available_height() - footer_reserved).max(100.0);

                    ui.allocate_ui(egui::vec2(ui.available_width(), body_h), |ui| {
                        if let Some(res) = render_body(ui, ws, body_h) {
                            result = Some(res);
                            interacted = true;
                        }
                    });

                    // FOOTER
                    ui.separator();
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if let Some(res) = render_footer(ui, ws) {
                            result = Some(res);
                            interacted = true;
                        }
                    });
                });
            });

        // Focus management using z-ordering:
        // - inner.response.rect contains the full window area (including child widgets)
        // - ctx.layer_id_at(pos) returns the TOPMOST layer at the pointer position,
        //   so overlapping windows won't steal focus from the one the user actually clicked.
        if let Some(inner) = window_res {
            let pointer_active = ctx.input(|i| i.pointer.any_click() || i.pointer.any_down());
            if pointer_active {
                let pos = ctx.input(|i| i.pointer.interact_pos());
                if let Some(pos) = pos
                    && inner.response.rect.contains(pos)
                {
                    let our_layer = egui::LayerId::new(egui::Order::Middle, self.id);
                    let is_topmost = ctx
                        .layer_id_at(pos)
                        .map(|top| top == our_layer)
                        .unwrap_or(false);

                    if is_topmost {
                        interacted = true;
                    }
                }
            }
        }

        if ws.focused_panel == self.focused_panel_type {
            interacted = true;
        }

        (interacted, result)
    }
}
