use eframe::egui;
use egui_commonmark::CommonMarkViewer;

use super::Editor;

impl Editor {
    pub(super) fn render_markdown_preview(
        ui: &mut egui::Ui,
        cache: &mut egui_commonmark::CommonMarkCache,
        content: &str,
    ) {
        // Render using egui_commonmark.
        // Style (colors and size) is controlled by the parent element in render.rs.
        CommonMarkViewer::new()
            .syntax_theme_dark("base16-ocean.dark")
            .syntax_theme_light("base16-ocean.light")
            .show(ui, cache, content);
    }
}
