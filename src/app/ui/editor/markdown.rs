use eframe::egui;
use egui_commonmark::CommonMarkViewer;

use super::Editor;

impl Editor {
    pub(super) fn render_markdown_preview(&mut self, ui: &mut egui::Ui, content: &str) {
        // Render using egui_commonmark.
        // Style (colors and size) is controlled by the parent element in render.rs.
        CommonMarkViewer::new()
            .show(ui, &mut self.markdown_cache, content);
    }
}
