use crate::app::ui::workspace::state::WorkspaceState;
use crate::i18n::I18n;
use eframe::egui;

pub fn render_inspector(ui: &mut egui::Ui, ws: &mut WorkspaceState, font_size: f32, i18n: &I18n) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("\u{1F50D} {}", i18n.get("cli-chat-inspector-title"))).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(i18n.get("cli-chat-clear")).clicked() {
                    ws.ai.chat.last_payload.clear();
                }
                if ui.button(i18n.get("cli-chat-copy")).clicked() {
                    ui.ctx().copy_text(ws.ai.chat.last_payload.clone());
                }
            });
        });
        ui.add_space(4.0);
        egui::ScrollArea::both()
            .id_salt("ai_chat_terminal_inspector_scroll")
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut ws.ai.chat.last_payload)
                        .font(egui::FontId::monospace(font_size * 0.9))
                        .desired_width(f32::INFINITY)
                        .code_editor(),
                );
            });
    });
}
