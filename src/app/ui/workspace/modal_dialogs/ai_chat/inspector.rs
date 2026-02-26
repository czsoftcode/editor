use crate::app::ui::workspace::state::WorkspaceState;
use eframe::egui;

pub fn render_inspector(ui: &mut egui::Ui, ws: &mut WorkspaceState, font_size: f32) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("\u{1F50D} AI Inspector").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear").clicked() {
                    ws.ai_last_payload.clear();
                }
                if ui.button("Copy").clicked() {
                    ui.ctx().copy_text(ws.ai_last_payload.clone());
                }
            });
        });
        ui.add_space(4.0);
        egui::ScrollArea::both()
            .id_salt("inspector_scroll")
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut ws.ai_last_payload)
                        .font(egui::FontId::monospace(font_size * 0.9))
                        .desired_width(f32::INFINITY)
                        .code_editor(),
                );
            });
    });
}
