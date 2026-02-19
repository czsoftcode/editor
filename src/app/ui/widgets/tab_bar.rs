use eframe::egui;

/// Akce, kterou může vrátit jakýkoliv tab bar v aplikaci.
#[derive(Debug)]
pub(crate) enum TabBarAction {
    Switch(usize),
    Close(usize),
    New,
}

/// Popis jedné záložky předaný do [`render_compact_tab_bar`].
pub(crate) struct TabItem {
    pub label: String,
    pub closable: bool,
}

/// Vykreslí kompaktní řadu záložek (styl AI panelu):
/// číslované/pojmenované záložky s × pro zavření a + pro novou záložku.
///
/// Používá se tam, kde není potřeba scrollování — pro scrollovatelný
/// editor tab bar existuje specializovaná implementace v `editor/render.rs`.
pub(crate) fn render_compact_tab_bar(
    ui: &mut egui::Ui,
    items: &[TabItem],
    active: usize,
    show_new_btn: bool,
    close_hover: &str,
    new_hover: &str,
) -> Option<TabBarAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 1.0;

        for (i, item) in items.iter().enumerate() {
            let is_active = active == i;
            let label = egui::RichText::new(format!(" {} ", item.label)).monospace();

            let fill = if is_active {
                ui.visuals().extreme_bg_color
            } else {
                ui.visuals().faint_bg_color
            };
            let stroke = if is_active {
                egui::Stroke::new(1.0, ui.visuals().selection.stroke.color)
            } else {
                egui::Stroke::NONE
            };

            let tab_resp = ui.add(
                egui::Button::new(label)
                    .fill(fill)
                    .stroke(stroke)
                    .min_size(egui::vec2(0.0, 18.0)),
            );
            if tab_resp.clicked() && !is_active {
                action = Some(TabBarAction::Switch(i));
            }

            if item.closable {
                let close_resp = ui
                    .add(
                        egui::Button::new(egui::RichText::new("×").size(11.0))
                            .fill(egui::Color32::TRANSPARENT)
                            .min_size(egui::vec2(14.0, 18.0)),
                    )
                    .on_hover_text(close_hover);
                if close_resp.clicked() {
                    action = Some(TabBarAction::Close(i));
                }
                ui.add_space(3.0);
            }
        }

        if show_new_btn {
            let add_resp = ui
                .add(
                    egui::Button::new(egui::RichText::new("+").size(14.0))
                        .fill(egui::Color32::TRANSPARENT)
                        .min_size(egui::vec2(22.0, 18.0)),
                )
                .on_hover_text(new_hover);
            if add_resp.clicked() {
                action = Some(TabBarAction::New);
            }
        }
    });
    action
}
