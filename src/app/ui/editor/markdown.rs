use eframe::egui;
use egui_commonmark::CommonMarkViewer;

use super::Editor;

impl Editor {
    pub(super) fn render_markdown_preview(&mut self, ui: &mut egui::Ui, content: &str) {
        // Vykreslení pomocí egui_commonmark.
        // Styl (barvy a velikost) je řízen nadřazeným prvkem v render.rs.
        CommonMarkViewer::new()
            .show(ui, &mut self.markdown_cache, content);
    }
}
